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

	"kogi.services/lib/eventbus"
	"kogi.services/lib/mesh"
)

type gatewayInfo struct {
	Name           string            `json:"name"`
	Version        string            `json:"version"`
	Services       []string          `json:"services"`
	ComponentCount int               `json:"component_count"`
	TopicCounts    map[string]uint64 `json:"topic_counts"`
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

func main() {
	bus := eventbus.NewWithHistory(25000)
	registry := mesh.NewRegistry()
	seedRegistry(registry)

	bus.SubscribeAll("gateway.logger", func(e eventbus.Event) {
		log.Printf("pubsub topic=%s source=%s target=%s payload=%s", e.Topic, e.Source, e.Target, e.Payload)
	})

	mux := http.NewServeMux()

	mux.HandleFunc("/health", func(w http.ResponseWriter, r *http.Request) {
		writeJSON(w, http.StatusOK, map[string]interface{}{
			"status":    "ok",
			"service":   "kogi-go-gateway",
			"component": "kogi.services.gateway",
		})
	})

	mux.HandleFunc("/api/v1/gateway/info", func(w http.ResponseWriter, r *http.Request) {
		writeJSON(w, http.StatusOK, gatewayInfo{
			Name:           "kogi-go-gateway",
			Version:        "0.2.0",
			Services:       []string{"auth", "portfolio", "exchange", "ims", "office"},
			ComponentCount: len(registry.Components()),
			TopicCounts:    bus.TopicStats(),
			Timestamp:      time.Now().UTC().Format(time.RFC3339),
		})
	})

	mux.HandleFunc("/api/v1/gateway/routes", func(w http.ResponseWriter, r *http.Request) {
		writeJSON(w, http.StatusOK, map[string]interface{}{
			"gateway": "kogi-go-gateway",
			"routes": map[string]string{
				"auth":      "/services/auth",
				"portfolio": "/services/portfolio",
				"exchange":  "/services/exchange",
				"ims":       "/services/ims",
				"office":    "/services/office",
			},
			"topic_routes": registry.TopicRoutes(),
			"discovery":    "kogi gateway now manages pub/sub + component communications + engine ingest routing",
		})
	})

	mux.HandleFunc("/api/v1/gateway/components/register", func(w http.ResponseWriter, r *http.Request) {
		if r.Method != http.MethodPost {
			writeJSON(w, http.StatusMethodNotAllowed, map[string]string{"error": "method_not_allowed"})
			return
		}

		var request registerComponentRequest
		if err := decodeJSON(r, &request); err != nil {
			writeJSON(w, http.StatusBadRequest, map[string]string{"error": err.Error()})
			return
		}

		registered := registry.Register(mesh.Component{
			ID:             request.ID,
			Kind:           request.Kind,
			Endpoint:       request.Endpoint,
			HealthPath:     request.HealthPath,
			NetworkManager: request.NetworkManager,
			Status:         request.Status,
			Metadata:       request.Metadata,
		})

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

	mux.HandleFunc("/api/v1/gateway/pubsub/publish", func(w http.ResponseWriter, r *http.Request) {
		if r.Method != http.MethodPost {
			writeJSON(w, http.StatusMethodNotAllowed, map[string]string{"error": "method_not_allowed"})
			return
		}

		var request publishRequest
		if err := decodeJSON(r, &request); err != nil {
			writeJSON(w, http.StatusBadRequest, map[string]string{"error": err.Error()})
			return
		}
		if request.Topic == "" {
			writeJSON(w, http.StatusBadRequest, map[string]string{"error": "topic is required"})
			return
		}

		source := defaultIfEmpty(request.Source, "gateway")
		target := registry.ResolveTarget(request.Topic, request.Target)
		event := bus.PublishWith(request.Topic, request.Payload, eventbus.PublishOptions{
			Source:   source,
			Target:   target,
			Metadata: request.Metadata,
		})
		networkMessage := registry.Send(source, target, request.Topic, request.Payload, request.Metadata)
		engineMessage := mirrorToEngine(bus, registry, event)

		writeJSON(w, http.StatusAccepted, map[string]interface{}{
			"event":          event,
			"route_target":   target,
			"network_result": networkMessage,
			"engine_result":  engineMessage,
		})
	})

	mux.HandleFunc("/api/v1/gateway/pubsub/history", func(w http.ResponseWriter, r *http.Request) {
		limit := queryInt(r, "limit", 100)
		topic := r.URL.Query().Get("topic")
		writeJSON(w, http.StatusOK, map[string]interface{}{
			"events": bus.History(limit, topic),
		})
	})

	mux.HandleFunc("/api/v1/gateway/pubsub/topics", func(w http.ResponseWriter, r *http.Request) {
		writeJSON(w, http.StatusOK, map[string]interface{}{
			"topic_counts":   bus.TopicStats(),
			"subscriptions":  bus.Subscriptions(),
			"total_messages": len(bus.History(0, "")),
		})
	})

	mux.HandleFunc("/api/v1/gateway/network/send", func(w http.ResponseWriter, r *http.Request) {
		if r.Method != http.MethodPost {
			writeJSON(w, http.StatusMethodNotAllowed, map[string]string{"error": "method_not_allowed"})
			return
		}

		var request networkSendRequest
		if err := decodeJSON(r, &request); err != nil {
			writeJSON(w, http.StatusBadRequest, map[string]string{"error": err.Error()})
			return
		}
		if request.Topic == "" {
			writeJSON(w, http.StatusBadRequest, map[string]string{"error": "topic is required"})
			return
		}

		target := registry.ResolveTarget(request.Topic, request.Target)
		message := registry.Send(
			defaultIfEmpty(request.Source, "gateway"),
			target,
			request.Topic,
			request.Payload,
			request.Metadata,
		)
		event := bus.PublishWith(request.Topic, request.Payload, eventbus.PublishOptions{
			Source:   defaultIfEmpty(request.Source, "gateway"),
			Target:   target,
			Metadata: request.Metadata,
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
		writeJSON(w, http.StatusOK, map[string]interface{}{
			"messages": registry.MessageHistory(limit),
		})
	})

	mux.HandleFunc("/api/v1/gateway/network/routes", func(w http.ResponseWriter, r *http.Request) {
		writeJSON(w, http.StatusOK, map[string]interface{}{
			"topic_routes": registry.TopicRoutes(),
		})
	})

	mux.HandleFunc("/api/v1/gateway/engine/stream", func(w http.ResponseWriter, r *http.Request) {
		limit := queryInt(r, "limit", 100)
		events := bus.History(limit*2, "")
		ingest := make([]eventbus.Event, 0, limit)
		for _, event := range events {
			if event.Topic == "engine.ingest" || event.Target == "kogi.engine" {
				ingest = append(ingest, event)
			}
			if len(ingest) >= limit {
				break
			}
		}

		writeJSON(w, http.StatusOK, map[string]interface{}{
			"engine_ingest": ingest,
			"count":         len(ingest),
		})
	})

	addr := resolveAddr("8090", "KOGI_GATEWAY_PORT")
	log.Printf("kogi-go-gateway listening on %s", addr)
	log.Fatal(http.ListenAndServe(addr, mux))
}

func seedRegistry(registry *mesh.Registry) {
	registry.Register(mesh.Component{
		ID:             "kogi.services.gateway",
		Kind:           "gateway",
		Endpoint:       "http://127.0.0.1:8090",
		NetworkManager: "kogi-go-network",
		Status:         "active",
		Metadata:       map[string]string{"owner": "kogi-host"},
	})
	registry.Register(mesh.Component{
		ID:             "kogi.server",
		Kind:           "server",
		Endpoint:       "http://127.0.0.1:8080",
		NetworkManager: "kogi-go-network",
		Status:         "active",
	})
	registry.Register(mesh.Component{
		ID:             "kogi.engine",
		Kind:           "engine",
		Endpoint:       "local://kogi-engine",
		NetworkManager: "kogi-go-network",
		Status:         "active",
	})
	registry.Register(mesh.Component{
		ID:             "kogi.services.auth",
		Kind:           "service",
		Endpoint:       "http://127.0.0.1:9001",
		NetworkManager: "kogi-go-network",
		Status:         "active",
	})
	registry.Register(mesh.Component{
		ID:             "kogi.services.portfolio",
		Kind:           "service",
		Endpoint:       "http://127.0.0.1:9002",
		NetworkManager: "kogi-go-network",
		Status:         "active",
	})
	registry.Register(mesh.Component{
		ID:             "kogi.services.exchange",
		Kind:           "service",
		Endpoint:       "http://127.0.0.1:9004",
		NetworkManager: "kogi-go-network",
		Status:         "active",
	})
	registry.Register(mesh.Component{
		ID:             "kogi.services.ims",
		Kind:           "service",
		Endpoint:       "http://127.0.0.1:9005",
		NetworkManager: "kogi-go-network",
		Status:         "active",
	})
	registry.Register(mesh.Component{
		ID:             "kogi.services.office",
		Kind:           "service",
		Endpoint:       "http://127.0.0.1:9006",
		NetworkManager: "kogi-go-network",
		Status:         "active",
	})

	registry.SetTopicRoute("ims.identity.created", "kogi.services.ims")
	registry.SetTopicRoute("ims.profile.updated", "kogi.services.ims")
	registry.SetTopicRoute("office.dashboard.refresh", "kogi.services.office")
	registry.SetTopicRoute("portfolio.item.created", "kogi.services.portfolio")
	registry.SetTopicRoute("exchange.order.created", "kogi.services.exchange")
	registry.SetTopicRoute("auth.login.request", "kogi.services.auth")
}

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
			Source:   "kogi.services.gateway",
			Target:   "kogi.engine",
			Metadata: map[string]string{"origin_topic": origin.Topic},
		},
	)

	return registry.Send(
		"kogi.services.gateway",
		"kogi.engine",
		event.Topic,
		event.Payload,
		map[string]string{"origin_topic": origin.Topic},
	)
}

func inferModule(topic, target string) string {
	normalized := strings.ToLower(topic + " " + target)
	switch {
	case strings.Contains(normalized, "ims"):
		return "ims"
	case strings.Contains(normalized, "office"):
		return "office"
	case strings.Contains(normalized, "portfolio"):
		return "portfolio"
	case strings.Contains(normalized, "exchange"):
		return "exchange"
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
