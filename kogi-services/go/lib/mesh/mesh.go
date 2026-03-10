package mesh

import (
	"fmt"
	"sort"
	"sync"
	"sync/atomic"
	"time"
)

type Component struct {
	ID             string            `json:"id"`
	Kind           string            `json:"kind"`
	Endpoint       string            `json:"endpoint"`
	HealthPath     string            `json:"health_path,omitempty"`
	NetworkManager string            `json:"network_manager"`
	Status         string            `json:"status"`
	Metadata       map[string]string `json:"metadata,omitempty"`
	RegisteredAt   string            `json:"registered_at"`
}

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

type Registry struct {
	mu          sync.RWMutex
	components  map[string]Component
	topicRoutes map[string]string
	messages    []RoutedMessage
	maxMessages int
	sequence    uint64
}

func NewRegistry() *Registry {
	return &Registry{
		components:  map[string]Component{},
		topicRoutes: map[string]string{},
		messages:    []RoutedMessage{},
		maxMessages: 10000,
	}
}

func (r *Registry) Register(component Component) Component {
	r.mu.Lock()
	defer r.mu.Unlock()

	component.ID = defaultIfEmpty(component.ID, fmt.Sprintf("component-%d", atomic.AddUint64(&r.sequence, 1)))
	component.Kind = defaultIfEmpty(component.Kind, "service")
	component.Endpoint = defaultIfEmpty(component.Endpoint, "local://undefined")
	component.NetworkManager = defaultIfEmpty(component.NetworkManager, "kogi-go-network")
	component.Status = defaultIfEmpty(component.Status, "active")
	component.RegisteredAt = time.Now().UTC().Format(time.RFC3339Nano)
	if component.Metadata == nil {
		component.Metadata = map[string]string{}
	}

	r.components[component.ID] = component
	return component
}

func (r *Registry) Components() []Component {
	r.mu.RLock()
	defer r.mu.RUnlock()

	result := make([]Component, 0, len(r.components))
	for _, component := range r.components {
		result = append(result, component)
	}

	sort.Slice(result, func(i, j int) bool {
		return result[i].ID < result[j].ID
	})
	return result
}

func (r *Registry) SetTopicRoute(topic, target string) {
	r.mu.Lock()
	defer r.mu.Unlock()
	r.topicRoutes[topic] = target
}

func (r *Registry) TopicRoutes() map[string]string {
	r.mu.RLock()
	defer r.mu.RUnlock()

	copy := make(map[string]string, len(r.topicRoutes))
	for topic, target := range r.topicRoutes {
		copy[topic] = target
	}
	return copy
}

func (r *Registry) ResolveTarget(topic, explicitTarget string) string {
	if explicitTarget != "" {
		return explicitTarget
	}

	r.mu.RLock()
	defer r.mu.RUnlock()
	if target, ok := r.topicRoutes[topic]; ok {
		return target
	}
	return ""
}

func (r *Registry) Send(source, target, topic, payload string, metadata map[string]string) RoutedMessage {
	r.mu.Lock()
	defer r.mu.Unlock()

	status := "queued"
	if target == "" {
		status = "unrouted"
	} else if _, exists := r.components[target]; !exists {
		status = "target_not_registered"
	} else {
		status = "delivered"
	}

	message := RoutedMessage{
		ID:        fmt.Sprintf("msg-%d", atomic.AddUint64(&r.sequence, 1)),
		Source:    defaultIfEmpty(source, "gateway"),
		Target:    target,
		Topic:     topic,
		Payload:   payload,
		Status:    status,
		Metadata:  cloneMap(metadata),
		Timestamp: time.Now().UTC().Format(time.RFC3339Nano),
	}

	r.messages = append(r.messages, message)
	if len(r.messages) > r.maxMessages {
		start := len(r.messages) - r.maxMessages
		r.messages = append([]RoutedMessage(nil), r.messages[start:]...)
	}

	return message
}

func (r *Registry) MessageHistory(limit int) []RoutedMessage {
	r.mu.RLock()
	defer r.mu.RUnlock()

	if limit <= 0 {
		limit = len(r.messages)
	}

	buffer := make([]RoutedMessage, 0, min(limit, len(r.messages)))
	for i := len(r.messages) - 1; i >= 0 && len(buffer) < limit; i-- {
		buffer = append(buffer, r.messages[i])
	}

	for i, j := 0, len(buffer)-1; i < j; i, j = i+1, j-1 {
		buffer[i], buffer[j] = buffer[j], buffer[i]
	}
	return buffer
}

func cloneMap(input map[string]string) map[string]string {
	if len(input) == 0 {
		return map[string]string{}
	}

	output := make(map[string]string, len(input))
	for key, value := range input {
		output[key] = value
	}
	return output
}

func defaultIfEmpty(value, fallback string) string {
	if value == "" {
		return fallback
	}
	return value
}

func min(a, b int) int {
	if a < b {
		return a
	}
	return b
}
