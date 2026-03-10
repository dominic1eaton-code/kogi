package mesh

// mesh.go
//
// Service-mesh registry used by the Kogi gateway and all Go services.
//
// New capabilities added to support the full PortfolioSystem event surface:
//
//   Component lifecycle
//     Unregister(id) — remove a component from the registry.
//     UpdateStatus(id, status) — change a component's status in-place.
//     ComponentByID(id) — look up a single component without iterating.
//     ComponentsByKind(kind) — filter components by kind ("service", "gateway", …).
//     ComponentsByStatus(status) — filter components by status ("active", "degraded", …).
//
//   Topic routing
//     SetTopicRoutes(map) — bulk-set multiple topic routes under one lock.
//     DeleteTopicRoute(topic) — remove a single route.
//     TopicRoutesForTarget(target) — reverse-lookup all topics that route to a
//       given component ID.
//
//   Message routing
//     SendBatch(messages []SendRequest) — send multiple messages atomically.
//     MessageHistoryByTopic(topic, limit) — filter message history by topic.
//     MessageHistoryBySource(source, limit) — filter by source component ID.
//     MessageHistoryByStatus(status, limit) — filter by delivery status.
//     MessageStats() — per-topic and per-status counts.
//
//   Health tracking
//     RecordHealth(id, healthy, note) — record a health check result.
//     HealthHistory(id, limit) — return recent health records for a component.
//     ComponentHealth(id) — return the latest health record.
//
//   Mesh metrics
//     Metrics() — snapshot of component count, topic route count, message count,
//       and per-status delivery tallies.

import (
	"fmt"
	"sort"
	"sync"
	"sync/atomic"
	"time"
)

// ── Public types ──────────────────────────────────────────────────────────────

// Component is a registered mesh participant (service, gateway, engine, …).
type Component struct {
	ID             string            `json:"id"`
	Kind           string            `json:"kind"`
	Endpoint       string            `json:"endpoint"`
	HealthPath     string            `json:"health_path,omitempty"`
	NetworkManager string            `json:"network_manager"`
	Status         string            `json:"status"`
	Metadata       map[string]string `json:"metadata,omitempty"`
	RegisteredAt   string            `json:"registered_at"`
	UpdatedAt      string            `json:"updated_at,omitempty"`
}

// RoutedMessage is an immutable record of one mesh send operation.
type RoutedMessage struct {
	ID        string            `json:"id"`
	Source    string            `json:"source"`
	Target    string            `json:"target"`
	Topic     string            `json:"topic"`
	Payload   string            `json:"payload"`
	Status    string            `json:"status"`
	Metadata  map[string]string `json:"metadata,omitempty"`
	Timestamp string            `json:"timestamp"`
}

// SendRequest is a single entry for SendBatch.
type SendRequest struct {
	Source   string
	Target   string
	Topic    string
	Payload  string
	Metadata map[string]string
}

// HealthRecord is one health-check observation for a component.
type HealthRecord struct {
	ComponentID string `json:"component_id"`
	Healthy     bool   `json:"healthy"`
	Note        string `json:"note,omitempty"`
	Timestamp   string `json:"timestamp"`
}

// MeshMetrics is a read-only snapshot of registry activity.
type MeshMetrics struct {
	ComponentCount   int               `json:"component_count"`
	TopicRouteCount  int               `json:"topic_route_count"`
	TotalMessages    int               `json:"total_messages"`
	StatusCounts     map[string]int    `json:"component_status_counts"`
	KindCounts       map[string]int    `json:"component_kind_counts"`
	DeliveryStatuses map[string]uint64 `json:"delivery_status_counts"`
}

// ── Registry ──────────────────────────────────────────────────────────────────

// Registry is the central mesh registry.  All methods are safe for concurrent
// use from multiple goroutines.
type Registry struct {
	mu             sync.RWMutex
	components     map[string]Component
	topicRoutes    map[string]string
	messages       []RoutedMessage
	maxMessages    int
	sequence       uint64
	deliveryCounts map[string]uint64 // status → count

	// health tracking: component ID → []HealthRecord (newest last)
	healthMu      sync.RWMutex
	healthHistory map[string][]HealthRecord
	maxHealthPer  int
}

// NewRegistry returns a Registry with a 10 000-message history.
func NewRegistry() *Registry {
	return NewRegistryWithCapacity(10000)
}

// NewRegistryWithCapacity returns a Registry with a custom message capacity.
func NewRegistryWithCapacity(maxMessages int) *Registry {
	if maxMessages <= 0 {
		maxMessages = 1000
	}
	return &Registry{
		components:     map[string]Component{},
		topicRoutes:    map[string]string{},
		messages:       []RoutedMessage{},
		maxMessages:    maxMessages,
		deliveryCounts: map[string]uint64{},
		healthHistory:  map[string][]HealthRecord{},
		maxHealthPer:   200,
	}
}

// ── Component lifecycle ───────────────────────────────────────────────────────

// Register adds or replaces a component in the registry.
// Default values are applied for empty fields.
func (r *Registry) Register(component Component) Component {
	r.mu.Lock()
	defer r.mu.Unlock()

	now := time.Now().UTC().Format(time.RFC3339Nano)
	component.ID = defaultIfEmpty(component.ID, fmt.Sprintf("component-%d", atomic.AddUint64(&r.sequence, 1)))
	component.Kind = defaultIfEmpty(component.Kind, "service")
	component.Endpoint = defaultIfEmpty(component.Endpoint, "local://undefined")
	component.NetworkManager = defaultIfEmpty(component.NetworkManager, "kogi-go-network")
	component.Status = defaultIfEmpty(component.Status, "active")
	if component.Metadata == nil {
		component.Metadata = map[string]string{}
	}
	if existing, ok := r.components[component.ID]; ok {
		// Preserve original registration timestamp on re-register
		component.RegisteredAt = existing.RegisteredAt
	} else {
		component.RegisteredAt = now
	}
	component.UpdatedAt = now

	r.components[component.ID] = component
	return component
}

// Unregister removes a component from the registry.
// Returns the removed component and true if it was found.
func (r *Registry) Unregister(id string) (Component, bool) {
	r.mu.Lock()
	defer r.mu.Unlock()
	c, ok := r.components[id]
	if ok {
		delete(r.components, id)
	}
	return c, ok
}

// UpdateStatus changes the Status field of an existing component in-place.
// Returns the updated component and true if found.
func (r *Registry) UpdateStatus(id, status string) (Component, bool) {
	r.mu.Lock()
	defer r.mu.Unlock()
	c, ok := r.components[id]
	if !ok {
		return Component{}, false
	}
	c.Status = status
	c.UpdatedAt = time.Now().UTC().Format(time.RFC3339Nano)
	r.components[id] = c
	return c, true
}

// ComponentByID returns the component with the given ID.
func (r *Registry) ComponentByID(id string) (Component, bool) {
	r.mu.RLock()
	defer r.mu.RUnlock()
	c, ok := r.components[id]
	return c, ok
}

// Components returns all registered components sorted by ID.
func (r *Registry) Components() []Component {
	r.mu.RLock()
	defer r.mu.RUnlock()

	result := make([]Component, 0, len(r.components))
	for _, c := range r.components {
		result = append(result, c)
	}
	sort.Slice(result, func(i, j int) bool { return result[i].ID < result[j].ID })
	return result
}

// ComponentsByKind returns all components whose Kind matches kind.
func (r *Registry) ComponentsByKind(kind string) []Component {
	r.mu.RLock()
	defer r.mu.RUnlock()

	var result []Component
	for _, c := range r.components {
		if c.Kind == kind {
			result = append(result, c)
		}
	}
	sort.Slice(result, func(i, j int) bool { return result[i].ID < result[j].ID })
	return result
}

// ComponentsByStatus returns all components whose Status matches status.
func (r *Registry) ComponentsByStatus(status string) []Component {
	r.mu.RLock()
	defer r.mu.RUnlock()

	var result []Component
	for _, c := range r.components {
		if c.Status == status {
			result = append(result, c)
		}
	}
	sort.Slice(result, func(i, j int) bool { return result[i].ID < result[j].ID })
	return result
}

// ── Topic routing ─────────────────────────────────────────────────────────────

// SetTopicRoute wires topic to target component ID.
func (r *Registry) SetTopicRoute(topic, target string) {
	r.mu.Lock()
	defer r.mu.Unlock()
	r.topicRoutes[topic] = target
}

// SetTopicRoutes bulk-sets multiple topic routes under a single lock.
func (r *Registry) SetTopicRoutes(routes map[string]string) {
	r.mu.Lock()
	defer r.mu.Unlock()
	for topic, target := range routes {
		r.topicRoutes[topic] = target
	}
}

// DeleteTopicRoute removes a topic route.
// Returns the previous target and true if the route existed.
func (r *Registry) DeleteTopicRoute(topic string) (string, bool) {
	r.mu.Lock()
	defer r.mu.Unlock()
	prev, ok := r.topicRoutes[topic]
	if ok {
		delete(r.topicRoutes, topic)
	}
	return prev, ok
}

// TopicRoutes returns a copy of all topic → target mappings.
func (r *Registry) TopicRoutes() map[string]string {
	r.mu.RLock()
	defer r.mu.RUnlock()

	result := make(map[string]string, len(r.topicRoutes))
	for topic, target := range r.topicRoutes {
		result[topic] = target
	}
	return result
}

// TopicRoutesForTarget returns all topics that are routed to target.
func (r *Registry) TopicRoutesForTarget(target string) []string {
	r.mu.RLock()
	defer r.mu.RUnlock()

	var topics []string
	for topic, t := range r.topicRoutes {
		if t == target {
			topics = append(topics, topic)
		}
	}
	sort.Strings(topics)
	return topics
}

// ResolveTarget returns the mesh component ID that should receive topic.
// explicitTarget takes precedence; falls back to the configured route; returns
// "" if neither is set.
func (r *Registry) ResolveTarget(topic, explicitTarget string) string {
	if explicitTarget != "" {
		return explicitTarget
	}
	r.mu.RLock()
	defer r.mu.RUnlock()
	return r.topicRoutes[topic]
}

// ── Message routing ───────────────────────────────────────────────────────────

// Send records a routed message and returns it.
// Status is set automatically:
//   "delivered"              — target is registered and active
//   "delivered_degraded"     — target is registered but not active
//   "target_not_registered"  — target ID unknown in registry
//   "unrouted"               — target is empty string
func (r *Registry) Send(source, target, topic, payload string, metadata map[string]string) RoutedMessage {
	r.mu.Lock()
	defer r.mu.Unlock()

	status := r.resolveDeliveryStatus(target)
	msg := RoutedMessage{
		ID:        fmt.Sprintf("msg-%d", atomic.AddUint64(&r.sequence, 1)),
		Source:    defaultIfEmpty(source, "gateway"),
		Target:    target,
		Topic:     topic,
		Payload:   payload,
		Status:    status,
		Metadata:  cloneMap(metadata),
		Timestamp: time.Now().UTC().Format(time.RFC3339Nano),
	}
	r.appendMessage(msg)
	r.deliveryCounts[status]++
	return msg
}

// SendBatch sends multiple messages atomically (single lock acquisition).
func (r *Registry) SendBatch(requests []SendRequest) []RoutedMessage {
	if len(requests) == 0 {
		return nil
	}
	r.mu.Lock()
	defer r.mu.Unlock()

	results := make([]RoutedMessage, len(requests))
	for i, req := range requests {
		status := r.resolveDeliveryStatus(req.Target)
		msg := RoutedMessage{
			ID:        fmt.Sprintf("msg-%d", atomic.AddUint64(&r.sequence, 1)),
			Source:    defaultIfEmpty(req.Source, "gateway"),
			Target:    req.Target,
			Topic:     req.Topic,
			Payload:   req.Payload,
			Status:    status,
			Metadata:  cloneMap(req.Metadata),
			Timestamp: time.Now().UTC().Format(time.RFC3339Nano),
		}
		r.appendMessage(msg)
		r.deliveryCounts[status]++
		results[i] = msg
	}
	return results
}

// resolveDeliveryStatus computes the status string for a Send operation.
// Must be called with r.mu held (at least read).
func (r *Registry) resolveDeliveryStatus(target string) string {
	if target == "" {
		return "unrouted"
	}
	c, ok := r.components[target]
	if !ok {
		return "target_not_registered"
	}
	if c.Status != "active" {
		return "delivered_degraded"
	}
	return "delivered"
}

// appendMessage appends msg to r.messages, trimming the oldest if over capacity.
// Must be called with r.mu held for writing.
func (r *Registry) appendMessage(msg RoutedMessage) {
	r.messages = append(r.messages, msg)
	if len(r.messages) > r.maxMessages {
		start := len(r.messages) - r.maxMessages
		r.messages = append([]RoutedMessage(nil), r.messages[start:]...)
	}
}

// ── Message history ───────────────────────────────────────────────────────────

// MessageHistory returns up to limit recent messages (newest first if limit>0).
// Pass limit ≤ 0 for all.
func (r *Registry) MessageHistory(limit int) []RoutedMessage {
	r.mu.RLock()
	defer r.mu.RUnlock()
	return r.sliceMessages(limit, func(RoutedMessage) bool { return true })
}

// MessageHistoryByTopic returns up to limit messages for a specific topic.
func (r *Registry) MessageHistoryByTopic(topic string, limit int) []RoutedMessage {
	r.mu.RLock()
	defer r.mu.RUnlock()
	return r.sliceMessages(limit, func(m RoutedMessage) bool { return m.Topic == topic })
}

// MessageHistoryBySource returns up to limit messages from a specific source.
func (r *Registry) MessageHistoryBySource(source string, limit int) []RoutedMessage {
	r.mu.RLock()
	defer r.mu.RUnlock()
	return r.sliceMessages(limit, func(m RoutedMessage) bool { return m.Source == source })
}

// MessageHistoryByStatus returns up to limit messages with a specific delivery status.
func (r *Registry) MessageHistoryByStatus(status string, limit int) []RoutedMessage {
	r.mu.RLock()
	defer r.mu.RUnlock()
	return r.sliceMessages(limit, func(m RoutedMessage) bool { return m.Status == status })
}

// sliceMessages applies a predicate and returns the newest-first slice.
// Must be called with r.mu held for reading.
func (r *Registry) sliceMessages(limit int, pred func(RoutedMessage) bool) []RoutedMessage {
	if limit <= 0 {
		limit = len(r.messages)
	}
	buf := make([]RoutedMessage, 0, min(limit, len(r.messages)))
	for i := len(r.messages) - 1; i >= 0 && len(buf) < limit; i-- {
		if pred(r.messages[i]) {
			buf = append(buf, r.messages[i])
		}
	}
	// Reverse to chronological order
	for i, j := 0, len(buf)-1; i < j; i, j = i+1, j-1 {
		buf[i], buf[j] = buf[j], buf[i]
	}
	return buf
}

// MessageStats returns per-topic send counts and per-status delivery counts.
func (r *Registry) MessageStats() map[string]interface{} {
	r.mu.RLock()
	defer r.mu.RUnlock()

	topicCounts := map[string]int{}
	for _, m := range r.messages {
		topicCounts[m.Topic]++
	}

	deliveryCopy := make(map[string]uint64, len(r.deliveryCounts))
	for k, v := range r.deliveryCounts {
		deliveryCopy[k] = v
	}

	return map[string]interface{}{
		"topic_counts":    topicCounts,
		"delivery_counts": deliveryCopy,
		"total":           len(r.messages),
	}
}

// ── Health tracking ───────────────────────────────────────────────────────────

// RecordHealth appends a health observation for a component.
func (r *Registry) RecordHealth(id string, healthy bool, note string) HealthRecord {
	rec := HealthRecord{
		ComponentID: id,
		Healthy:     healthy,
		Note:        note,
		Timestamp:   time.Now().UTC().Format(time.RFC3339Nano),
	}
	r.healthMu.Lock()
	defer r.healthMu.Unlock()

	r.healthHistory[id] = append(r.healthHistory[id], rec)
	if len(r.healthHistory[id]) > r.maxHealthPer {
		start := len(r.healthHistory[id]) - r.maxHealthPer
		r.healthHistory[id] = append([]HealthRecord(nil), r.healthHistory[id][start:]...)
	}
	return rec
}

// ComponentHealth returns the most recent health record for a component,
// and false if no records exist.
func (r *Registry) ComponentHealth(id string) (HealthRecord, bool) {
	r.healthMu.RLock()
	defer r.healthMu.RUnlock()

	recs := r.healthHistory[id]
	if len(recs) == 0 {
		return HealthRecord{}, false
	}
	return recs[len(recs)-1], true
}

// HealthHistory returns up to limit health records for a component (newest first).
func (r *Registry) HealthHistory(id string, limit int) []HealthRecord {
	r.healthMu.RLock()
	defer r.healthMu.RUnlock()

	recs := r.healthHistory[id]
	if limit <= 0 || limit >= len(recs) {
		result := make([]HealthRecord, len(recs))
		copy(result, recs)
		reverseHealth(result)
		return result
	}
	result := make([]HealthRecord, limit)
	copy(result, recs[len(recs)-limit:])
	reverseHealth(result)
	return result
}

// ── Metrics ───────────────────────────────────────────────────────────────────

// Metrics returns a snapshot of registry activity.
func (r *Registry) Metrics() MeshMetrics {
	r.mu.RLock()
	defer r.mu.RUnlock()

	statusCounts := map[string]int{}
	kindCounts := map[string]int{}
	for _, c := range r.components {
		statusCounts[c.Status]++
		kindCounts[c.Kind]++
	}
	deliveryCopy := make(map[string]uint64, len(r.deliveryCounts))
	for k, v := range r.deliveryCounts {
		deliveryCopy[k] = v
	}

	return MeshMetrics{
		ComponentCount:   len(r.components),
		TopicRouteCount:  len(r.topicRoutes),
		TotalMessages:    len(r.messages),
		StatusCounts:     statusCounts,
		KindCounts:       kindCounts,
		DeliveryStatuses: deliveryCopy,
	}
}

// ── Internal helpers ──────────────────────────────────────────────────────────

func cloneMap(input map[string]string) map[string]string {
	if len(input) == 0 {
		return map[string]string{}
	}
	output := make(map[string]string, len(input))
	for k, v := range input {
		output[k] = v
	}
	return output
}

func defaultIfEmpty(value, fallback string) string {
	if value == "" {
		return fallback
	}
	return value
}

func reverseHealth(s []HealthRecord) {
	for i, j := 0, len(s)-1; i < j; i, j = i+1, j-1 {
		s[i], s[j] = s[j], s[i]
	}
}

func min(a, b int) int {
	if a < b {
		return a
	}
	return b
}
