package main

// office_service.go
//
// HTTP service for the Kogi Office module.  Delegates all PortfolioSystem
// operations to the portfolio-service (which owns the Rust bridge) and
// publishes/subscribes events through the gateway bus.
//
// Relationship to portfolio_service.go
// -------------------------------------
//   office_service is the Office-layer orchestrator.  It:
//     1. Calls portfolio-service REST endpoints for portfolio mutations.
//     2. Calls the Rust kogi-office-system binary for office-specific views
//        (dashboard, timeline, workspace, assistant).
//     3. Publishes composite events (e.g. "office.portfolio.item.created")
//        that carry both office and portfolio context.
//     4. Subscribes (via gateway) to portfolio events so it can update the
//        office dashboard and assistant automatically.
//
// Endpoints (all under /api/v1/office/…):
//   GET  /health
//   GET  /runtime
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
//   POST /api/v1/office/portfolio/books          create book
//   POST /api/v1/office/portfolio/active         set active portfolio
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
//   GET  /api/v1/office/portfolio/query          ?pql=…
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
//   GET  /api/v1/office/pubsub/history           recent office events
//   GET  /api/v1/office/pubsub/topics

import (
	"bytes"
	"context"
	"encoding/json"
	"fmt"
	"io"
	"log"
	"net/http"
	"os"
	"os/exec"
	"path/filepath"
	"runtime"
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

// ── In-process event log (recent office events) ───────────────────────────────

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

// ── Rust bridge (office-system binary) ───────────────────────────────────────

type officeRustReq struct {
	Action  string      `json:"action"`
	Payload interface{} `json:"payload,omitempty"`
}

func officeCallRust(action string, payload interface{}) (interface{}, error) {
	bin, err := officeResolveBinary()
	if err != nil {
		return nil, err
	}
	envelope, _ := json.Marshal(officeRustReq{Action: action, Payload: payload})
	ctx, cancel := context.WithTimeout(context.Background(), 5*time.Second)
	defer cancel()
	cmd := exec.CommandContext(ctx, bin, "--request", string(envelope))
	out, err := cmd.CombinedOutput()
	if err != nil {
		return nil, fmt.Errorf("office rust action=%s: %w – %s", action, err, strings.TrimSpace(string(out)))
	}
	var result interface{}
	if err := json.Unmarshal(out, &result); err != nil {
		return nil, fmt.Errorf("office rust parse action=%s: %w", action, err)
	}
	return result, nil
}

func officeRust(action string, payload interface{}, fallback interface{}) interface{} {
	r, err := officeCallRust(action, payload)
	if err != nil {
		log.Printf("[office-svc] rust action=%s err=%v", action, err)
		return fallback
	}
	return r
}

func officeResolveBinary() (string, error) {
	if p := os.Getenv("KOGI_OFFICE_SYSTEM_BIN"); p != "" {
		return p, nil
	}
	if p, err := exec.LookPath("kogi-office-system"); err == nil {
		return p, nil
	}
	if root, ok := officeFindRepoRoot(); ok {
		exe := "kogi-office-system"
		if runtime.GOOS == "windows" {
			exe += ".exe"
		}
		for _, c := range []string{
			filepath.Join(root, "kogi-modules", "office", "target", "debug", exe),
			filepath.Join(root, "kogi-modules", "office", "target", "release", exe),
			filepath.Join(root, "kogi-modules", "office", exe),
		} {
			if officeFileExists(c) {
				return c, nil
			}
		}
	}
	return "", fmt.Errorf("kogi-office-system not found; set KOGI_OFFICE_SYSTEM_BIN")
}

// ── Portfolio-service proxy ───────────────────────────────────────────────────
//
// officeProxyPortfolio forwards a request to portfolio-service, substituting
// the /api/v1/office/portfolio/ prefix with /api/v1/portfolio/.  The response
// is written directly to w.

func officeProxyPortfolio(w http.ResponseWriter, r *http.Request) {
	// Rewrite path: /api/v1/office/portfolio/… → /api/v1/portfolio/…
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

// officeProxyPortfolioAndPublish wraps officeProxyPortfolio and, on success,
// also fires a gateway event so the office dashboard can react.
func officeProxyPortfolioAndPublish(w http.ResponseWriter, r *http.Request, topic, payloadFmt string, args ...interface{}) {
	// Buffer the proxy response so we can inspect the status code before publishing.
	rec := &responseRecorder{header: make(http.Header), code: http.StatusOK}
	officeProxyPortfolio(rec, r)

	// Forward buffered response to actual ResponseWriter.
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
		go recordOfficeEvent(topic, payload, officeServiceID)
	}
}

// responseRecorder captures an http.ResponseWriter for buffering.
type responseRecorder struct {
	header http.Header
	code   int
	body   []byte
}

func (r *responseRecorder) Header() http.Header        { return r.header }
func (r *responseRecorder) WriteHeader(code int)        { r.code = code }
func (r *responseRecorder) Write(b []byte) (int, error) { r.body = append(r.body, b...); return len(b), nil }

// ── Gateway pub/sub ───────────────────────────────────────────────────────────

func officePublish(topic, payload string, meta map[string]string) {
	body, _ := json.Marshal(map[string]interface{}{
		"topic": topic, "payload": payload,
		"source": officeServiceID, "metadata": meta,
	})
	ctx, cancel := context.WithTimeout(context.Background(), 2*time.Second)
	defer cancel()
	req, _ := http.NewRequestWithContext(ctx, http.MethodPost,
		officeGateway+"/api/v1/gateway/pubsub/publish", bytes.NewReader(body))
	req.Header.Set("Content-Type", "application/json")
	resp, err := http.DefaultClient.Do(req)
	if err != nil {
		log.Printf("[office-svc] publish topic=%s err=%v", topic, err)
		return
	}
	defer resp.Body.Close()
	recordOfficeEvent(topic, payload, officeServiceID)
}

// officeSubscribeGateway long-polls the gateway for a set of exact topics and
// calls handler whenever new events arrive.  Kept for backward compatibility.
func officeSubscribeGateway(topics []string, handler func(topic, payload string)) {
	for {
		for _, topic := range topics {
			func() {
				ctx, cancel := context.WithTimeout(context.Background(), 5*time.Second)
				defer cancel()
				req, _ := http.NewRequestWithContext(ctx, http.MethodGet,
					fmt.Sprintf("%s/api/v1/gateway/pubsub/history?limit=5&topic=%s", officeGateway, topic), nil)
				resp, err := http.DefaultClient.Do(req)
				if err != nil {
					return
				}
				defer resp.Body.Close()
				var result struct {
					Events []struct {
						Topic   string `json:"topic"`
						Payload string `json:"payload"`
					} `json:"events"`
				}
				if err := json.NewDecoder(resp.Body).Decode(&result); err != nil {
					return
				}
				for _, e := range result.Events {
					handler(e.Topic, e.Payload)
				}
			}()
		}
		time.Sleep(3 * time.Second)
	}
}

// officeSubscribeGatewayPrefix polls the gateway /pubsub/history/prefix endpoint
// for each prefix, delivering all matching events to handler.  This is the
// preferred method: new portfolio sub-topics (e.g. portfolio.crdt.*) are
// automatically included without code changes.
func officeSubscribeGatewayPrefix(prefixes []string, handler func(topic, payload string)) {
	for {
		for _, prefix := range prefixes {
			func() {
				ctx, cancel := context.WithTimeout(context.Background(), 5*time.Second)
				defer cancel()
				req, _ := http.NewRequestWithContext(ctx, http.MethodGet,
					fmt.Sprintf("%s/api/v1/gateway/pubsub/history/prefix?prefix=%s&limit=10",
						officeGateway, prefix), nil)
				resp, err := http.DefaultClient.Do(req)
				if err != nil {
					return
				}
				defer resp.Body.Close()
				var result struct {
					Events []struct {
						Topic   string `json:"topic"`
						Payload string `json:"payload"`
					} `json:"events"`
				}
				if err := json.NewDecoder(resp.Body).Decode(&result); err != nil {
					return
				}
				for _, e := range result.Events {
					handler(e.Topic, e.Payload)
				}
			}()
		}
		time.Sleep(3 * time.Second)
	}
}

// officeReplayGateway fetches a timestamp-windowed replay from the gateway for
// a topic or prefix and appends results to the in-process event log.
func officeReplayGateway(topic, prefix, from, to string) {
	q := ""
	switch {
	case prefix != "":
		q = fmt.Sprintf("prefix=%s&from=%s&to=%s", prefix, from, to)
	case topic != "":
		q = fmt.Sprintf("topic=%s&from=%s&to=%s", topic, from, to)
	default:
		return
	}
	ctx, cancel := context.WithTimeout(context.Background(), 5*time.Second)
	defer cancel()
	req, _ := http.NewRequestWithContext(ctx, http.MethodGet,
		officeGateway+"/api/v1/gateway/pubsub/replay?"+q, nil)
	resp, err := http.DefaultClient.Do(req)
	if err != nil {
		return
	}
	defer resp.Body.Close()
	var result struct {
		Events []struct {
			Topic   string `json:"topic"`
			Payload string `json:"payload"`
			Source  string `json:"source"`
		} `json:"events"`
	}
	if err := json.NewDecoder(resp.Body).Decode(&result); err != nil {
		return
	}
	for _, e := range result.Events {
		recordOfficeEvent(e.Topic, e.Payload, e.Source)
	}
}

func officeRegisterWithGateway(selfEndpoint string) {
	body, _ := json.Marshal(map[string]interface{}{
		"id": officeServiceID, "kind": "service",
		"endpoint": selfEndpoint, "health_path": "/health",
		"network_manager": "kogi-go-network", "status": "active",
		"metadata": map[string]string{
			"publishes": strings.Join([]string{
				"office.dashboard.refresh", "office.timeline.updated",
				"office.workspace.story.created", "office.assistant.subscription.active",
				"office.portfolio.item.created", "office.portfolio.component.updated",
				"office.portfolio.component.removed", "office.portfolio.graph.changed",
				"office.portfolio.snapshot.saved", "office.portfolio.checkpoint.created",
				"office.portfolio.governance.approval.requested",
				"office.portfolio.governance.resource.allocated",
				"office.portfolio.model.computed",
			}, ","),
			"subscribes": strings.Join([]string{
				"portfolio.item.created", "portfolio.component.updated",
				"portfolio.component.removed", "portfolio.graph.changed",
				"portfolio.snapshot.saved", "portfolio.checkpoint.created",
				"portfolio.governance.resource.allocated", "portfolio.governance.resource.consumed",
				"portfolio.governance.approval.requested", "portfolio.governance.approval.resolved",
				"portfolio.model.computed", "portfolio.health.updated",
				"ims.profile.updated", "exchange.trade.executed",
			}, ","),
		},
	})
	ctx, cancel := context.WithTimeout(context.Background(), 3*time.Second)
	defer cancel()
	req, _ := http.NewRequestWithContext(ctx, http.MethodPost,
		officeGateway+"/api/v1/gateway/components/register", bytes.NewReader(body))
	req.Header.Set("Content-Type", "application/json")
	resp, err := http.DefaultClient.Do(req)
	if err != nil {
		log.Printf("[office-svc] gateway registration err=%v", err)
		return
	}
	defer resp.Body.Close()
	log.Printf("[office-svc] registered with gateway %s", officeGateway)

	// Register prefix subscriptions for the two namespaces we care about.
	// Using prefix subscriptions means new portfolio topics (e.g. future CRDT
	// events) are caught automatically without code changes.
	for _, sub := range []struct{ prefix, consumer string }{
		{"portfolio.", officeServiceID},
		{"ims.", officeServiceID},
		{"exchange.trade", officeServiceID},
	} {
		officeGatewaySubscribePrefix(sub.prefix, sub.consumer)
	}
}

// officeGatewaySubscribePrefix registers a prefix subscription on the gateway.
// The returned subscription ID is discarded here — the office service uses
// polling rather than push delivery.
func officeGatewaySubscribePrefix(prefix, consumer string) {
	body, _ := json.Marshal(map[string]string{"prefix": prefix, "consumer": consumer})
	ctx, cancel := context.WithTimeout(context.Background(), 2*time.Second)
	defer cancel()
	req, _ := http.NewRequestWithContext(ctx, http.MethodPost,
		officeGateway+"/api/v1/gateway/pubsub/subscribe/prefix", bytes.NewReader(body))
	req.Header.Set("Content-Type", "application/json")
	resp, err := http.DefaultClient.Do(req)
	if err != nil {
		log.Printf("[office-svc] prefix subscribe prefix=%s err=%v", prefix, err)
		return
	}
	defer resp.Body.Close()
}

// officeReportHealth sends a health record to the gateway mesh registry.
func officeReportHealth(healthy bool, note string) {
	body, _ := json.Marshal(map[string]interface{}{
		"id":      officeServiceID,
		"healthy": healthy,
		"note":    note,
	})
	ctx, cancel := context.WithTimeout(context.Background(), 2*time.Second)
	defer cancel()
	req, _ := http.NewRequestWithContext(ctx, http.MethodPost,
		officeGateway+"/api/v1/gateway/components/health", bytes.NewReader(body))
	req.Header.Set("Content-Type", "application/json")
	resp, err := http.DefaultClient.Do(req)
	if err != nil {
		log.Printf("[office-svc] health report err=%v", err)
		return
	}
	defer resp.Body.Close()
}

// officeHealthLoop reports health every 30 seconds.
func officeHealthLoop() {
	for {
		time.Sleep(30 * time.Second)
		_, err := officeResolveBinary()
		officeReportHealth(err == nil, "")
	}
}

// officeDeadLetterMonitor polls the gateway dead-letter queue for office.*
// events that had no subscribers and logs them.
func officeDeadLetterMonitor() {
	for {
		time.Sleep(60 * time.Second)
		ctx, cancel := context.WithTimeout(context.Background(), 3*time.Second)
		req, _ := http.NewRequestWithContext(ctx, http.MethodGet,
			officeGateway+"/api/v1/gateway/pubsub/dead-letters?limit=20", nil)
		resp, err := http.DefaultClient.Do(req)
		cancel()
		if err != nil {
			continue
		}
		var result struct {
			DeadLetters []struct {
				Topic string `json:"topic"`
				ID    string `json:"id"`
			} `json:"dead_letters"`
		}
		if err := json.NewDecoder(resp.Body).Decode(&result); err != nil {
			resp.Body.Close()
			continue
		}
		resp.Body.Close()
		for _, dl := range result.DeadLetters {
			if strings.HasPrefix(dl.Topic, "office.") {
				log.Printf("[office-svc] dead-letter id=%s topic=%s", dl.ID, dl.Topic)
			}
		}
	}
}

// ── Misc helpers ──────────────────────────────────────────────────────────────

// officeProxyGateway does a GET to the gateway at path and writes the response
// body directly.  On any error it writes fallback as JSON.
func officeProxyGateway(w http.ResponseWriter, path string, fallback interface{}) {
	ctx, cancel := context.WithTimeout(context.Background(), 3*time.Second)
	defer cancel()
	req, _ := http.NewRequestWithContext(ctx, http.MethodGet, officeGateway+path, nil)
	resp, err := http.DefaultClient.Do(req)
	if err != nil {
		writeOfficeJSON(w, http.StatusOK, fallback)
		return
	}
	defer resp.Body.Close()
	w.Header().Set("Content-Type", "application/json")
	w.WriteHeader(resp.StatusCode)
	_, _ = io.Copy(w, resp.Body)
}

func writeOfficeJSON(w http.ResponseWriter, status int, payload interface{}) {
	w.Header().Set("Content-Type", "application/json")
	w.WriteHeader(status)
	_ = json.NewEncoder(w).Encode(payload)
}

func decodeOfficeBody(r *http.Request, dst interface{}) error {
	defer r.Body.Close()
	b, _ := io.ReadAll(r.Body)
	if err := json.Unmarshal(b, dst); err != nil {
		return fmt.Errorf("invalid JSON: %w", err)
	}
	return nil
}

func officeTrimPrefix(r *http.Request, prefix string) string {
	return strings.TrimPrefix(r.URL.Path, prefix)
}

func resolveOfficeAddr(defaultPort, envKey string) string {
	if p := os.Getenv(envKey); p != "" {
		return officeToListenAddr(p)
	}
	if p := os.Getenv("KOGI_PORT"); p != "" {
		return officeToListenAddr(p)
	}
	return ":" + defaultPort
}

func officeToListenAddr(port string) string {
	if strings.HasPrefix(port, ":") {
		return port
	}
	return ":" + port
}

func officeFileExists(p string) bool {
	info, err := os.Stat(p)
	return err == nil && !info.IsDir()
}

func officeFindRepoRoot() (string, bool) {
	cur, _ := os.Getwd()
	for i := 0; i < 8; i++ {
		if _, err := os.Stat(filepath.Join(cur, "kogi-modules")); err == nil {
			return cur, true
		}
		if _, err := os.Stat(filepath.Join(cur, "go.work")); err == nil {
			return cur, true
		}
		parent := filepath.Dir(cur)
		if parent == cur {
			break
		}
		cur = parent
	}
	return "", false
}

func officeResolveBinaryHint() string {
	if p := os.Getenv("KOGI_OFFICE_SYSTEM_BIN"); p != "" {
		return p
	}
	if p, err := exec.LookPath("kogi-office-system"); err == nil {
		return p
	}
	return ""
}

// ── main ──────────────────────────────────────────────────────────────────────

func main() {
	mux := http.NewServeMux()

	addr := resolveOfficeAddr(officeDefaultPort, "KOGI_OFFICE_PORT")
	selfEndpoint := fmt.Sprintf("http://127.0.0.1%s", addr)

	// Registration + inbound subscription pump
	go func() {
		time.Sleep(600 * time.Millisecond)
		officeRegisterWithGateway(selfEndpoint)
		// Poll using the gateway's prefix history endpoint — catches all current
		// and future portfolio.* topics without enumerating them individually.
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
		writeOfficeJSON(w, http.StatusOK, officeRust("overview", nil, map[string]interface{}{
			"module":      "kogi.office",
			"application": "Kogi Office",
			"service":     officeServiceName,
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
			writeOfficeJSON(w, http.StatusOK, officeRust("dashboard", nil, map[string]interface{}{
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
		result := officeRust("ack_dashboard_notification",
			map[string]string{"notification_id": body.NotificationID},
			map[string]interface{}{"ok": true, "message": "acknowledged"})
		go officePublish("office.dashboard.refresh",
			fmt.Sprintf(`{"notification_id":%q,"action":"acknowledged"}`, body.NotificationID), nil)
		writeOfficeJSON(w, http.StatusOK, result)
	})

	// ── Portfolio proxy — ALL /api/v1/office/portfolio/… routes ──────────
	//
	// The office service proxies every portfolio sub-path to portfolio-service,
	// adding office-level pub/sub publishing for mutations.

	// Proxy: snapshot
	mux.HandleFunc("/api/v1/office/portfolio/snapshot", func(w http.ResponseWriter, r *http.Request) {
		officeProxyPortfolio(w, r)
	})
	// Proxy: metadata
	mux.HandleFunc("/api/v1/office/portfolio/metadata", func(w http.ResponseWriter, r *http.Request) {
		officeProxyPortfolio(w, r)
	})
	// Proxy: query
	mux.HandleFunc("/api/v1/office/portfolio/query", func(w http.ResponseWriter, r *http.Request) {
		officeProxyPortfolio(w, r)
	})
	// Proxy: events
	mux.HandleFunc("/api/v1/office/portfolio/events", func(w http.ResponseWriter, r *http.Request) {
		officeProxyPortfolio(w, r)
	})
	// Proxy: snapshots list / save
	mux.HandleFunc("/api/v1/office/portfolio/snapshots", func(w http.ResponseWriter, r *http.Request) {
		if r.Method == http.MethodPost {
			officeProxyPortfolioAndPublish(w, r, "office.portfolio.snapshot.saved", `{}`)
		} else {
			officeProxyPortfolio(w, r)
		}
	})
	mux.HandleFunc("/api/v1/office/portfolio/snapshots/", func(w http.ResponseWriter, r *http.Request) {
		officeProxyPortfolio(w, r)
	})
	// Proxy: checkpoints list / save
	mux.HandleFunc("/api/v1/office/portfolio/checkpoints", func(w http.ResponseWriter, r *http.Request) {
		if r.Method == http.MethodPost {
			officeProxyPortfolioAndPublish(w, r, "office.portfolio.checkpoint.created", `{}`)
		} else {
			officeProxyPortfolio(w, r)
		}
	})
	mux.HandleFunc("/api/v1/office/portfolio/checkpoints/", func(w http.ResponseWriter, r *http.Request) {
		officeProxyPortfolio(w, r)
	})
	// Proxy: books
	mux.HandleFunc("/api/v1/office/portfolio/books", func(w http.ResponseWriter, r *http.Request) {
		if r.Method == http.MethodPost {
			officeProxyPortfolioAndPublish(w, r, "office.portfolio.item.created", `{"type":"book"}`)
		} else {
			officeProxyPortfolio(w, r)
		}
	})
	// Proxy: active portfolio
	mux.HandleFunc("/api/v1/office/portfolio/active", func(w http.ResponseWriter, r *http.Request) {
		officeProxyPortfolio(w, r)
	})
	// Proxy: graph operations
	mux.HandleFunc("/api/v1/office/portfolio/graph/hierarchy", func(w http.ResponseWriter, r *http.Request) {
		officeProxyPortfolioAndPublish(w, r, "office.portfolio.graph.changed", `{"edge":"hierarchy","method":%q}`, r.Method)
	})
	mux.HandleFunc("/api/v1/office/portfolio/graph/dependency", func(w http.ResponseWriter, r *http.Request) {
		officeProxyPortfolioAndPublish(w, r, "office.portfolio.graph.changed", `{"edge":"dependency","method":%q}`, r.Method)
	})
	mux.HandleFunc("/api/v1/office/portfolio/graph/link", func(w http.ResponseWriter, r *http.Request) {
		officeProxyPortfolioAndPublish(w, r, "office.portfolio.graph.changed", `{"edge":"link","method":%q}`, r.Method)
	})
	mux.HandleFunc("/api/v1/office/portfolio/graph/member", func(w http.ResponseWriter, r *http.Request) {
		officeProxyPortfolioAndPublish(w, r, "office.portfolio.graph.changed", `{"edge":"member","method":%q}`, r.Method)
	})
	mux.HandleFunc("/api/v1/office/portfolio/graph/subtree/", func(w http.ResponseWriter, r *http.Request) {
		officeProxyPortfolio(w, r)
	})
	mux.HandleFunc("/api/v1/office/portfolio/graph/dependencies/", func(w http.ResponseWriter, r *http.Request) {
		officeProxyPortfolio(w, r)
	})
	mux.HandleFunc("/api/v1/office/portfolio/graph/order", func(w http.ResponseWriter, r *http.Request) {
		officeProxyPortfolio(w, r)
	})
	// Proxy: governance
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
		// POST …/{id}/resolve
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
		officeProxyPortfolio(w, r)
	})
	mux.HandleFunc("/api/v1/office/portfolio/governance/resource/", func(w http.ResponseWriter, r *http.Request) {
		officeProxyPortfolio(w, r)
	})
	// Proxy: computational models
	mux.HandleFunc("/api/v1/office/portfolio/models/health/", func(w http.ResponseWriter, r *http.Request) {
		id := officeTrimPrefix(r, "/api/v1/office/portfolio/models/health/")
		officeProxyPortfolioAndPublish(w, r, "office.portfolio.model.computed",
			`{"model":"portfolio_health","id":%q}`, id)
	})
	mux.HandleFunc("/api/v1/office/portfolio/models/project", func(w http.ResponseWriter, r *http.Request) {
		officeProxyPortfolio(w, r)
	})
	mux.HandleFunc("/api/v1/office/portfolio/models/program", func(w http.ResponseWriter, r *http.Request) {
		officeProxyPortfolio(w, r)
	})
	mux.HandleFunc("/api/v1/office/portfolio/models/subportfolio/", func(w http.ResponseWriter, r *http.Request) {
		officeProxyPortfolio(w, r)
	})
	mux.HandleFunc("/api/v1/office/portfolio/models/resource/", func(w http.ResponseWriter, r *http.Request) {
		officeProxyPortfolio(w, r)
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
		officeProxyPortfolio(w, r)
	})
	// Proxy: components (type-specific must be before /{id})
	mux.HandleFunc("/api/v1/office/portfolio/components/type/", func(w http.ResponseWriter, r *http.Request) {
		officeProxyPortfolio(w, r)
	})
	mux.HandleFunc("/api/v1/office/portfolio/components", func(w http.ResponseWriter, r *http.Request) {
		if r.Method == http.MethodPost {
			// Read body for publishing; re-attach a new reader for proxy
			bodyBytes, _ := io.ReadAll(r.Body)
			defer r.Body.Close()
			r.Body = io.NopCloser(bytes.NewReader(bodyBytes))

			var partial struct {
				Name          string `json:"name"`
				ComponentType string `json:"component_type"`
			}
			_ = json.Unmarshal(bodyBytes, &partial)

			officeProxyPortfolioAndPublish(w, r,
				"office.portfolio.item.created",
				`{"name":%q,"type":%q}`, partial.Name, partial.ComponentType)
		} else {
			officeProxyPortfolio(w, r)
		}
	})
	mux.HandleFunc("/api/v1/office/portfolio/components/", func(w http.ResponseWriter, r *http.Request) {
		id := strings.TrimPrefix(officeTrimPrefix(r, "/api/v1/office/portfolio/components/"), "/")
		switch r.Method {
		case http.MethodPut:
			officeProxyPortfolioAndPublish(w, r, "office.portfolio.component.updated",
				`{"id":%q}`, id)
		case http.MethodDelete:
			officeProxyPortfolioAndPublish(w, r, "office.portfolio.component.removed",
				`{"id":%q}`, id)
		default:
			officeProxyPortfolio(w, r)
		}
	})
	// Proxy: portfolio overview (fallback catch-all for /api/v1/office/portfolio)
	mux.HandleFunc("/api/v1/office/portfolio", func(w http.ResponseWriter, r *http.Request) {
		if r.URL.Path == "/api/v1/office/portfolio" || r.URL.Path == "/api/v1/office/portfolio/" {
			officeProxyPortfolio(w, r)
			return
		}
		http.NotFound(w, r)
	})

	// ── Timeline ──────────────────────────────────────────────────────────
	mux.HandleFunc("/api/v1/office/timeline", func(w http.ResponseWriter, r *http.Request) {
		writeOfficeJSON(w, http.StatusOK, officeRust("timeline", nil, map[string]interface{}{
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
		result := officeRust("create_timeline_event", body, map[string]interface{}{"ok": true})
		go officePublish("office.timeline.updated", `{"action":"event_created"}`, nil)
		writeOfficeJSON(w, http.StatusCreated, result)
	})

	// ── Workspace ─────────────────────────────────────────────────────────
	mux.HandleFunc("/api/v1/office/workspace", func(w http.ResponseWriter, r *http.Request) {
		writeOfficeJSON(w, http.StatusOK, officeRust("workspace", nil, map[string]interface{}{
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
		result := officeRust("create_workspace_story", body, map[string]interface{}{"ok": true})
		go officePublish("office.workspace.story.created", `{"action":"story_created"}`, nil)
		writeOfficeJSON(w, http.StatusCreated, result)
	})

	// ── Assistant ─────────────────────────────────────────────────────────
	mux.HandleFunc("/api/v1/office/assistant", func(w http.ResponseWriter, r *http.Request) {
		writeOfficeJSON(w, http.StatusOK, officeRust("assistant", nil, map[string]interface{}{
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
		result := officeRust("create_assistant_subscription",
			map[string]string{"topic": body.Topic},
			map[string]interface{}{"ok": true, "topic": body.Topic})
		go officePublish("office.assistant.subscription.active",
			fmt.Sprintf(`{"topic":%q}`, body.Topic), nil)
		writeOfficeJSON(w, http.StatusCreated, result)
	})

	// ── Office pub/sub introspection ──────────────────────────────────────
	mux.HandleFunc("/api/v1/office/pubsub/history", func(w http.ResponseWriter, r *http.Request) {
		officeEventMu.RLock()
		events := make([]officeEventEntry, len(officeEventLog))
		copy(events, officeEventLog)
		officeEventMu.RUnlock()
		// Return newest first, up to 200
		n := len(events)
		if n > 200 {
			events = events[n-200:]
		}
		for i, j := 0, len(events)-1; i < j; i, j = i+1, j-1 {
			events[i], events[j] = events[j], events[i]
		}
		writeOfficeJSON(w, http.StatusOK, map[string]interface{}{
			"events": events,
			"count":  len(events),
		})
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
			"topic_counts": topicCounts,
			"total_events": total,
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

	// Dead-letters: forwards gateway /pubsub/dead-letters, filtered to office.*
	mux.HandleFunc("/api/v1/office/pubsub/dead-letters", func(w http.ResponseWriter, r *http.Request) {
		officeProxyGateway(w, "/api/v1/gateway/pubsub/dead-letters?limit=20",
			map[string]interface{}{"dead_letters": []interface{}{}})
	})

	// Replay: fetch a window of office.* events from the gateway bus history
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

	// Bus metrics: forwards gateway /pubsub/metrics
	mux.HandleFunc("/api/v1/office/pubsub/metrics", func(w http.ResponseWriter, r *http.Request) {
		officeProxyGateway(w, "/api/v1/gateway/pubsub/metrics",
			map[string]interface{}{"error": "gateway unavailable"})
	})

	// Mesh: what the gateway mesh registry knows about this service
	mux.HandleFunc("/api/v1/office/mesh", func(w http.ResponseWriter, r *http.Request) {
		officeProxyGateway(w, "/api/v1/gateway/components/id/"+officeServiceID,
			map[string]interface{}{"error": "gateway unavailable"})
	})

	// Mesh messages sent to/from this service
	mux.HandleFunc("/api/v1/office/mesh/messages", func(w http.ResponseWriter, r *http.Request) {
		limit := r.URL.Query().Get("limit")
		if limit == "" {
			limit = "50"
		}
		officeProxyGateway(w,
			"/api/v1/gateway/network/history?source="+officeServiceID+"&limit="+limit,
			map[string]interface{}{"messages": []interface{}{}})
	})

	// Office topic routes: what the gateway has wired for this service
	mux.HandleFunc("/api/v1/office/mesh/routes", func(w http.ResponseWriter, r *http.Request) {
		officeProxyGateway(w, "/api/v1/gateway/portfolio/routes",
			map[string]interface{}{"error": "gateway unavailable"})
	})

	log.Printf("%s listening on %s", officeServiceName, addr)
	log.Fatal(http.ListenAndServe(addr, mux))
}
