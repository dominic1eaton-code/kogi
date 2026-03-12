package main

import (
	"encoding/json"
	"fmt"
	"log"
	"net/http"
	"os"
	"strconv"
	"strings"
	"time"

	"kogi.network/lib/eventbus"
	"kogi.network/lib/mesh"
	"kogi.network/lib/ops"
)

type gatewayInfo struct {
	Name           string            `json:"name"`
	Version        string            `json:"version"`
	Services       []string          `json:"services"`
	ComponentCount int               `json:"component_count"`
	TopicCounts    map[string]uint64 `json:"topic_counts"`
	BusMetrics     interface{}       `json:"bus_metrics"`
	MeshMetrics    interface{}       `json:"mesh_metrics"`
	Timestamp      string            `json:"timestamp"`
}

type publishRequest struct {
	Topic    string            `json:"topic"`
	Payload  string            `json:"payload"`
	Source   string            `json:"source"`
	Target   string            `json:"target"`
	Metadata map[string]string `json:"metadata"`
}

type registerComponentRequest struct {
	ID             string            `json:"id"`
	Kind           string            `json:"kind"`
	Endpoint       string            `json:"endpoint"`
	HealthPath     string            `json:"health_path"`
	NetworkManager string            `json:"network_manager"`
	Status         string            `json:"status"`
	Metadata       map[string]string `json:"metadata"`
}

type networkSendRequest struct {
	Source   string            `json:"source"`
	Target   string            `json:"target"`
	Topic    string            `json:"topic"`
	Payload  string            `json:"payload"`
	Metadata map[string]string `json:"metadata"`
}

// subscribeRequest registers a named consumer for a topic.
// The handler field is not used server-side; consumers poll /pubsub/history.
type subscribeRequest struct {
	Topic    string `json:"topic"`
	Consumer string `json:"consumer"`
}

// subscribePrefixRequest registers a consumer for all topics starting with a prefix.
type subscribePrefixRequest struct {
	Prefix   string `json:"prefix"`
	Consumer string `json:"consumer"`
}

// unsubscribeRequest removes a subscription by its ID.
type unsubscribeRequest struct {
	SubscriptionID string `json:"subscription_id"`
}

// routeTopicRequest wires a topic to a specific mesh component ID.
type routeTopicRequest struct {
	Topic  string `json:"topic"`
	Target string `json:"target"`
}

// batchRouteRequest sets multiple topic routes in one call.
type batchRouteRequest struct {
	Routes map[string]string `json:"routes"`
}

// updateStatusRequest changes a component's status.
type updateStatusRequest struct {
	ID     string `json:"id"`
	Status string `json:"status"`
}

// recordHealthRequest records a health observation for a component.
type recordHealthRequest struct {
	ID      string `json:"id"`
	Healthy bool   `json:"healthy"`
	Note    string `json:"note"`
}

var gatewayRuntime *ops.Runtime

func main() {
	gatewayRuntime = ops.Init("gateway")
	gatewayRuntime.State("init")
	bus := eventbus.NewWithHistory(25000)
	registry := mesh.NewRegistry()
	seedRegistry(registry, bus)

	gatewayRuntime.State("configure")
	gatewayRuntime.WatchSignals()

	if gatewayRuntime.DebugEnabled() {
		bus.SubscribeAll("gateway.debug", func(e eventbus.Event) {
			log.Printf("[gateway/debug] topic=%s source=%s target=%s", e.Topic, e.Source, e.Target)
		})
	}

	// Logger subscription always active
	bus.SubscribeAll("gateway.logger", func(e eventbus.Event) {
		log.Printf("[gateway] pubsub topic=%s source=%s target=%s payload=%s",
			e.Topic, e.Source, e.Target, e.Payload)
	})

	// Auto-mirror portfolio and office.portfolio events to engine ingest using
	// prefix subscriptions — one handler per namespace, no manual enumeration.
	bus.SubscribePrefix("portfolio.", "gateway.engine-mirror", func(e eventbus.Event) {
		mirrorToEngine(bus, registry, e)
	})
	bus.SubscribePrefix("office.portfolio.", "gateway.engine-mirror", func(e eventbus.Event) {
		mirrorToEngine(bus, registry, e)
	})

	// Dead-letter logger — warn whenever an event has no subscribers.
	bus.SubscribeAll("gateway.dead-letter-watch", func(e eventbus.Event) {
		// We check after the fact via DeadLetters; this is a no-op watcher
		// kept here to ensure the wildcard bucket is always non-empty so the
		// dead-letter condition is never triggered by this watcher itself.
		_ = e
	})

	mux := http.NewServeMux()

	// ── Health ────────────────────────────────────────────────────────────
	mux.HandleFunc("/health", func(w http.ResponseWriter, r *http.Request) {
		writeJSON(w, http.StatusOK, map[string]interface{}{
			"status":    "ok",
			"service":   "kogi-go-gateway",
			"component": "kogi.network.gateway",
		})
	})

	// ── Gateway info ──────────────────────────────────────────────────────
	mux.HandleFunc("/api/v1/gateway/info", func(w http.ResponseWriter, r *http.Request) {
		writeJSON(w, http.StatusOK, gatewayInfo{
			Name:    "kogi-go-gateway",
			Version: "0.3.0",
			Services: []string{
				"auth", "portfolio", "exchange", "ims", "office",
				"bank", "marketplace", "studio", "community",
				"developer", "profile", "providers", "organizations", "engine", "database",
			},
			ComponentCount: len(registry.Components()),
			TopicCounts:    bus.TopicStats(),
			BusMetrics:     bus.Metrics(),
			MeshMetrics:    registry.Metrics(),
			Timestamp:      time.Now().UTC().Format(time.RFC3339),
		})
	})

	// ── Routes overview ───────────────────────────────────────────────────
	mux.HandleFunc("/api/v1/gateway/routes", func(w http.ResponseWriter, r *http.Request) {
		writeJSON(w, http.StatusOK, map[string]interface{}{
			"gateway": "kogi-go-gateway",
			"routes": map[string]string{
				"auth":          "/services/auth",
				"portfolio":     "/services/portfolio",
				"exchange":      "/services/exchange",
				"ims":           "/services/ims",
				"office":        "/services/office",
				"bank":          "/services/bank",
				"marketplace":   "/services/marketplace",
				"studio":        "/services/studio",
				"community":     "/services/community",
				"developer":     "/services/developer",
				"profile":       "/services/profile",
				"providers":     "/services/providers",
				"organizations": "/services/organizations",
				"engine":        "/services/engine",
				"database":      "/services/database",
			},
			"topic_routes": registry.TopicRoutes(),
			"discovery":    "kogi gateway manages pub/sub, mesh routing, portfolio system events, and engine ingest",
		})
	})

	// ── Component registry ────────────────────────────────────────────────

	mux.HandleFunc("/api/v1/gateway/components/register", func(w http.ResponseWriter, r *http.Request) {
		if r.Method != http.MethodPost {
			writeJSON(w, http.StatusMethodNotAllowed, map[string]string{"error": "method_not_allowed"})
			return
		}
		var req registerComponentRequest
		if err := decodeJSON(r, &req); err != nil {
			writeJSON(w, http.StatusBadRequest, map[string]string{"error": err.Error()})
			return
		}
		if gatewayRuntime != nil {
			gatewayRuntime.Debugf("component register id=%s kind=%s endpoint=%s", req.ID, req.Kind, req.Endpoint)
		}
		registered := registry.Register(mesh.Component{
			ID:             req.ID,
			Kind:           req.Kind,
			Endpoint:       req.Endpoint,
			HealthPath:     req.HealthPath,
			NetworkManager: req.NetworkManager,
			Status:         req.Status,
			Metadata:       req.Metadata,
		})
		// Auto-register any topics declared in metadata.publishes
		if req.Metadata != nil {
			if pubs, ok := req.Metadata["publishes"]; ok {
				for _, topic := range strings.Split(pubs, ",") {
					topic = strings.TrimSpace(topic)
					if topic != "" {
						registry.SetTopicRoute(topic, req.ID)
					}
				}
			}
		}
		bus.PublishWith("gateway.component.registered",
			fmt.Sprintf(`{"id":%q,"kind":%q}`, req.ID, req.Kind),
			eventbus.PublishOptions{Source: "kogi.network.gateway"})
		if gatewayRuntime != nil {
			gatewayRuntime.Publish("gateway.component.registered", "kogi.network.gateway", req.ID, fmt.Sprintf(`{"id":%q,"kind":%q}`, req.ID, req.Kind))
		}
		writeJSON(w, http.StatusCreated, map[string]interface{}{
			"registered": registered,
			"count":      len(registry.Components()),
		})
	})

	mux.HandleFunc("/api/v1/gateway/components", func(w http.ResponseWriter, r *http.Request) {
		writeJSON(w, http.StatusOK, map[string]interface{}{
			"components": registry.Components(),
		})
	})

	// Filter by kind: GET /api/v1/gateway/components/kind/{kind}
	mux.HandleFunc("/api/v1/gateway/components/kind/", func(w http.ResponseWriter, r *http.Request) {
		kind := strings.TrimPrefix(r.URL.Path, "/api/v1/gateway/components/kind/")
		writeJSON(w, http.StatusOK, map[string]interface{}{
			"kind":       kind,
			"components": registry.ComponentsByKind(kind),
		})
	})

	// Filter by status: GET /api/v1/gateway/components/status/{status}
	mux.HandleFunc("/api/v1/gateway/components/status/", func(w http.ResponseWriter, r *http.Request) {
		status := strings.TrimPrefix(r.URL.Path, "/api/v1/gateway/components/status/")
		writeJSON(w, http.StatusOK, map[string]interface{}{
			"status":     status,
			"components": registry.ComponentsByStatus(status),
		})
	})

	// Single component lookup: GET /api/v1/gateway/components/id/{id}
	mux.HandleFunc("/api/v1/gateway/components/id/", func(w http.ResponseWriter, r *http.Request) {
		id := strings.TrimPrefix(r.URL.Path, "/api/v1/gateway/components/id/")
		c, ok := registry.ComponentByID(id)
		if !ok {
			writeJSON(w, http.StatusNotFound, map[string]string{"error": "component not found"})
			return
		}
		health, _ := registry.ComponentHealth(id)
		writeJSON(w, http.StatusOK, map[string]interface{}{
			"component": c,
			"health":    health,
			"topics":    registry.TopicRoutesForTarget(id),
		})
	})

	// Unregister: DELETE /api/v1/gateway/components/unregister
	mux.HandleFunc("/api/v1/gateway/components/unregister", func(w http.ResponseWriter, r *http.Request) {
		if r.Method != http.MethodDelete && r.Method != http.MethodPost {
			writeJSON(w, http.StatusMethodNotAllowed, map[string]string{"error": "method_not_allowed"})
			return
		}
		var body struct {
			ID string `json:"id"`
		}
		if err := decodeJSON(r, &body); err != nil {
			writeJSON(w, http.StatusBadRequest, map[string]string{"error": err.Error()})
			return
		}
		if gatewayRuntime != nil {
			gatewayRuntime.Debugf("component unregister id=%s", body.ID)
		}
		removed, ok := registry.Unregister(body.ID)
		if !ok {
			writeJSON(w, http.StatusNotFound, map[string]string{"error": "component not found"})
			return
		}
		bus.PublishWith("gateway.component.unregistered",
			fmt.Sprintf(`{"id":%q}`, body.ID),
			eventbus.PublishOptions{Source: "kogi.network.gateway"})
		if gatewayRuntime != nil {
			gatewayRuntime.Publish("gateway.component.unregistered", "kogi.network.gateway", body.ID, fmt.Sprintf(`{"id":%q}`, body.ID))
		}
		writeJSON(w, http.StatusOK, map[string]interface{}{
			"unregistered": removed,
			"count":        len(registry.Components()),
		})
	})

	// Update status: POST /api/v1/gateway/components/status
	mux.HandleFunc("/api/v1/gateway/components/status", func(w http.ResponseWriter, r *http.Request) {
		if r.Method != http.MethodPost {
			writeJSON(w, http.StatusMethodNotAllowed, map[string]string{"error": "method_not_allowed"})
			return
		}
		var req updateStatusRequest
		if err := decodeJSON(r, &req); err != nil {
			writeJSON(w, http.StatusBadRequest, map[string]string{"error": err.Error()})
			return
		}
		if gatewayRuntime != nil {
			gatewayRuntime.Debugf("component status update id=%s status=%s", req.ID, req.Status)
		}
		updated, ok := registry.UpdateStatus(req.ID, req.Status)
		if !ok {
			writeJSON(w, http.StatusNotFound, map[string]string{"error": "component not found"})
			return
		}
		bus.PublishWith("gateway.component.status.updated",
			fmt.Sprintf(`{"id":%q,"status":%q}`, req.ID, req.Status),
			eventbus.PublishOptions{Source: "kogi.network.gateway"})
		if gatewayRuntime != nil {
			gatewayRuntime.Publish("gateway.component.status.updated", "kogi.network.gateway", req.ID, fmt.Sprintf(`{"id":%q,"status":%q}`, req.ID, req.Status))
		}
		writeJSON(w, http.StatusOK, map[string]interface{}{"component": updated})
	})

	// Record health: POST /api/v1/gateway/components/health
	mux.HandleFunc("/api/v1/gateway/components/health", func(w http.ResponseWriter, r *http.Request) {
		if r.Method != http.MethodPost {
			writeJSON(w, http.StatusMethodNotAllowed, map[string]string{"error": "method_not_allowed"})
			return
		}
		var req recordHealthRequest
		if err := decodeJSON(r, &req); err != nil {
			writeJSON(w, http.StatusBadRequest, map[string]string{"error": err.Error()})
			return
		}
		if gatewayRuntime != nil {
			gatewayRuntime.Debugf("component health id=%s healthy=%v note=%s", req.ID, req.Healthy, req.Note)
		}
		rec := registry.RecordHealth(req.ID, req.Healthy, req.Note)
		if !req.Healthy {
			_, _ = registry.UpdateStatus(req.ID, "degraded")
		} else {
			_, _ = registry.UpdateStatus(req.ID, "active")
		}
		writeJSON(w, http.StatusCreated, map[string]interface{}{"health_record": rec})
	})

	// Health history: GET /api/v1/gateway/components/health/{id}
	mux.HandleFunc("/api/v1/gateway/components/health/", func(w http.ResponseWriter, r *http.Request) {
		id := strings.TrimPrefix(r.URL.Path, "/api/v1/gateway/components/health/")
		limit := queryInt(r, "limit", 20)
		writeJSON(w, http.StatusOK, map[string]interface{}{
			"component_id":   id,
			"health_history": registry.HealthHistory(id, limit),
		})
	})

	// Mesh metrics: GET /api/v1/gateway/components/metrics
	mux.HandleFunc("/api/v1/gateway/components/metrics", func(w http.ResponseWriter, r *http.Request) {
		writeJSON(w, http.StatusOK, registry.Metrics())
	})

	// ── Pub/sub ───────────────────────────────────────────────────────────

	mux.HandleFunc("/api/v1/gateway/pubsub/publish", func(w http.ResponseWriter, r *http.Request) {
		if r.Method != http.MethodPost {
			writeJSON(w, http.StatusMethodNotAllowed, map[string]string{"error": "method_not_allowed"})
			return
		}
		var req publishRequest
		if err := decodeJSON(r, &req); err != nil {
			writeJSON(w, http.StatusBadRequest, map[string]string{"error": err.Error()})
			return
		}
		if req.Topic == "" {
			writeJSON(w, http.StatusBadRequest, map[string]string{"error": "topic is required"})
			return
		}
		source := defaultIfEmpty(req.Source, "gateway")
		target := registry.ResolveTarget(req.Topic, req.Target)
		if gatewayRuntime != nil {
			gatewayRuntime.Message("receive", req.Topic, source, target, req.Payload)
		}
		event := bus.PublishWith(req.Topic, req.Payload, eventbus.PublishOptions{
			Source:   source,
			Target:   target,
			Metadata: req.Metadata,
		})
		networkMessage := registry.Send(source, target, req.Topic, req.Payload, req.Metadata)
		if gatewayRuntime != nil {
			gatewayRuntime.Message("send", req.Topic, source, target, req.Payload)
			gatewayRuntime.Debugf("network status=%s message_id=%s", networkMessage.Status, networkMessage.ID)
		}
		engineMessage := mirrorToEngine(bus, registry, event)
		writeJSON(w, http.StatusAccepted, map[string]interface{}{
			"event":          event,
			"route_target":   target,
			"network_result": networkMessage,
			"engine_result":  engineMessage,
		})
	})

	// Subscribe: registers a named consumer on the bus (exact topic).
	// Consumers receive events by polling /pubsub/history?topic=…
	mux.HandleFunc("/api/v1/gateway/pubsub/subscribe", func(w http.ResponseWriter, r *http.Request) {
		if r.Method != http.MethodPost {
			writeJSON(w, http.StatusMethodNotAllowed, map[string]string{"error": "method_not_allowed"})
			return
		}
		var req subscribeRequest
		if err := decodeJSON(r, &req); err != nil {
			writeJSON(w, http.StatusBadRequest, map[string]string{"error": err.Error()})
			return
		}
		if req.Topic == "" {
			writeJSON(w, http.StatusBadRequest, map[string]string{"error": "topic is required"})
			return
		}
		consumer := defaultIfEmpty(req.Consumer, "anonymous")
		if gatewayRuntime != nil {
			gatewayRuntime.Subscribe("exact", req.Topic, consumer)
		}
		subID := bus.SubscribeNamed(req.Topic, consumer, func(e eventbus.Event) {
			// Events delivered via poll at /pubsub/history or /pubsub/receive
		})
		writeJSON(w, http.StatusCreated, map[string]interface{}{
			"subscription_id": subID,
			"topic":           req.Topic,
			"consumer":        consumer,
			"poll_url":        fmt.Sprintf("/api/v1/gateway/pubsub/history?topic=%s", req.Topic),
		})
	})

	// Subscribe prefix: registers a consumer for all topics starting with a prefix.
	mux.HandleFunc("/api/v1/gateway/pubsub/subscribe/prefix", func(w http.ResponseWriter, r *http.Request) {
		if r.Method != http.MethodPost {
			writeJSON(w, http.StatusMethodNotAllowed, map[string]string{"error": "method_not_allowed"})
			return
		}
		var req subscribePrefixRequest
		if err := decodeJSON(r, &req); err != nil {
			writeJSON(w, http.StatusBadRequest, map[string]string{"error": err.Error()})
			return
		}
		if req.Prefix == "" {
			writeJSON(w, http.StatusBadRequest, map[string]string{"error": "prefix is required"})
			return
		}
		consumer := defaultIfEmpty(req.Consumer, "anonymous")
		if gatewayRuntime != nil {
			gatewayRuntime.Subscribe("prefix", req.Prefix, consumer)
		}
		subID := bus.SubscribePrefix(req.Prefix, consumer, func(e eventbus.Event) {
			// Events delivered via poll at /pubsub/history/prefix
		})
		writeJSON(w, http.StatusCreated, map[string]interface{}{
			"subscription_id": subID,
			"prefix":          req.Prefix,
			"consumer":        consumer,
			"poll_url":        fmt.Sprintf("/api/v1/gateway/pubsub/history/prefix?prefix=%s", req.Prefix),
		})
	})

	// Unsubscribe: removes a subscription by its ID.
	mux.HandleFunc("/api/v1/gateway/pubsub/unsubscribe", func(w http.ResponseWriter, r *http.Request) {
		if r.Method != http.MethodPost && r.Method != http.MethodDelete {
			writeJSON(w, http.StatusMethodNotAllowed, map[string]string{"error": "method_not_allowed"})
			return
		}
		var req unsubscribeRequest
		if err := decodeJSON(r, &req); err != nil {
			writeJSON(w, http.StatusBadRequest, map[string]string{"error": err.Error()})
			return
		}
		if req.SubscriptionID == "" {
			writeJSON(w, http.StatusBadRequest, map[string]string{"error": "subscription_id is required"})
			return
		}
		if gatewayRuntime != nil {
			gatewayRuntime.Debugf("unsubscribe id=%s", req.SubscriptionID)
		}
		removed := bus.Unsubscribe(req.SubscriptionID)
		writeJSON(w, http.StatusOK, map[string]interface{}{
			"subscription_id": req.SubscriptionID,
			"removed":         removed,
		})
	})

	// Receive: returns recent events for a topic (poll-based delivery).
	mux.HandleFunc("/api/v1/gateway/pubsub/receive", func(w http.ResponseWriter, r *http.Request) {
		topic := r.URL.Query().Get("topic")
		limit := queryInt(r, "limit", 20)
		if topic == "" {
			writeJSON(w, http.StatusBadRequest, map[string]string{"error": "topic is required"})
			return
		}
		writeJSON(w, http.StatusOK, map[string]interface{}{
			"topic":  topic,
			"events": bus.History(limit, topic),
		})
	})

	mux.HandleFunc("/api/v1/gateway/pubsub/history", func(w http.ResponseWriter, r *http.Request) {
		limit := queryInt(r, "limit", 100)
		topic := r.URL.Query().Get("topic")
		writeJSON(w, http.StatusOK, map[string]interface{}{
			"events": bus.History(limit, topic),
		})
	})

	// History filtered by topic prefix: GET /pubsub/history/prefix?prefix=portfolio.&limit=50
	mux.HandleFunc("/api/v1/gateway/pubsub/history/prefix", func(w http.ResponseWriter, r *http.Request) {
		prefix := r.URL.Query().Get("prefix")
		limit := queryInt(r, "limit", 100)
		if prefix == "" {
			writeJSON(w, http.StatusBadRequest, map[string]string{"error": "prefix is required"})
			return
		}
		writeJSON(w, http.StatusOK, map[string]interface{}{
			"prefix": prefix,
			"events": bus.HistoryPrefix(prefix, limit),
		})
	})

	// History filtered by source component: GET /pubsub/history/source?source=kogi.network.portfolio&limit=50
	mux.HandleFunc("/api/v1/gateway/pubsub/history/source", func(w http.ResponseWriter, r *http.Request) {
		source := r.URL.Query().Get("source")
		limit := queryInt(r, "limit", 100)
		if source == "" {
			writeJSON(w, http.StatusBadRequest, map[string]string{"error": "source is required"})
			return
		}
		writeJSON(w, http.StatusOK, map[string]interface{}{
			"source": source,
			"events": bus.HistorySource(source, limit),
		})
	})

	// Replay within a timestamp window: GET /pubsub/replay?topic=portfolio.item.created&from=…&to=…
	mux.HandleFunc("/api/v1/gateway/pubsub/replay", func(w http.ResponseWriter, r *http.Request) {
		topic := r.URL.Query().Get("topic")
		prefix := r.URL.Query().Get("prefix")
		from := r.URL.Query().Get("from")
		to := r.URL.Query().Get("to")
		var events []eventbus.Event
		if prefix != "" {
			events = bus.ReplayPrefix(prefix, from, to)
		} else {
			events = bus.Replay(topic, from, to)
		}
		writeJSON(w, http.StatusOK, map[string]interface{}{
			"topic":  topic,
			"prefix": prefix,
			"from":   from,
			"to":     to,
			"events": events,
			"count":  len(events),
		})
	})

	// Dead-letters: events published with no subscribers at time of publish.
	mux.HandleFunc("/api/v1/gateway/pubsub/dead-letters", func(w http.ResponseWriter, r *http.Request) {
		limit := queryInt(r, "limit", 50)
		writeJSON(w, http.StatusOK, map[string]interface{}{
			"dead_letters": bus.DeadLetters(limit),
			"count":        len(bus.DeadLetters(limit)),
		})
	})

	mux.HandleFunc("/api/v1/gateway/pubsub/topics", func(w http.ResponseWriter, r *http.Request) {
		writeJSON(w, http.StatusOK, map[string]interface{}{
			"topic_counts":   bus.TopicStats(),
			"subscriptions":  bus.Subscriptions(),
			"total_messages": len(bus.History(0, "")),
		})
	})

	// Bus metrics snapshot.
	mux.HandleFunc("/api/v1/gateway/pubsub/metrics", func(w http.ResponseWriter, r *http.Request) {
		writeJSON(w, http.StatusOK, bus.Metrics())
	})

	// ── Portfolio-specific pub/sub endpoints ──────────────────────────────
	//
	// Convenience endpoints for portfolio and office.portfolio event namespaces,
	// using the new HistoryPrefix and TopicRoutesForTarget mesh methods.

	mux.HandleFunc("/api/v1/gateway/portfolio/events", func(w http.ResponseWriter, r *http.Request) {
		limit := queryInt(r, "limit", 50)
		// Merge both prefixes — portfolio. and office.portfolio.
		portEvents := bus.HistoryPrefix("portfolio.", limit)
		officePortEvents := bus.HistoryPrefix("office.portfolio.", limit)
		portEvents = append(portEvents, officePortEvents...)
		// Re-sort by timestamp (both slices are already chronological)
		// Simple merge: sort by Timestamp string (RFC3339Nano sorts lexicographically)
		if len(portEvents) > limit {
			portEvents = portEvents[len(portEvents)-limit:]
		}
		writeJSON(w, http.StatusOK, map[string]interface{}{
			"portfolio_events": portEvents,
			"count":            len(portEvents),
		})
	})

	// Portfolio topic routes — what routes in and what routes from portfolio-service.
	mux.HandleFunc("/api/v1/gateway/portfolio/routes", func(w http.ResponseWriter, r *http.Request) {
		writeJSON(w, http.StatusOK, map[string]interface{}{
			"portfolio_topic_routes":      filterRoutesByPrefix(registry.TopicRoutes(), "portfolio."),
			"office_portfolio_routes":     filterRoutesByPrefix(registry.TopicRoutes(), "office.portfolio."),
			"routes_to_portfolio_service": registry.TopicRoutesForTarget("kogi.network.portfolio"),
			"routes_to_office_service":    registry.TopicRoutesForTarget("kogi.network.office"),
		})
	})

	// Portfolio network message history — mesh messages for portfolio topics.
	mux.HandleFunc("/api/v1/gateway/portfolio/messages", func(w http.ResponseWriter, r *http.Request) {
		topic := r.URL.Query().Get("topic")
		limit := queryInt(r, "limit", 50)
		var messages interface{}
		if topic != "" {
			messages = registry.MessageHistoryByTopic(topic, limit)
		} else {
			// Return portfolio-topic messages by filtering source
			messages = registry.MessageHistoryBySource("kogi.network.portfolio", limit)
		}
		writeJSON(w, http.StatusOK, map[string]interface{}{
			"messages": messages,
		})
	})

	// Network message stats — per-topic send counts and delivery status breakdown.
	mux.HandleFunc("/api/v1/gateway/network/stats", func(w http.ResponseWriter, r *http.Request) {
		writeJSON(w, http.StatusOK, registry.MessageStats())
	})

	// ── Network mesh ──────────────────────────────────────────────────────

	mux.HandleFunc("/api/v1/gateway/network/send", func(w http.ResponseWriter, r *http.Request) {
		if r.Method != http.MethodPost {
			writeJSON(w, http.StatusMethodNotAllowed, map[string]string{"error": "method_not_allowed"})
			return
		}
		var req networkSendRequest
		if err := decodeJSON(r, &req); err != nil {
			writeJSON(w, http.StatusBadRequest, map[string]string{"error": err.Error()})
			return
		}
		if req.Topic == "" {
			writeJSON(w, http.StatusBadRequest, map[string]string{"error": "topic is required"})
			return
		}
		target := registry.ResolveTarget(req.Topic, req.Target)
		if gatewayRuntime != nil {
			gatewayRuntime.Message("receive", req.Topic, defaultIfEmpty(req.Source, "gateway"), target, req.Payload)
		}
		message := registry.Send(
			defaultIfEmpty(req.Source, "gateway"),
			target,
			req.Topic,
			req.Payload,
			req.Metadata,
		)
		if gatewayRuntime != nil {
			gatewayRuntime.Message("send", req.Topic, defaultIfEmpty(req.Source, "gateway"), target, req.Payload)
			gatewayRuntime.Debugf("network status=%s message_id=%s", message.Status, message.ID)
		}
		event := bus.PublishWith(req.Topic, req.Payload, eventbus.PublishOptions{
			Source:   defaultIfEmpty(req.Source, "gateway"),
			Target:   target,
			Metadata: req.Metadata,
		})
		engineMessage := mirrorToEngine(bus, registry, event)
		writeJSON(w, http.StatusAccepted, map[string]interface{}{
			"message":       message,
			"event":         event,
			"engine_result": engineMessage,
		})
	})

	mux.HandleFunc("/api/v1/gateway/network/history", func(w http.ResponseWriter, r *http.Request) {
		limit := queryInt(r, "limit", 100)
		topic := r.URL.Query().Get("topic")
		source := r.URL.Query().Get("source")
		status := r.URL.Query().Get("status")
		switch {
		case topic != "":
			writeJSON(w, http.StatusOK, map[string]interface{}{"messages": registry.MessageHistoryByTopic(topic, limit)})
		case source != "":
			writeJSON(w, http.StatusOK, map[string]interface{}{"messages": registry.MessageHistoryBySource(source, limit)})
		case status != "":
			writeJSON(w, http.StatusOK, map[string]interface{}{"messages": registry.MessageHistoryByStatus(status, limit)})
		default:
			writeJSON(w, http.StatusOK, map[string]interface{}{"messages": registry.MessageHistory(limit)})
		}
	})

	mux.HandleFunc("/api/v1/gateway/network/routes", func(w http.ResponseWriter, r *http.Request) {
		id := r.URL.Query().Get("target")
		if id != "" {
			writeJSON(w, http.StatusOK, map[string]interface{}{
				"target":       id,
				"topic_routes": registry.TopicRoutesForTarget(id),
			})
			return
		}
		writeJSON(w, http.StatusOK, map[string]interface{}{
			"topic_routes": registry.TopicRoutes(),
		})
	})

	// Dynamic topic route registration — single
	mux.HandleFunc("/api/v1/gateway/network/routes/set", func(w http.ResponseWriter, r *http.Request) {
		if r.Method != http.MethodPost {
			writeJSON(w, http.StatusMethodNotAllowed, map[string]string{"error": "method_not_allowed"})
			return
		}
		var req routeTopicRequest
		if err := decodeJSON(r, &req); err != nil {
			writeJSON(w, http.StatusBadRequest, map[string]string{"error": err.Error()})
			return
		}
		if req.Topic == "" || req.Target == "" {
			writeJSON(w, http.StatusBadRequest, map[string]string{"error": "topic and target are required"})
			return
		}
		registry.SetTopicRoute(req.Topic, req.Target)
		writeJSON(w, http.StatusOK, map[string]interface{}{
			"topic":  req.Topic,
			"target": req.Target,
		})
	})

	// Bulk route registration — set many routes in one call
	mux.HandleFunc("/api/v1/gateway/network/routes/batch", func(w http.ResponseWriter, r *http.Request) {
		if r.Method != http.MethodPost {
			writeJSON(w, http.StatusMethodNotAllowed, map[string]string{"error": "method_not_allowed"})
			return
		}
		var req batchRouteRequest
		if err := decodeJSON(r, &req); err != nil {
			writeJSON(w, http.StatusBadRequest, map[string]string{"error": err.Error()})
			return
		}
		registry.SetTopicRoutes(req.Routes)
		writeJSON(w, http.StatusOK, map[string]interface{}{
			"set":    len(req.Routes),
			"routes": registry.TopicRoutes(),
		})
	})

	// Delete a topic route
	mux.HandleFunc("/api/v1/gateway/network/routes/delete", func(w http.ResponseWriter, r *http.Request) {
		if r.Method != http.MethodPost && r.Method != http.MethodDelete {
			writeJSON(w, http.StatusMethodNotAllowed, map[string]string{"error": "method_not_allowed"})
			return
		}
		var req routeTopicRequest
		if err := decodeJSON(r, &req); err != nil {
			writeJSON(w, http.StatusBadRequest, map[string]string{"error": err.Error()})
			return
		}
		prev, ok := registry.DeleteTopicRoute(req.Topic)
		writeJSON(w, http.StatusOK, map[string]interface{}{
			"topic":          req.Topic,
			"previous_target": prev,
			"existed":        ok,
		})
	})

	// ── Engine stream ─────────────────────────────────────────────────────
	mux.HandleFunc("/api/v1/gateway/engine/stream", func(w http.ResponseWriter, r *http.Request) {
		limit := queryInt(r, "limit", 100)
		ingest := bus.HistoryPrefix("engine.ingest", limit)
		// Also include events directly targeted at kogi.engine
		targeted := bus.HistorySource("kogi.network.gateway", limit*2)
		for _, e := range targeted {
			if e.Target == "kogi.engine" && e.Topic != "engine.ingest" {
				ingest = append(ingest, e)
				if len(ingest) >= limit*2 {
					break
				}
			}
		}
		if len(ingest) > limit {
			ingest = ingest[len(ingest)-limit:]
		}
		writeJSON(w, http.StatusOK, map[string]interface{}{
			"engine_ingest": ingest,
			"count":         len(ingest),
		})
	})

	// addr := resolveAddr("8090", "KOGI_GATEWAY_PORT")
	addr := resolveAddr("9005", "KOGI_GATEWAY_PORT")
	gatewayRuntime.State("running")
	gatewayRuntime.Status("ok", "listening="+addr)
	log.Printf("[gateway] kogi-go-gateway v0.3.0 listening on %s", addr)
	log.Fatal(http.ListenAndServe(addr, ops.WithHTTPDebug(gatewayRuntime, mux)))
}

// ── Registry seed ─────────────────────────────────────────────────────────────

func seedRegistry(registry *mesh.Registry, bus *eventbus.Bus) {
	// Core infrastructure
	registry.Register(mesh.Component{
		ID: "kogi.network.gateway", Kind: "gateway",
		Endpoint: "http://127.0.0.1:9005", NetworkManager: "kogi-go-network",
		Status: "active", Metadata: map[string]string{"owner": "kogi-host"},
	})
	registry.Register(mesh.Component{
		ID: "kogi.server", Kind: "server",
		Endpoint: "http://127.0.0.1:8080", NetworkManager: "kogi-go-network", Status: "active",
	})
	registry.Register(mesh.Component{
		ID: "kogi.engine", Kind: "engine",
		Endpoint: "local://kogi-engine", NetworkManager: "kogi-go-network", Status: "active",
	})

	// Services
	for _, svc := range []struct {
		id, port string
	}{
		{"kogi.network.auth", "9001"},
		{"kogi.network.portfolio", "9002"},
		{"kogi.network.exchange", "9004"},
		{"kogi.network.ims", "9005"},
		{"kogi.network.office", "9006"},
		{"kogi.network.bank", "9007"},
		{"kogi.network.marketplace", "9008"},
		{"kogi.network.studio", "9009"},
		{"kogi.network.community", "9010"},
		{"kogi.network.developer", "9011"},
		{"kogi.network.profile", "9012"},
		{"kogi.network.providers", "9016"},
		{"kogi.network.organizations", "9013"},
		{"kogi.network.engine", "9014"},
		{"kogi.network.database", "9015"},
	} {
		registry.Register(mesh.Component{
			ID:             svc.id,
			Kind:           "service",
			Endpoint:       fmt.Sprintf("http://127.0.0.1:%s", svc.port),
			NetworkManager: "kogi-go-network",
			Status:         "active",
		})
	}

	// ── Topic routes — all set atomically in batches ──────────────────────

	registry.SetTopicRoutes(map[string]string{
		// IMS
		"ims.identity.created": "kogi.network.ims",
		"ims.profile.updated":  "kogi.network.ims",

		// Exchange
		"exchange.order.created": "kogi.network.exchange",
		"exchange.trade.executed": "kogi.network.exchange",

		// Auth
		"auth.login.request": "kogi.network.auth",

		// Bank
		"bank.wallet.updated": "kogi.network.bank",
		"bank.ledger.posted":  "kogi.network.bank",

		// Marketplace
		"marketplace.listing.created": "kogi.network.marketplace",
		"marketplace.match.found":     "kogi.network.marketplace",

		// Studio
		"studio.idea.created":      "kogi.network.studio",
		"studio.prototype.updated": "kogi.network.studio",

		// Community
		"community.room.updated":    "kogi.network.community",
		"community.message.posted":  "kogi.network.community",

		// Developer
		"developer.sdk.published":      "kogi.network.developer",
		"developer.extension.updated":  "kogi.network.developer",

		// Profile
		"profile.settings.updated": "kogi.network.profile",
		"profile.persona.updated":  "kogi.network.profile",

		// Providers
		"provider.platform.registered": "kogi.network.providers",
		"provider.registered":          "kogi.network.providers",
		"provider.resource.added":      "kogi.network.providers",
		"provider.version.published":   "kogi.network.providers",
		"provider.metadata.updated":    "kogi.network.providers",
		"provider.data.asset.updated":  "kogi.network.providers",
		"provider.affiliate.registered": "kogi.network.providers",
		"provider.affiliate.linked":     "kogi.network.providers",

		// Organizations
		"organizations.role.updated":      "kogi.network.organizations",
		"organizations.proposal.created":  "kogi.network.organizations",

		// Engine
		"engine.control.requested": "kogi.network.engine",

		// Internal gateway events
		"gateway.component.registered":      "kogi.network.gateway",
		"gateway.component.unregistered":    "kogi.network.gateway",
		"gateway.component.status.updated":  "kogi.network.gateway",
	})

	// Portfolio system — full surface from PortfolioSystem.rs
	registry.SetTopicRoutes(map[string]string{
		"portfolio.item.created":                    "kogi.network.portfolio",
		"portfolio.component.updated":               "kogi.network.portfolio",
		"portfolio.component.removed":               "kogi.network.portfolio",
		"portfolio.graph.changed":                   "kogi.network.portfolio",
		"portfolio.graph.hierarchy.changed":         "kogi.network.portfolio",
		"portfolio.graph.dependency.changed":        "kogi.network.portfolio",
		"portfolio.graph.link.changed":              "kogi.network.portfolio",
		"portfolio.graph.member.changed":            "kogi.network.portfolio",
		"portfolio.snapshot.saved":                  "kogi.network.portfolio",
		"portfolio.checkpoint.created":              "kogi.network.portfolio",
		"portfolio.governance.policy.attached":      "kogi.network.portfolio",
		"portfolio.governance.policy.detached":      "kogi.network.portfolio",
		"portfolio.governance.approval.requested":   "kogi.network.portfolio",
		"portfolio.governance.approval.resolved":    "kogi.network.portfolio",
		"portfolio.governance.resource.allocated":   "kogi.network.portfolio",
		"portfolio.governance.resource.consumed":    "kogi.network.portfolio",
		"portfolio.model.computed":                  "kogi.network.portfolio",
		"portfolio.health.updated":                  "kogi.network.portfolio",
		"portfolio.crdt.merge.applied":              "kogi.network.portfolio",
	})

	// Office — routes for all portfolio-proxied and office-native events
	registry.SetTopicRoutes(map[string]string{
		"office.dashboard.refresh":                        "kogi.network.office",
		"office.timeline.updated":                         "kogi.network.office",
		"office.workspace.story.created":                  "kogi.network.office",
		"office.assistant.subscription.active":            "kogi.network.office",
		"office.portfolio.item.created":                   "kogi.network.office",
		"office.portfolio.component.updated":              "kogi.network.office",
		"office.portfolio.component.removed":              "kogi.network.office",
		"office.portfolio.graph.changed":                  "kogi.network.office",
		"office.portfolio.snapshot.saved":                 "kogi.network.office",
		"office.portfolio.checkpoint.created":             "kogi.network.office",
		"office.portfolio.governance.approval.requested":  "kogi.network.office",
		"office.portfolio.governance.resource.allocated":  "kogi.network.office",
		"office.portfolio.model.computed":                 "kogi.network.office",
	})

	// Database
	registry.SetTopicRoutes(map[string]string{
		"database.query.executed":    "kogi.network.database",
		"database.snapshot.created": "kogi.network.database",
		"database.checkpoint.created": "kogi.network.database",
		"database.backup.completed":  "kogi.network.database",
		"database.restore.completed": "kogi.network.database",
		"database.scale.updated":     "kogi.network.database",
		"database.optimize.completed": "kogi.network.database",
		"database.access.updated":    "kogi.network.database",
		"database.concurrency.updated": "kogi.network.database",
	})

	// Seed: announce gateway is online
	bus.PublishWith("gateway.component.registered",
		`{"id":"kogi.network.gateway","kind":"gateway","status":"seeded"}`,
		eventbus.PublishOptions{Source: "kogi.network.gateway"})
	if gatewayRuntime != nil {
		gatewayRuntime.Publish("gateway.component.registered", "kogi.network.gateway", "kogi.network.gateway", `{"id":"kogi.network.gateway","kind":"gateway","status":"seeded"}`)
	}
}

// ── Engine mirror ─────────────────────────────────────────────────────────────

func mirrorToEngine(bus *eventbus.Bus, registry *mesh.Registry, origin eventbus.Event) mesh.RoutedMessage {
	enginePayload := map[string]string{
		"event_id":           origin.ID,
		"event_type":         strings.ReplaceAll(origin.Topic, ".", "_"),
		"module":             inferModule(origin.Topic, origin.Target),
		"topic":              origin.Topic,
		"source":             origin.Source,
		"target":             origin.Target,
		"latency_ms":         metadataDefault(origin.Metadata, "latency_ms", "0"),
		"queue_depth":        metadataDefault(origin.Metadata, "queue_depth", "0"),
		"cpu_pct":            metadataDefault(origin.Metadata, "cpu_pct", "0"),
		"memory_mb":          metadataDefault(origin.Metadata, "memory_mb", "0"),
		"host_id":            metadataDefault(origin.Metadata, "host_id", "kogi-host-001"),
		"host_cpu_pct":       metadataDefault(origin.Metadata, "host_cpu_pct", "0"),
		"host_memory_pct":    metadataDefault(origin.Metadata, "host_memory_pct", "0"),
		"host_disk_pct":      metadataDefault(origin.Metadata, "host_disk_pct", "0"),
		"host_process_count": metadataDefault(origin.Metadata, "host_process_count", "0"),
	}
	encoded, _ := json.Marshal(enginePayload)
	event := bus.PublishWith(
		"engine.ingest",
		string(encoded),
		eventbus.PublishOptions{
			Source:   "kogi.network.gateway",
			Target:   "kogi.engine",
			Metadata: map[string]string{"origin_topic": origin.Topic},
		},
	)
	if gatewayRuntime != nil {
		gatewayRuntime.Publish("engine.ingest", "kogi.network.gateway", "kogi.engine", string(encoded))
	}
	msg := registry.Send(
		"kogi.network.gateway", "kogi.engine",
		event.Topic, event.Payload,
		map[string]string{"origin_topic": origin.Topic},
	)
	if gatewayRuntime != nil {
		gatewayRuntime.Debugf("engine mirror status=%s message_id=%s origin_topic=%s", msg.Status, msg.ID, origin.Topic)
	}
	return msg
}

// ── Topic → module inference ──────────────────────────────────────────────────

func inferModule(topic, target string) string {
	normalized := strings.ToLower(topic + " " + target)
	switch {
	case strings.Contains(normalized, "ims"):
		return "ims"
	case strings.Contains(normalized, "office.portfolio"), strings.Contains(normalized, "office"):
		return "office"
	case strings.Contains(normalized, "portfolio"):
		return "portfolio"
	case strings.Contains(normalized, "exchange"):
		return "exchange"
	case strings.Contains(normalized, "bank"):
		return "bank"
	case strings.Contains(normalized, "marketplace"):
		return "marketplace"
	case strings.Contains(normalized, "studio"):
		return "studio"
	case strings.Contains(normalized, "community"):
		return "community"
	case strings.Contains(normalized, "developer"):
		return "developer"
	case strings.Contains(normalized, "profile"):
		return "profile"
	case strings.Contains(normalized, "organizations"):
		return "organizations"
	case strings.Contains(normalized, "database"):
		return "database"
	case strings.Contains(normalized, "engine"):
		return "engine"
	case strings.Contains(normalized, "auth"):
		return "auth"
	case strings.Contains(normalized, "kernel"):
		return "kogi-kernel"
	case strings.Contains(normalized, "host"):
		return "kogi-host"
	case strings.Contains(normalized, "server"):
		return "kogi-server"
	default:
		return "gateway"
	}
}

// ── Helpers ───────────────────────────────────────────────────────────────────

func filterRoutesByPrefix(routes map[string]string, prefix string) map[string]string {
	result := map[string]string{}
	for topic, target := range routes {
		if strings.HasPrefix(topic, prefix) {
			result[topic] = target
		}
	}
	return result
}

func metadataDefault(metadata map[string]string, key, fallback string) string {
	if value, ok := metadata[key]; ok && value != "" {
		return value
	}
	return fallback
}

func decodeJSON(r *http.Request, target interface{}) error {
	defer r.Body.Close()
	decoder := json.NewDecoder(r.Body)
	decoder.DisallowUnknownFields()
	if err := decoder.Decode(target); err != nil {
		return fmt.Errorf("invalid JSON: %w", err)
	}
	return nil
}

func queryInt(r *http.Request, key string, fallback int) int {
	value := r.URL.Query().Get(key)
	if value == "" {
		return fallback
	}
	parsed, err := strconv.Atoi(value)
	if err != nil || parsed <= 0 {
		return fallback
	}
	return parsed
}

func writeJSON(w http.ResponseWriter, status int, payload interface{}) {
	w.Header().Set("Content-Type", "application/json")
	w.WriteHeader(status)
	_ = json.NewEncoder(w).Encode(payload)
}

func resolveAddr(defaultPort, servicePortEnv string) string {
	if port := os.Getenv(servicePortEnv); port != "" {
		return toListenAddr(port)
	}
	if port := os.Getenv("KOGI_PORT"); port != "" {
		return toListenAddr(port)
	}
	return ":" + defaultPort
}

func toListenAddr(port string) string {
	if strings.HasPrefix(port, ":") {
		return port
	}
	return ":" + port
}

func defaultIfEmpty(value, fallback string) string {
	if value == "" {
		return fallback
	}
	return value
}
