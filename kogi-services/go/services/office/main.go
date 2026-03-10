package main

// office_service.go
//
// HTTP service for the Kogi Office module.  Delegates all PortfolioSystem
// operations to kogi_office.dll (or kogi-portfolio-system exe as fallback)
// and publishes/subscribes events through the gateway bus.
//
// All utility functions (Rust bridge, DLL loading, HTTP helpers, gateway
// pub/sub, health reporting, file-system helpers) live in utility.go.
//
// Endpoints (all under /api/v1/office/…):
//   GET  /health
//   GET  /api/v1/office/runtime
//   GET  /api/v1/office                          overview
//   GET  /api/v1/office/dashboard                dashboard snapshot
//   POST /api/v1/office/dashboard/ack            acknowledge notification
//   GET  /api/v1/office/portfolio                portfolio overview (proxy)
//   GET  /api/v1/office/portfolio/snapshot       full portfolio snapshot
//   GET  /api/v1/office/portfolio/metadata       portfolio metadata
//   GET  /api/v1/office/portfolio/components     all components
//   POST /api/v1/office/portfolio/components     create component
//   GET  /api/v1/office/portfolio/components/{id}
//   PUT  /api/v1/office/portfolio/components/{id}
//   DELETE /api/v1/office/portfolio/components/{id}
//   GET  /api/v1/office/portfolio/components/type/{type}
//   POST /api/v1/office/portfolio/books
//   POST /api/v1/office/portfolio/active
//   POST /api/v1/office/portfolio/graph/hierarchy
//   DELETE /api/v1/office/portfolio/graph/hierarchy
//   POST /api/v1/office/portfolio/graph/dependency
//   DELETE /api/v1/office/portfolio/graph/dependency
//   POST /api/v1/office/portfolio/graph/link
//   DELETE /api/v1/office/portfolio/graph/link
//   POST /api/v1/office/portfolio/graph/member
//   DELETE /api/v1/office/portfolio/graph/member
//   GET  /api/v1/office/portfolio/graph/subtree/{id}
//   GET  /api/v1/office/portfolio/graph/dependencies/{id}
//   GET  /api/v1/office/portfolio/graph/order
//   GET  /api/v1/office/portfolio/snapshots
//   POST /api/v1/office/portfolio/snapshots
//   POST /api/v1/office/portfolio/snapshots/{id}/restore
//   GET  /api/v1/office/portfolio/checkpoints
//   POST /api/v1/office/portfolio/checkpoints
//   POST /api/v1/office/portfolio/checkpoints/{id}/restore
//   GET  /api/v1/office/portfolio/query   ?pql=…
//   GET  /api/v1/office/portfolio/events
//   POST /api/v1/office/portfolio/governance/policy/attach
//   POST /api/v1/office/portfolio/governance/policy/detach
//   POST /api/v1/office/portfolio/governance/approval/request
//   POST /api/v1/office/portfolio/governance/approval/{id}/resolve
//   POST /api/v1/office/portfolio/governance/resource/allocate
//   POST /api/v1/office/portfolio/governance/resource/consume
//   GET  /api/v1/office/portfolio/governance/resource/overruns
//   GET  /api/v1/office/portfolio/governance/resource/{id}
//   GET  /api/v1/office/portfolio/models/health/{id}
//   POST /api/v1/office/portfolio/models/project
//   POST /api/v1/office/portfolio/models/program
//   GET  /api/v1/office/portfolio/models/subportfolio/{id}
//   GET  /api/v1/office/portfolio/models/resource/{id}
//   POST /api/v1/office/portfolio/models/asset
//   GET/POST /api/v1/office/portfolio/models/artifact/{id}
//   GET/POST /api/v1/office/portfolio/models/binder/{id}
//   POST /api/v1/office/portfolio/models/book
//   GET/POST /api/v1/office/portfolio/models/folder/{id}
//   GET  /api/v1/office/portfolio/models/record/{id}
//   GET  /api/v1/office/timeline
//   POST /api/v1/office/timeline/events
//   GET  /api/v1/office/workspace
//   POST /api/v1/office/workspace/stories
//   GET  /api/v1/office/assistant
//   POST /api/v1/office/assistant/subscriptions
//   GET  /api/v1/office/pubsub/history
//   GET  /api/v1/office/pubsub/topics
//   GET  /api/v1/office/pubsub/dead-letters
//   GET  /api/v1/office/pubsub/replay
//   GET  /api/v1/office/pubsub/metrics
//   GET  /api/v1/office/mesh
//   GET  /api/v1/office/mesh/messages
//   GET  /api/v1/office/mesh/routes

import (
	"bytes"
	"context"
	"encoding/json"
	"fmt"
	"io"
	"log"
	"net/http"
	"strings"
	"sync"
	"time"
)

const (
	officeServiceID   = "kogi.services.office"
	officeServiceName = "office-service"
	officeDefaultPort = "9006"
	officeGateway     = "http://127.0.0.1:8090"
	portfolioService  = "http://127.0.0.1:9002"
)

// ── In-process event log ──────────────────────────────────────────────────────

type officeEventEntry struct {
	Topic     string `json:"topic"`
	Payload   string `json:"payload"`
	Source    string `json:"source"`
	Timestamp string `json:"timestamp"`
}

var (
	officeEventMu  sync.RWMutex
	officeEventLog []officeEventEntry
)

func recordOfficeEvent(topic, payload, source string) {
	officeEventMu.Lock()
	defer officeEventMu.Unlock()
	officeEventLog = append(officeEventLog, officeEventEntry{
		Topic:     topic,
		Payload:   payload,
		Source:    source,
		Timestamp: time.Now().UTC().Format(time.RFC3339),
	})
	if len(officeEventLog) > 2000 {
		officeEventLog = officeEventLog[len(officeEventLog)-2000:]
	}
}

// ── Service-specific publish / gateway helpers ────────────────────────────────

func officePublish(topic, payload string, meta map[string]string) {
	publish(officeGateway, officeServiceID, topic, payload, meta)
	recordOfficeEvent(topic, payload, officeServiceID)
}

func officeGatewaySubscribePrefix(prefix, consumer string) {
	gatewaySubscribePrefix(officeGateway, prefix, consumer)
}

func officeReportHealth(healthy bool, note string) {
	reportHealth(officeGateway, officeServiceID, healthy, note)
}

func officeHealthLoop() {
	healthLoop(officeGateway, officeServiceID, 30*time.Second, func() (bool, string) {
		_, err := officeResolveBinary()
		return err == nil, ""
	})
}

func officeDeadLetterMonitor() {
	pollDeadLetters(officeGateway,
		func(topic string) bool { return strings.HasPrefix(topic, "office.") },
		func(id, topic string) { log.Printf("[office-svc] dead-letter id=%s topic=%s", id, topic) },
	)
}

func officeSubscribeGateway(topics []string, handler func(topic, payload string)) {
	subscribeGatewayExact(officeGateway, topics, handler)
}

func officeSubscribeGatewayPrefix(prefixes []string, handler func(topic, payload string)) {
	subscribeGatewayPrefix(officeGateway, prefixes, handler)
}

func officeReplayGateway(topic, prefix, from, to string) {
	replayGateway(officeGateway, topic, prefix, from, to,
		func(t, p, s string) { recordOfficeEvent(t, p, s) })
}

func officeRegisterWithGateway(selfEndpoint string) {
	registerWithGateway(
		officeGateway, officeServiceID, selfEndpoint, "/health",
		[]string{
			"office.dashboard.refresh", "office.timeline.updated",
			"office.workspace.story.created", "office.assistant.subscription.active",
			"office.portfolio.item.created", "office.portfolio.component.updated",
			"office.portfolio.component.removed", "office.portfolio.graph.changed",
			"office.portfolio.snapshot.saved", "office.portfolio.checkpoint.created",
			"office.portfolio.governance.approval.requested",
			"office.portfolio.governance.resource.allocated",
			"office.portfolio.model.computed",
		},
		[]string{
			"portfolio.item.created", "portfolio.component.updated",
			"portfolio.component.removed", "portfolio.graph.changed",
			"portfolio.snapshot.saved", "portfolio.checkpoint.created",
			"portfolio.governance.resource.allocated", "portfolio.governance.resource.consumed",
			"portfolio.governance.approval.requested", "portfolio.governance.approval.resolved",
			"portfolio.model.computed", "portfolio.health.updated",
			"ims.profile.updated", "exchange.trade.executed",
		},
		[]struct{ Prefix, Consumer string }{
			{"portfolio.", officeServiceID},
			{"ims.", officeServiceID},
			{"exchange.trade", officeServiceID},
		},
	)
}

// ── Portfolio proxy (to portfolio-service REST API) ───────────────────────────
//
// These helpers proxy requests to portfolio-service when the office service
// is acting as a pass-through orchestrator.  Direct DLL calls are preferred
// for mutations; proxying is used only when routing to the portfolio-service
// HTTP layer is needed (e.g. for cross-service event publishing).

func officeProxyPortfolio(w http.ResponseWriter, r *http.Request) {
	downstream := strings.Replace(r.URL.Path, "/api/v1/office/portfolio", "/api/v1/portfolio", 1)
	if r.URL.RawQuery != "" {
		downstream += "?" + r.URL.RawQuery
	}
	targetURL := portfolioService + downstream

	ctx, cancel := context.WithTimeout(r.Context(), 10*time.Second)
	defer cancel()

	var bodyReader io.Reader
	if r.Body != nil && r.ContentLength != 0 {
		bodyBytes, _ := io.ReadAll(r.Body)
		defer r.Body.Close()
		bodyReader = bytes.NewReader(bodyBytes)
	}

	proxyReq, err := http.NewRequestWithContext(ctx, r.Method, targetURL, bodyReader)
	if err != nil {
		writeOfficeJSON(w, http.StatusInternalServerError, map[string]string{"error": err.Error()})
		return
	}
	proxyReq.Header.Set("Content-Type", "application/json")
	proxyReq.Header.Set("X-Forwarded-By", officeServiceID)

	resp, err := http.DefaultClient.Do(proxyReq)
	if err != nil {
		writeOfficeJSON(w, http.StatusBadGateway, map[string]string{
			"error":   "portfolio-service unavailable",
			"details": err.Error(),
		})
		return
	}
	defer resp.Body.Close()

	w.Header().Set("Content-Type", "application/json")
	w.WriteHeader(resp.StatusCode)
	_, _ = io.Copy(w, resp.Body)
}

func officeProxyPortfolioAndPublish(w http.ResponseWriter, r *http.Request, topic, payloadFmt string, args ...interface{}) {
	rec := &responseRecorder{header: make(http.Header), code: http.StatusOK}
	officeProxyPortfolio(rec, r)
	for k, vs := range rec.header {
		for _, v := range vs {
			w.Header().Add(k, v)
		}
	}
	w.WriteHeader(rec.code)
	_, _ = w.Write(rec.body)
	if rec.code < 300 {
		payload := fmt.Sprintf(payloadFmt, args...)
		go officePublish(topic, payload, nil)
	}
}

// officeDLL calls the office DLL and also records the event on success.
func officeDLL(funcName string, payload interface{}, topic string, pubPayload string) interface{} {
	result := officeRust(funcName, payload, nil)
	if result != nil && topic != "" {
		go officePublish(topic, pubPayload, nil)
	}
	return result
}

// ── main ──────────────────────────────────────────────────────────────────────

func main() {
	mux := http.NewServeMux()

	addr := resolveOfficeAddr(officeDefaultPort, "KOGI_OFFICE_PORT")
	selfEndpoint := fmt.Sprintf("http://127.0.0.1%s", addr)

	go func() {
		time.Sleep(600 * time.Millisecond)
		officeRegisterWithGateway(selfEndpoint)
		go officeSubscribeGatewayPrefix(
			[]string{"portfolio.", "ims.", "exchange.trade"},
			func(topic, payload string) {
				recordOfficeEvent(topic, payload, "gateway.inbound")
				log.Printf("[office-svc] received topic=%s payload=%s", topic, payload)
			},
		)
		go officeHealthLoop()
		go officeDeadLetterMonitor()
	}()

	// ── Health ────────────────────────────────────────────────────────────
	mux.HandleFunc("/health", func(w http.ResponseWriter, r *http.Request) {
		writeOfficeJSON(w, http.StatusOK, map[string]string{"status": "ok", "service": officeServiceName})
	})

	// ── Runtime manifest ──────────────────────────────────────────────────
	mux.HandleFunc("/api/v1/office/runtime", func(w http.ResponseWriter, r *http.Request) {
		writeOfficeJSON(w, http.StatusOK, map[string]interface{}{
			"service":            officeServiceName,
			"component_id":       officeServiceID,
			"gateway":            officeGateway,
			"portfolio_service":  portfolioService,
			"network_manager":    "kogi-go-network",
			"office_rust_binary": officeResolveBinaryHint(),
			"dll_config": map[string]string{
				"office_dll_env": OfficeDLLConfig.EnvBinKey,
				"office_exe_env": OfficeExeConfig.EnvBinKey,
			},
			"publishes": []string{
				"office.dashboard.refresh", "office.timeline.updated",
				"office.workspace.story.created", "office.assistant.subscription.active",
				"office.portfolio.item.created", "office.portfolio.component.updated",
				"office.portfolio.component.removed", "office.portfolio.graph.changed",
				"office.portfolio.snapshot.saved", "office.portfolio.checkpoint.created",
				"office.portfolio.governance.approval.requested",
				"office.portfolio.governance.resource.allocated",
				"office.portfolio.model.computed",
			},
			"subscribed_prefixes": []string{"portfolio.", "ims.", "exchange.trade"},
			"subscribes": []string{
				"portfolio.item.created", "portfolio.component.updated",
				"portfolio.component.removed", "portfolio.graph.changed",
				"portfolio.snapshot.saved", "portfolio.checkpoint.created",
				"portfolio.governance.resource.allocated", "portfolio.governance.resource.consumed",
				"portfolio.governance.approval.requested", "portfolio.governance.approval.resolved",
				"portfolio.model.computed", "portfolio.health.updated",
				"ims.profile.updated", "exchange.trade.executed",
			},
			"engine_stream": "engine.ingest",
		})
	})

	// ── Office overview ───────────────────────────────────────────────────
	mux.HandleFunc("/api/v1/office", func(w http.ResponseWriter, r *http.Request) {
		if r.URL.Path != "/api/v1/office" && r.URL.Path != "/api/v1/office/" {
			http.NotFound(w, r)
			return
		}
		writeOfficeJSON(w, http.StatusOK, officeRust("kogi_office_overview", nil, map[string]interface{}{
			"module": "kogi.office", "application": "Kogi Office", "service": officeServiceName,
			"views": []map[string]string{
				{"id": "dashboard", "title": "Office Dashboard", "status": "active"},
				{"id": "portfolio", "title": "Office Portfolio", "status": "active"},
				{"id": "timeline", "title": "Office Timeline", "status": "active"},
				{"id": "workspace", "title": "Office Workspace", "status": "active"},
				{"id": "assistant", "title": "Office Assistant", "status": "active"},
			},
			"integrations": []string{"jira", "monday", "openai", "gitlab", "github"},
		}))
	})

	// ── Dashboard ─────────────────────────────────────────────────────────
	mux.HandleFunc("/api/v1/office/dashboard", func(w http.ResponseWriter, r *http.Request) {
		if r.URL.Path == "/api/v1/office/dashboard" || r.URL.Path == "/api/v1/office/dashboard/" {
			writeOfficeJSON(w, http.StatusOK, officeRust("kogi_office_dashboard", nil, map[string]interface{}{
				"view": "dashboard",
				"active_projects": []map[string]interface{}{
					{"id": "proj-kogi-mvp", "name": "Kogi MVP Prototype", "status": "active", "progress_percent": 68},
					{"id": "proj-api-fabric", "name": "API Fabric Reconcile", "status": "active", "progress_percent": 44},
				},
				"notifications": []map[string]string{
					{"id": "notif-001", "category": "task", "message": "3 blocked stories need triage"},
					{"id": "notif-002", "category": "integration", "message": "Jira sync delayed for 2 projects"},
				},
			}))
			return
		}
		http.NotFound(w, r)
	})

	mux.HandleFunc("/api/v1/office/dashboard/ack", func(w http.ResponseWriter, r *http.Request) {
		if r.Method != http.MethodPost {
			writeOfficeJSON(w, http.StatusMethodNotAllowed, map[string]string{"error": "method_not_allowed"})
			return
		}
		var body struct {
			NotificationID string `json:"notification_id"`
		}
		if err := decodeOfficeBody(r, &body); err != nil {
			writeOfficeJSON(w, http.StatusBadRequest, map[string]string{"error": err.Error()})
			return
		}
		result := officeDLL("kogi_ack_dashboard_notification",
			map[string]string{"notification_id": body.NotificationID},
			"office.dashboard.refresh",
			fmt.Sprintf(`{"notification_id":%q,"action":"acknowledged"}`, body.NotificationID))
		if result == nil {
			result = map[string]interface{}{"ok": true, "message": "acknowledged"}
		}
		writeOfficeJSON(w, http.StatusOK, result)
	})

	// ── Portfolio: all routes call the DLL directly ───────────────────────
	//
	// For mutations we call the DLL and also publish an office-namespace event.
	// For reads we call the DLL.  A proxy to portfolio-service is used only when
	// the DLL is explicitly unavailable (handled inside officeRust fallback).

	// Snapshot / metadata / events / query — read-only, call DLL
	mux.HandleFunc("/api/v1/office/portfolio/snapshot", func(w http.ResponseWriter, r *http.Request) {
		writeOfficeJSON(w, http.StatusOK, officeRust("kogi_portfolio_snapshot", nil,
			map[string]interface{}{"snapshot_id": "unavailable"}))
	})
	mux.HandleFunc("/api/v1/office/portfolio/metadata", func(w http.ResponseWriter, r *http.Request) {
		writeOfficeJSON(w, http.StatusOK, officeRust("kogi_portfolio_metadata", nil, map[string]interface{}{}))
	})
	mux.HandleFunc("/api/v1/office/portfolio/events", func(w http.ResponseWriter, r *http.Request) {
		writeOfficeJSON(w, http.StatusOK, officeRust("kogi_portfolio_event_log", nil,
			map[string]interface{}{"events": []interface{}{}}))
	})
	mux.HandleFunc("/api/v1/office/portfolio/query", func(w http.ResponseWriter, r *http.Request) {
		pql := r.URL.Query().Get("pql")
		writeOfficeJSON(w, http.StatusOK, officeRust("kogi_portfolio_query_pql",
			map[string]string{"pql": pql},
			map[string]interface{}{"query": pql, "results": []interface{}{}}))
	})

	// Snapshots — list / save / restore
	mux.HandleFunc("/api/v1/office/portfolio/snapshots", func(w http.ResponseWriter, r *http.Request) {
		if r.Method == http.MethodPost {
			var req struct{ Label *string `json:"label,omitempty"` }
			_ = decodeOfficeBody(r, &req)
			result, err := officeCallRust("kogi_portfolio_save_snapshot", req)
			if err != nil {
				writeOfficeJSON(w, http.StatusInternalServerError, map[string]string{"error": err.Error()})
				return
			}
			go officePublish("office.portfolio.snapshot.saved", `{}`, nil)
			writeOfficeJSON(w, http.StatusCreated, result)
		} else {
			writeOfficeJSON(w, http.StatusOK, map[string]interface{}{
				"snapshots": officeRust("kogi_portfolio_list_snapshots", nil, []interface{}{}),
			})
		}
	})
	mux.HandleFunc("/api/v1/office/portfolio/snapshots/", func(w http.ResponseWriter, r *http.Request) {
		parts := strings.SplitN(officeTrimPrefix(r, "/api/v1/office/portfolio/snapshots/"), "/", 2)
		if len(parts) != 2 || parts[1] != "restore" {
			writeOfficeJSON(w, http.StatusBadRequest, map[string]string{"error": "use /{id}/restore"})
			return
		}
		result, err := officeCallRust("kogi_portfolio_restore_snapshot", map[string]string{"snapshot_id": parts[0]})
		if err != nil {
			writeOfficeJSON(w, http.StatusInternalServerError, map[string]string{"error": err.Error()})
			return
		}
		writeOfficeJSON(w, http.StatusOK, result)
	})

	// Checkpoints — list / save / restore
	mux.HandleFunc("/api/v1/office/portfolio/checkpoints", func(w http.ResponseWriter, r *http.Request) {
		if r.Method == http.MethodPost {
			var req struct {
				Label string  `json:"label"`
				Note  *string `json:"note,omitempty"`
			}
			if err := decodeOfficeBody(r, &req); err != nil {
				writeOfficeJSON(w, http.StatusBadRequest, map[string]string{"error": err.Error()})
				return
			}
			result, err := officeCallRust("kogi_portfolio_save_checkpoint", req)
			if err != nil {
				writeOfficeJSON(w, http.StatusInternalServerError, map[string]string{"error": err.Error()})
				return
			}
			go officePublish("office.portfolio.checkpoint.created", `{}`, nil)
			writeOfficeJSON(w, http.StatusCreated, result)
		} else {
			writeOfficeJSON(w, http.StatusOK, map[string]interface{}{
				"checkpoints": officeRust("kogi_portfolio_list_checkpoints", nil, []interface{}{}),
			})
		}
	})
	mux.HandleFunc("/api/v1/office/portfolio/checkpoints/", func(w http.ResponseWriter, r *http.Request) {
		parts := strings.SplitN(officeTrimPrefix(r, "/api/v1/office/portfolio/checkpoints/"), "/", 2)
		if len(parts) != 2 || parts[1] != "restore" {
			writeOfficeJSON(w, http.StatusBadRequest, map[string]string{"error": "use /{id}/restore"})
			return
		}
		result, err := officeCallRust("kogi_portfolio_restore_checkpoint", map[string]string{"checkpoint_id": parts[0]})
		if err != nil {
			writeOfficeJSON(w, http.StatusInternalServerError, map[string]string{"error": err.Error()})
			return
		}
		writeOfficeJSON(w, http.StatusOK, result)
	})

	// Books
	mux.HandleFunc("/api/v1/office/portfolio/books", func(w http.ResponseWriter, r *http.Request) {
		if r.Method == http.MethodPost {
			officeProxyPortfolioAndPublish(w, r, "office.portfolio.item.created", `{"type":"book"}`)
		} else {
			officeProxyPortfolio(w, r)
		}
	})

	// Active portfolio
	mux.HandleFunc("/api/v1/office/portfolio/active", func(w http.ResponseWriter, r *http.Request) {
		officeProxyPortfolio(w, r)
	})

	// Graph operations — call DLL and publish
	mux.HandleFunc("/api/v1/office/portfolio/graph/hierarchy", func(w http.ResponseWriter, r *http.Request) {
		officeProxyPortfolioAndPublish(w, r, "office.portfolio.graph.changed",
			`{"edge":"hierarchy","method":%q}`, r.Method)
	})
	mux.HandleFunc("/api/v1/office/portfolio/graph/dependency", func(w http.ResponseWriter, r *http.Request) {
		officeProxyPortfolioAndPublish(w, r, "office.portfolio.graph.changed",
			`{"edge":"dependency","method":%q}`, r.Method)
	})
	mux.HandleFunc("/api/v1/office/portfolio/graph/link", func(w http.ResponseWriter, r *http.Request) {
		officeProxyPortfolioAndPublish(w, r, "office.portfolio.graph.changed",
			`{"edge":"link","method":%q}`, r.Method)
	})
	mux.HandleFunc("/api/v1/office/portfolio/graph/member", func(w http.ResponseWriter, r *http.Request) {
		officeProxyPortfolioAndPublish(w, r, "office.portfolio.graph.changed",
			`{"edge":"member","method":%q}`, r.Method)
	})
	mux.HandleFunc("/api/v1/office/portfolio/graph/subtree/", func(w http.ResponseWriter, r *http.Request) {
		id := officeTrimPrefix(r, "/api/v1/office/portfolio/graph/subtree/")
		writeOfficeJSON(w, http.StatusOK, officeRust("kogi_portfolio_subtree",
			map[string]string{"id": id}, map[string]interface{}{"root": id, "subtree": []interface{}{}}))
	})
	mux.HandleFunc("/api/v1/office/portfolio/graph/dependencies/", func(w http.ResponseWriter, r *http.Request) {
		id := officeTrimPrefix(r, "/api/v1/office/portfolio/graph/dependencies/")
		writeOfficeJSON(w, http.StatusOK, officeRust("kogi_portfolio_transitive_dependencies",
			map[string]string{"id": id}, map[string]interface{}{"root": id, "dependencies": []interface{}{}}))
	})
	mux.HandleFunc("/api/v1/office/portfolio/graph/order", func(w http.ResponseWriter, r *http.Request) {
		writeOfficeJSON(w, http.StatusOK, officeRust("kogi_portfolio_dependency_order", nil,
			map[string]interface{}{"order": []interface{}{}}))
	})

	// Governance
	mux.HandleFunc("/api/v1/office/portfolio/governance/policy/attach", func(w http.ResponseWriter, r *http.Request) {
		officeProxyPortfolio(w, r)
	})
	mux.HandleFunc("/api/v1/office/portfolio/governance/policy/detach", func(w http.ResponseWriter, r *http.Request) {
		officeProxyPortfolio(w, r)
	})
	mux.HandleFunc("/api/v1/office/portfolio/governance/approval/request", func(w http.ResponseWriter, r *http.Request) {
		officeProxyPortfolioAndPublish(w, r, "office.portfolio.governance.approval.requested", `{}`)
	})
	mux.HandleFunc("/api/v1/office/portfolio/governance/approval/", func(w http.ResponseWriter, r *http.Request) {
		if r.Method == http.MethodPost && strings.HasSuffix(r.URL.Path, "/resolve") {
			officeProxyPortfolioAndPublish(w, r, "office.portfolio.governance.approval.resolved", `{}`)
		} else {
			officeProxyPortfolio(w, r)
		}
	})
	mux.HandleFunc("/api/v1/office/portfolio/governance/resource/allocate", func(w http.ResponseWriter, r *http.Request) {
		officeProxyPortfolioAndPublish(w, r, "office.portfolio.governance.resource.allocated", `{}`)
	})
	mux.HandleFunc("/api/v1/office/portfolio/governance/resource/consume", func(w http.ResponseWriter, r *http.Request) {
		if r.Method == http.MethodPost {
			officeProxyPortfolioAndPublish(w, r, "office.portfolio.governance.resource.consumed", `{}`)
		} else {
			officeProxyPortfolio(w, r)
		}
	})
	mux.HandleFunc("/api/v1/office/portfolio/governance/resource/overruns", func(w http.ResponseWriter, r *http.Request) {
		writeOfficeJSON(w, http.StatusOK, officeRust("kogi_portfolio_overrun_allocations", nil,
			map[string]interface{}{"overruns": []interface{}{}}))
	})
	mux.HandleFunc("/api/v1/office/portfolio/governance/resource/", func(w http.ResponseWriter, r *http.Request) {
		id := officeTrimPrefix(r, "/api/v1/office/portfolio/governance/resource/")
		res := officeRust("kogi_portfolio_get_resource_allocation",
			map[string]string{"component_id": id}, nil)
		if res == nil {
			writeOfficeJSON(w, http.StatusNotFound, map[string]string{"error": "allocation not found"})
			return
		}
		writeOfficeJSON(w, http.StatusOK, res)
	})

	// Computational models
	mux.HandleFunc("/api/v1/office/portfolio/models/health/", func(w http.ResponseWriter, r *http.Request) {
		id := officeTrimPrefix(r, "/api/v1/office/portfolio/models/health/")
		res := officeRust("kogi_portfolio_compute_health", map[string]string{"portfolio_id": id}, nil)
		if res == nil {
			writeOfficeJSON(w, http.StatusNotFound, map[string]string{"error": "not found"})
			return
		}
		go officePublish("office.portfolio.model.computed",
			fmt.Sprintf(`{"model":"portfolio_health","id":%q}`, id), nil)
		writeOfficeJSON(w, http.StatusOK, res)
	})
	mux.HandleFunc("/api/v1/office/portfolio/models/project", func(w http.ResponseWriter, r *http.Request) {
		officeProxyPortfolio(w, r)
	})
	mux.HandleFunc("/api/v1/office/portfolio/models/program", func(w http.ResponseWriter, r *http.Request) {
		officeProxyPortfolio(w, r)
	})
	mux.HandleFunc("/api/v1/office/portfolio/models/subportfolio/", func(w http.ResponseWriter, r *http.Request) {
		id := officeTrimPrefix(r, "/api/v1/office/portfolio/models/subportfolio/")
		writeOfficeJSON(w, http.StatusOK, officeRust("kogi_portfolio_compute_subportfolio_rollup",
			map[string]string{"subportfolio_id": id}, map[string]interface{}{"error": "not found"}))
	})
	mux.HandleFunc("/api/v1/office/portfolio/models/resource/", func(w http.ResponseWriter, r *http.Request) {
		id := officeTrimPrefix(r, "/api/v1/office/portfolio/models/resource/")
		writeOfficeJSON(w, http.StatusOK, officeRust("kogi_portfolio_compute_resource_utilisation",
			map[string]string{"resource_id": id}, map[string]interface{}{"error": "not found"}))
	})
	mux.HandleFunc("/api/v1/office/portfolio/models/asset", func(w http.ResponseWriter, r *http.Request) {
		officeProxyPortfolio(w, r)
	})
	mux.HandleFunc("/api/v1/office/portfolio/models/artifact/", func(w http.ResponseWriter, r *http.Request) {
		officeProxyPortfolio(w, r)
	})
	mux.HandleFunc("/api/v1/office/portfolio/models/binder/", func(w http.ResponseWriter, r *http.Request) {
		officeProxyPortfolio(w, r)
	})
	mux.HandleFunc("/api/v1/office/portfolio/models/book", func(w http.ResponseWriter, r *http.Request) {
		officeProxyPortfolio(w, r)
	})
	mux.HandleFunc("/api/v1/office/portfolio/models/folder/", func(w http.ResponseWriter, r *http.Request) {
		officeProxyPortfolio(w, r)
	})
	mux.HandleFunc("/api/v1/office/portfolio/models/record/", func(w http.ResponseWriter, r *http.Request) {
		id := officeTrimPrefix(r, "/api/v1/office/portfolio/models/record/")
		writeOfficeJSON(w, http.StatusOK, officeRust("kogi_portfolio_compute_record_integrity",
			map[string]string{"record_id": id}, map[string]interface{}{"error": "not found"}))
	})

	// Components — type-specific before /{id}
	mux.HandleFunc("/api/v1/office/portfolio/components/type/", func(w http.ResponseWriter, r *http.Request) {
		ct := officeTrimPrefix(r, "/api/v1/office/portfolio/components/type/")
		writeOfficeJSON(w, http.StatusOK, map[string]interface{}{
			"type":       ct,
			"components": officeRust("kogi_portfolio_components_by_type", map[string]string{"component_type": ct}, []interface{}{}),
		})
	})
	mux.HandleFunc("/api/v1/office/portfolio/components", func(w http.ResponseWriter, r *http.Request) {
		if r.Method == http.MethodPost {
			bodyBytes, _ := io.ReadAll(r.Body)
			defer r.Body.Close()
			r.Body = io.NopCloser(bytes.NewReader(bodyBytes))
			var partial struct {
				Name          string `json:"name"`
				ComponentType string `json:"component_type"`
			}
			_ = jsonUnmarshal(bodyBytes, &partial)
			officeProxyPortfolioAndPublish(w, r,
				"office.portfolio.item.created",
				`{"name":%q,"type":%q}`, partial.Name, partial.ComponentType)
		} else {
			writeOfficeJSON(w, http.StatusOK, map[string]interface{}{
				"components": officeRust("kogi_portfolio_all_components", nil, []interface{}{}),
			})
		}
	})
	mux.HandleFunc("/api/v1/office/portfolio/components/", func(w http.ResponseWriter, r *http.Request) {
		id := strings.TrimPrefix(officeTrimPrefix(r, "/api/v1/office/portfolio/components/"), "/")
		switch r.Method {
		case http.MethodGet:
			res := officeRust("kogi_portfolio_get_component", map[string]string{"id": id}, nil)
			if res == nil {
				writeOfficeJSON(w, http.StatusNotFound, map[string]string{"error": "not found"})
				return
			}
			writeOfficeJSON(w, http.StatusOK, res)
		case http.MethodPut:
			officeProxyPortfolioAndPublish(w, r, "office.portfolio.component.updated", `{"id":%q}`, id)
		case http.MethodDelete:
			officeProxyPortfolioAndPublish(w, r, "office.portfolio.component.removed", `{"id":%q}`, id)
		default:
			officeProxyPortfolio(w, r)
		}
	})

	// Portfolio overview catch-all
	mux.HandleFunc("/api/v1/office/portfolio", func(w http.ResponseWriter, r *http.Request) {
		if r.URL.Path == "/api/v1/office/portfolio" || r.URL.Path == "/api/v1/office/portfolio/" {
			writeOfficeJSON(w, http.StatusOK, officeRust("kogi_office_portfolio_snapshot", nil,
				map[string]interface{}{"view": "portfolio"}))
			return
		}
		http.NotFound(w, r)
	})

	// ── Timeline ──────────────────────────────────────────────────────────
	mux.HandleFunc("/api/v1/office/timeline", func(w http.ResponseWriter, r *http.Request) {
		writeOfficeJSON(w, http.StatusOK, officeRust("kogi_office_timeline", nil, map[string]interface{}{
			"view": "timeline",
			"calendars": []map[string]interface{}{
				{"id": "cal-personal", "name": "Personal Calendar", "events": 14},
				{"id": "cal-work", "name": "Work Calendar", "events": 26},
				{"id": "cal-community", "name": "Community Calendar", "events": 9},
			},
		}))
	})

	mux.HandleFunc("/api/v1/office/timeline/events", func(w http.ResponseWriter, r *http.Request) {
		if r.Method != http.MethodPost {
			writeOfficeJSON(w, http.StatusMethodNotAllowed, map[string]string{"error": "method_not_allowed"})
			return
		}
		var body interface{}
		_ = decodeOfficeBody(r, &body)
		result := officeDLL("kogi_office_create_timeline_event", body,
			"office.timeline.updated", `{"action":"event_created"}`)
		if result == nil {
			result = map[string]interface{}{"ok": true}
		}
		writeOfficeJSON(w, http.StatusCreated, result)
	})

	// ── Workspace ─────────────────────────────────────────────────────────
	mux.HandleFunc("/api/v1/office/workspace", func(w http.ResponseWriter, r *http.Request) {
		writeOfficeJSON(w, http.StatusOK, officeRust("kogi_office_workspace", nil, map[string]interface{}{
			"view":    "workspace",
			"domains": []string{"personal_work", "operations", "tactics", "strategy", "governance"},
		}))
	})

	mux.HandleFunc("/api/v1/office/workspace/stories", func(w http.ResponseWriter, r *http.Request) {
		if r.Method != http.MethodPost {
			writeOfficeJSON(w, http.StatusMethodNotAllowed, map[string]string{"error": "method_not_allowed"})
			return
		}
		var body interface{}
		_ = decodeOfficeBody(r, &body)
		result := officeDLL("kogi_office_create_workspace_story", body,
			"office.workspace.story.created", `{"action":"story_created"}`)
		if result == nil {
			result = map[string]interface{}{"ok": true}
		}
		writeOfficeJSON(w, http.StatusCreated, result)
	})

	// ── Assistant ─────────────────────────────────────────────────────────
	mux.HandleFunc("/api/v1/office/assistant", func(w http.ResponseWriter, r *http.Request) {
		writeOfficeJSON(w, http.StatusOK, officeRust("kogi_office_assistant", nil, map[string]interface{}{
			"view":         "assistant",
			"assistant_id": "office-assistant-001",
		}))
	})

	mux.HandleFunc("/api/v1/office/assistant/subscriptions", func(w http.ResponseWriter, r *http.Request) {
		if r.Method != http.MethodPost {
			writeOfficeJSON(w, http.StatusMethodNotAllowed, map[string]string{"error": "method_not_allowed"})
			return
		}
		var body struct {
			Topic string `json:"topic"`
		}
		_ = decodeOfficeBody(r, &body)
		result := officeDLL("kogi_office_create_assistant_subscription",
			map[string]string{"topic": body.Topic},
			"office.assistant.subscription.active",
			fmt.Sprintf(`{"topic":%q}`, body.Topic))
		if result == nil {
			result = map[string]interface{}{"ok": true, "topic": body.Topic}
		}
		writeOfficeJSON(w, http.StatusCreated, result)
	})

	// ── Office pub/sub introspection ──────────────────────────────────────
	mux.HandleFunc("/api/v1/office/pubsub/history", func(w http.ResponseWriter, r *http.Request) {
		officeEventMu.RLock()
		events := make([]officeEventEntry, len(officeEventLog))
		copy(events, officeEventLog)
		officeEventMu.RUnlock()
		n := len(events)
		if n > 200 {
			events = events[n-200:]
		}
		for i, j := 0, len(events)-1; i < j; i, j = i+1, j-1 {
			events[i], events[j] = events[j], events[i]
		}
		writeOfficeJSON(w, http.StatusOK, map[string]interface{}{"events": events, "count": len(events)})
	})

	mux.HandleFunc("/api/v1/office/pubsub/topics", func(w http.ResponseWriter, r *http.Request) {
		officeEventMu.RLock()
		topicCounts := map[string]int{}
		for _, e := range officeEventLog {
			topicCounts[e.Topic]++
		}
		total := len(officeEventLog)
		officeEventMu.RUnlock()
		writeOfficeJSON(w, http.StatusOK, map[string]interface{}{
			"topic_counts": topicCounts, "total_events": total,
			"published_topics": []string{
				"office.dashboard.refresh", "office.timeline.updated",
				"office.workspace.story.created", "office.assistant.subscription.active",
				"office.portfolio.item.created", "office.portfolio.component.updated",
				"office.portfolio.component.removed", "office.portfolio.graph.changed",
				"office.portfolio.snapshot.saved", "office.portfolio.checkpoint.created",
				"office.portfolio.governance.approval.requested",
				"office.portfolio.governance.resource.allocated",
				"office.portfolio.model.computed",
			},
			"subscribed_prefixes": []string{"portfolio.", "ims.", "exchange.trade"},
			"subscribed_topics": []string{
				"portfolio.item.created", "portfolio.component.updated",
				"portfolio.component.removed", "portfolio.graph.changed",
				"portfolio.snapshot.saved", "portfolio.checkpoint.created",
				"portfolio.governance.resource.allocated", "portfolio.governance.resource.consumed",
				"portfolio.governance.approval.requested", "portfolio.governance.approval.resolved",
				"portfolio.model.computed", "portfolio.health.updated",
				"ims.profile.updated", "exchange.trade.executed",
			},
		})
	})

	mux.HandleFunc("/api/v1/office/pubsub/dead-letters", func(w http.ResponseWriter, r *http.Request) {
		officeProxyGateway(w, "/api/v1/gateway/pubsub/dead-letters?limit=20",
			map[string]interface{}{"dead_letters": []interface{}{}})
	})

	mux.HandleFunc("/api/v1/office/pubsub/replay", func(w http.ResponseWriter, r *http.Request) {
		from := r.URL.Query().Get("from")
		to := r.URL.Query().Get("to")
		topic := r.URL.Query().Get("topic")
		q := "prefix=office."
		if topic != "" {
			q = "topic=" + topic
		}
		if from != "" {
			q += "&from=" + from
		}
		if to != "" {
			q += "&to=" + to
		}
		officeProxyGateway(w, "/api/v1/gateway/pubsub/replay?"+q,
			map[string]interface{}{"events": []interface{}{}})
	})

	mux.HandleFunc("/api/v1/office/pubsub/metrics", func(w http.ResponseWriter, r *http.Request) {
		officeProxyGateway(w, "/api/v1/gateway/pubsub/metrics",
			map[string]interface{}{"error": "gateway unavailable"})
	})

	// ── Mesh ──────────────────────────────────────────────────────────────
	mux.HandleFunc("/api/v1/office/mesh", func(w http.ResponseWriter, r *http.Request) {
		officeProxyGateway(w, "/api/v1/gateway/components/id/"+officeServiceID,
			map[string]interface{}{"error": "gateway unavailable"})
	})

	mux.HandleFunc("/api/v1/office/mesh/messages", func(w http.ResponseWriter, r *http.Request) {
		limit := r.URL.Query().Get("limit")
		if limit == "" {
			limit = "50"
		}
		officeProxyGateway(w, "/api/v1/gateway/network/history?source="+officeServiceID+"&limit="+limit,
			map[string]interface{}{"messages": []interface{}{}})
	})

	mux.HandleFunc("/api/v1/office/mesh/routes", func(w http.ResponseWriter, r *http.Request) {
		officeProxyGateway(w, "/api/v1/gateway/portfolio/routes",
			map[string]interface{}{"error": "gateway unavailable"})
	})

	log.Printf("%s listening on %s", officeServiceName, addr)
	log.Fatal(http.ListenAndServe(addr, mux))
}

// jsonUnmarshal delegates to the standard library unmarshal, available via the
// shared "encoding/json" import in utility.go (same package).
func jsonUnmarshal(b []byte, v interface{}) error {
	return json.Unmarshal(b, v)
}
