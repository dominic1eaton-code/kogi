package main

// portfolio_service.go
//
// HTTP service that bridges the Go network layer to the Rust PortfolioSystem
// binary (kogi-portfolio-system).  Every PortfolioSystem operation exposed by
// the Rust crate has a corresponding HTTP endpoint here.
//
// Pub/Sub integration
// -------------------
//   On startup the service registers itself with the gateway and subscribes to
//   inbound topics via a long-poll helper.  Mutations that succeed publish
//   outbound events to the gateway pub/sub bus so that office-service and
//   other consumers stay in sync.
//
// Rust bridge
// -----------
//   The service calls the Rust binary with --request <json>.  The binary reads
//   the request, executes the operation against its in-memory PortfolioSystem,
//   serialises the result, and prints it to stdout.
//
// Endpoints exposed (all under /api/v1/portfolio/…):
//   GET  /health
//   GET  /runtime
//   GET  /overview
//   GET  /snapshot                        full PortfolioSnapshot
//   GET  /metadata                        portfolio_metadata summary
//   GET  /components                      all_components
//   POST /components                      create_component
//   GET  /components/{id}
//   PUT  /components/{id}                 edit_component
//   DELETE /components/{id}              remove_component
//   GET  /components/type/{type}          components_by_type
//   POST /books                           create_book
//   POST /active                          set_active_portfolio
//   POST /graph/hierarchy                 add_hierarchy
//   DELETE /graph/hierarchy               remove_hierarchy
//   POST /graph/dependency                add_dependency
//   DELETE /graph/dependency              remove_dependency
//   POST /graph/link                      add_link
//   DELETE /graph/link                    remove_link
//   POST /graph/member                    add_member
//   DELETE /graph/member                  remove_member
//   GET  /graph/subtree/{id}              subtree
//   GET  /graph/dependencies/{id}         transitive_dependencies
//   GET  /graph/order                     dependency_order
//   GET  /snapshots                       list snapshots
//   POST /snapshots                       save_snapshot
//   POST /snapshots/{id}/restore          restore_snapshot
//   GET  /checkpoints                     list checkpoints
//   POST /checkpoints                     save_checkpoint
//   POST /checkpoints/{id}/restore        restore_checkpoint
//   GET  /query                           query_pql   (?pql=…)
//   GET  /events                          event_log
//   POST /governance/policy/attach        attach_policy
//   POST /governance/policy/detach        detach_policy
//   POST /governance/approval/request     request_approval
//   POST /governance/approval/{id}/resolve resolve_approval
//   POST /governance/resource/allocate    allocate_resource
//   POST /governance/resource/consume     record_consumption
//   GET  /governance/resource/overruns    overrun_allocations
//   GET  /governance/resource/{id}        get_resource_allocation
//   GET  /models/health/{id}              compute_portfolio_health
//   POST /models/project                  compute_project_metrics
//   POST /models/program                  compute_program_alignment
//   GET  /models/subportfolio/{id}        compute_subportfolio_rollup
//   GET  /models/resource/{id}            compute_resource_utilisation
//   POST /models/asset                    compute_asset_value
//   GET/POST /models/artifact/{id}        compute_artifact_maturity
//   GET/POST /models/binder/{id}          compute_binder_coverage
//   POST /models/book                     compute_book_consistency
//   GET/POST /models/folder/{id}          compute_folder_organisation
//   GET  /models/record/{id}              compute_record_integrity

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
	"time"
)

const (
	portfolioServiceID   = "kogi.services.portfolio"
	portfolioServiceName = "portfolio-service"
	portfolioDefaultPort = "9002"
	portfolioGateway     = "http://127.0.0.1:8090"
)

// ── Rust bridge ───────────────────────────────────────────────────────────────

type rustReq struct {
	Action  string      `json:"action"`
	Payload interface{} `json:"payload,omitempty"`
}

func portfolioCallRust(action string, payload interface{}) (interface{}, error) {
	bin, err := portfolioResolveBinary()
	if err != nil {
		return nil, err
	}
	envelope, _ := json.Marshal(rustReq{Action: action, Payload: payload})
	ctx, cancel := context.WithTimeout(context.Background(), 5*time.Second)
	defer cancel()
	cmd := exec.CommandContext(ctx, bin, "--request", string(envelope))
	out, err := cmd.CombinedOutput()
	if err != nil {
		return nil, fmt.Errorf("rust bridge action=%s: %w – %s", action, err, strings.TrimSpace(string(out)))
	}
	var result interface{}
	if err := json.Unmarshal(out, &result); err != nil {
		return nil, fmt.Errorf("rust response parse action=%s: %w", action, err)
	}
	return result, nil
}

func portfolioRust(action string, payload interface{}, fallback interface{}) interface{} {
	r, err := portfolioCallRust(action, payload)
	if err != nil {
		log.Printf("[portfolio-svc] rust action=%s err=%v", action, err)
		return fallback
	}
	return r
}

func portfolioResolveBinary() (string, error) {
	if p := os.Getenv("KOGI_PORTFOLIO_SYSTEM_BIN"); p != "" {
		return p, nil
	}
	if p, err := exec.LookPath("kogi-portfolio-system"); err == nil {
		return p, nil
	}
	if root, ok := portfolioFindRepoRoot(); ok {
		exe := "kogi-portfolio-system"
		if runtime.GOOS == "windows" {
			exe += ".exe"
		}
		for _, c := range []string{
			filepath.Join(root, "kogi-modules", "portfolio", "target", "debug", exe),
			filepath.Join(root, "kogi-modules", "portfolio", "target", "release", exe),
			filepath.Join(root, "kogi-modules", "portfolio", exe),
		} {
			if portfolioFileExists(c) {
				return c, nil
			}
		}
	}
	return "", fmt.Errorf("kogi-portfolio-system not found; set KOGI_PORTFOLIO_SYSTEM_BIN")
}

// ── Gateway pub/sub ───────────────────────────────────────────────────────────

func portfolioPublish(topic, payload string, meta map[string]string) {
	body, _ := json.Marshal(map[string]interface{}{
		"topic": topic, "payload": payload,
		"source": portfolioServiceID, "metadata": meta,
	})
	ctx, cancel := context.WithTimeout(context.Background(), 2*time.Second)
	defer cancel()
	req, _ := http.NewRequestWithContext(ctx, http.MethodPost,
		portfolioGateway+"/api/v1/gateway/pubsub/publish", bytes.NewReader(body))
	req.Header.Set("Content-Type", "application/json")
	resp, err := http.DefaultClient.Do(req)
	if err != nil {
		log.Printf("[portfolio-svc] publish topic=%s err=%v", topic, err)
		return
	}
	defer resp.Body.Close()
}

func portfolioRegisterWithGateway(selfEndpoint string) {
	body, _ := json.Marshal(map[string]interface{}{
		"id": portfolioServiceID, "kind": "service",
		"endpoint": selfEndpoint, "health_path": "/health",
		"network_manager": "kogi-go-network", "status": "active",
		"metadata": map[string]string{
			"publishes": strings.Join([]string{
				"portfolio.item.created", "portfolio.component.updated",
				"portfolio.component.removed", "portfolio.graph.changed",
				"portfolio.snapshot.saved", "portfolio.checkpoint.created",
				"portfolio.governance.resource.allocated", "portfolio.governance.resource.consumed",
				"portfolio.governance.approval.requested", "portfolio.governance.approval.resolved",
				"portfolio.model.computed",
			}, ","),
			"subscribes": "office.dashboard.refresh,ims.profile.updated,exchange.trade.executed",
		},
	})
	ctx, cancel := context.WithTimeout(context.Background(), 3*time.Second)
	defer cancel()
	req, _ := http.NewRequestWithContext(ctx, http.MethodPost,
		portfolioGateway+"/api/v1/gateway/components/register", bytes.NewReader(body))
	req.Header.Set("Content-Type", "application/json")
	resp, err := http.DefaultClient.Do(req)
	if err != nil {
		log.Printf("[portfolio-svc] gateway registration err=%v", err)
		return
	}
	defer resp.Body.Close()
	log.Printf("[portfolio-svc] registered with gateway %s", portfolioGateway)

	// Subscribe to inbound topics using the gateway's prefix-subscribe endpoint.
	// office.* events affect dashboard state; exchange.trade.executed may trigger
	// resource consumption reconciliation.
	for _, sub := range []struct{ prefix, consumer string }{
		{"office.", "kogi.services.portfolio"},
		{"ims.", "kogi.services.portfolio"},
		{"exchange.trade", "kogi.services.portfolio"},
	} {
		portfolioGatewaySubscribePrefix(sub.prefix, sub.consumer)
	}
}

// portfolioGatewaySubscribePrefix registers a prefix subscription on the gateway bus.
func portfolioGatewaySubscribePrefix(prefix, consumer string) {
	body, _ := json.Marshal(map[string]string{"prefix": prefix, "consumer": consumer})
	ctx, cancel := context.WithTimeout(context.Background(), 2*time.Second)
	defer cancel()
	req, _ := http.NewRequestWithContext(ctx, http.MethodPost,
		portfolioGateway+"/api/v1/gateway/pubsub/subscribe/prefix", bytes.NewReader(body))
	req.Header.Set("Content-Type", "application/json")
	resp, err := http.DefaultClient.Do(req)
	if err != nil {
		log.Printf("[portfolio-svc] prefix subscribe prefix=%s err=%v", prefix, err)
		return
	}
	defer resp.Body.Close()
}

// portfolioReportHealth sends a health record to the gateway mesh registry.
// Called periodically by the health-check goroutine.
func portfolioReportHealth(healthy bool, note string) {
	body, _ := json.Marshal(map[string]interface{}{
		"id":      portfolioServiceID,
		"healthy": healthy,
		"note":    note,
	})
	ctx, cancel := context.WithTimeout(context.Background(), 2*time.Second)
	defer cancel()
	req, _ := http.NewRequestWithContext(ctx, http.MethodPost,
		portfolioGateway+"/api/v1/gateway/components/health", bytes.NewReader(body))
	req.Header.Set("Content-Type", "application/json")
	resp, err := http.DefaultClient.Do(req)
	if err != nil {
		log.Printf("[portfolio-svc] health report err=%v", err)
		return
	}
	defer resp.Body.Close()
}

// portfolioHealthLoop reports health to the gateway every 30 seconds.
func portfolioHealthLoop() {
	for {
		time.Sleep(30 * time.Second)
		bin, err := portfolioResolveBinary()
		portfolioReportHealth(err == nil, bin)
	}
}

// portfolioDeadLetterMonitor polls the gateway dead-letter queue and logs any
// portfolio-namespace events that arrived with no subscribers.
func portfolioDeadLetterMonitor() {
	for {
		time.Sleep(60 * time.Second)
		ctx, cancel := context.WithTimeout(context.Background(), 3*time.Second)
		req, _ := http.NewRequestWithContext(ctx, http.MethodGet,
			portfolioGateway+"/api/v1/gateway/pubsub/dead-letters?limit=20", nil)
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
			if strings.HasPrefix(dl.Topic, "portfolio.") {
				log.Printf("[portfolio-svc] dead-letter id=%s topic=%s", dl.ID, dl.Topic)
			}
		}
	}
}
// subscribes to and logs them.  Runs in its own goroutine.
func portfolioInboundPoll(topics []string) {
	for {
		for _, topic := range topics {
			func() {
				ctx, cancel := context.WithTimeout(context.Background(), 3*time.Second)
				defer cancel()
				req, _ := http.NewRequestWithContext(ctx, http.MethodGet,
					fmt.Sprintf("%s/api/v1/gateway/pubsub/receive?topic=%s&limit=5",
						portfolioGateway, topic), nil)
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
					log.Printf("[portfolio-svc] inbound topic=%s source=%s payload=%s",
						e.Topic, e.Source, e.Payload)
				}
			}()
		}
		time.Sleep(4 * time.Second)
	}
}

// ── HTTP helpers ──────────────────────────────────────────────────────────────

func writeJSON(w http.ResponseWriter, status int, payload interface{}) {
	w.Header().Set("Content-Type", "application/json")
	w.WriteHeader(status)
	_ = json.NewEncoder(w).Encode(payload)
}

func decodeBody(r *http.Request, dst interface{}) error {
	defer r.Body.Close()
	b, _ := io.ReadAll(r.Body)
	if err := json.Unmarshal(b, dst); err != nil {
		return fmt.Errorf("invalid JSON: %w", err)
	}
	return nil
}

func trimPrefix(r *http.Request, prefix string) string {
	return strings.TrimPrefix(r.URL.Path, prefix)
}

func resolveAddr(defaultPort, envKey string) string {
	if p := os.Getenv(envKey); p != "" {
		return portfolioToListenAddr(p)
	}
	if p := os.Getenv("KOGI_PORT"); p != "" {
		return portfolioToListenAddr(p)
	}
	return ":" + defaultPort
}

func portfolioToListenAddr(port string) string {
	if strings.HasPrefix(port, ":") {
		return port
	}
	return ":" + port
}

// portfolioProxyGateway does a GET to the gateway at path and writes the
// response body directly.  On any error it writes fallback as JSON.
func portfolioProxyGateway(w http.ResponseWriter, path string, fallback interface{}) {
	ctx, cancel := context.WithTimeout(context.Background(), 3*time.Second)
	defer cancel()
	req, _ := http.NewRequestWithContext(ctx, http.MethodGet, portfolioGateway+path, nil)
	resp, err := http.DefaultClient.Do(req)
	if err != nil {
		writeJSON(w, http.StatusOK, fallback)
		return
	}
	defer resp.Body.Close()
	w.Header().Set("Content-Type", "application/json")
	w.WriteHeader(resp.StatusCode)
	_, _ = io.Copy(w, resp.Body)
}

// portfolioFileExists reports whether p names an existing regular file.
	info, err := os.Stat(p)
	return err == nil && !info.IsDir()
}

func portfolioFindRepoRoot() (string, bool) {
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

// ── Request / Response types ──────────────────────────────────────────────────

type createComponentReq struct {
	ComponentType string `json:"component_type"`
	Name          string `json:"name"`
}
type createBookReq struct {
	BookType string `json:"book_type"`
	Name     string `json:"name"`
}
type editComponentReq struct {
	ID         string            `json:"id"`
	Name       *string           `json:"name,omitempty"`
	Status     *string           `json:"status,omitempty"`
	Tags       []string          `json:"tags,omitempty"`
	Properties map[string]string `json:"properties,omitempty"`
	Owner      *string           `json:"owner,omitempty"`
}
type hierarchyReq struct {
	ParentID string `json:"parent_id"`
	ChildID  string `json:"child_id"`
}
type dependencyReq struct {
	FromID string `json:"from_id"`
	ToID   string `json:"to_id"`
}
type linkReq struct {
	AID string `json:"a_id"`
	BID string `json:"b_id"`
}
type memberReq struct {
	ContainerID string `json:"container_id"`
	ItemID      string `json:"item_id"`
}
type saveSnapshotReq struct {
	Label *string `json:"label,omitempty"`
}
type saveCheckpointReq struct {
	Label string  `json:"label"`
	Note  *string `json:"note,omitempty"`
}
type policyReq struct {
	ComponentID string `json:"component_id"`
	PolicyID    string `json:"policy_id"`
}
type approvalReqBody struct {
	ComponentID string `json:"component_id"`
	Reason      string `json:"reason"`
}
type approvalResolveBody struct {
	RequestID string  `json:"request_id"`
	Approved  bool    `json:"approved"`
	Resolver  string  `json:"resolver"`
	Notes     *string `json:"notes,omitempty"`
}
type allocateResourceReq struct {
	ComponentID  string  `json:"component_id"`
	Kind         string  `json:"kind"`
	Total        float64 `json:"total"`
	Denomination string  `json:"denomination"`
	Period       *string `json:"period,omitempty"`
}
type recordConsumptionReq struct {
	ComponentID string  `json:"component_id"`
	Amount      float64 `json:"amount"`
}
type setActiveReq struct {
	ComponentID string `json:"component_id"`
}
type projectMetricsReq struct {
	PlannedValue       float64 `json:"planned_value"`
	EarnedValue        float64 `json:"earned_value"`
	ActualCost         float64 `json:"actual_cost"`
	BudgetAtCompletion float64 `json:"budget_at_completion"`
	ScheduleRiskWeight float64 `json:"schedule_risk_weight"`
	OpenRiskCount      int     `json:"open_risk_count"`
}
type assetValueReq struct {
	AcquisitionCost   float64 `json:"acquisition_cost"`
	CurrentBookValue  float64 `json:"current_book_value"`
	MarketValue       float64 `json:"market_value"`
	TotalReturn       float64 `json:"total_return"`
	AgePeriods        float64 `json:"age_periods"`
	UsefulLifePeriods float64 `json:"useful_life_periods"`
}
type bookConsistencyReq struct {
	BookType         string `json:"book_type"`
	PageCount        int    `json:"page_count"`
	MinExpectedPages int    `json:"min_expected_pages"`
	ReviewedPages    int    `json:"reviewed_pages"`
	BrokenRefs       int    `json:"broken_refs"`
	HasIndex         bool   `json:"has_index"`
}
type binderCoverageReq struct {
	BinderID    string   `json:"binder_id"`
	ExpectedIDs []string `json:"expected_ids"`
}
type folderOrganisationReq struct {
	FolderID       string `json:"folder_id"`
	DepthThreshold int    `json:"depth_threshold"`
}
type artifactMaturityReq struct {
	ArtifactID     string   `json:"artifact_id"`
	RequiredFields []string `json:"required_fields"`
	MaxFreshDays   uint64   `json:"max_fresh_days"`
}

// ── main ──────────────────────────────────────────────────────────────────────

func main() {
	mux := http.NewServeMux()

	addr := resolveAddr(portfolioDefaultPort, "KOGI_PORTFOLIO_PORT")
	selfEndpoint := fmt.Sprintf("http://127.0.0.1%s", addr)
	go func() {
		time.Sleep(600 * time.Millisecond)
		portfolioRegisterWithGateway(selfEndpoint)
		// Start inbound poll after registration so gateway subscriptions exist.
		go portfolioInboundPoll([]string{
			"office.dashboard.refresh",
			"ims.profile.updated",
			"exchange.trade.executed",
		})
		go portfolioHealthLoop()
		go portfolioDeadLetterMonitor()
	}()

	// ── Health ────────────────────────────────────────────────────────────
	mux.HandleFunc("/health", func(w http.ResponseWriter, r *http.Request) {
		writeJSON(w, http.StatusOK, map[string]string{"status": "ok", "service": portfolioServiceName})
	})

	// ── Runtime manifest ──────────────────────────────────────────────────
	mux.HandleFunc("/api/v1/portfolio/runtime", func(w http.ResponseWriter, r *http.Request) {
		bin, _ := portfolioResolveBinary()
		writeJSON(w, http.StatusOK, map[string]interface{}{
			"service": portfolioServiceName, "component_id": portfolioServiceID,
			"gateway": portfolioGateway, "network_manager": "kogi-go-network",
			"rust_bridge": "kogi-portfolio-system", "rust_binary": bin,
			"publishes": []string{
				"portfolio.item.created", "portfolio.component.updated",
				"portfolio.component.removed", "portfolio.graph.changed",
				"portfolio.snapshot.saved", "portfolio.checkpoint.created",
				"portfolio.governance.resource.allocated", "portfolio.governance.resource.consumed",
				"portfolio.governance.approval.requested", "portfolio.governance.approval.resolved",
				"portfolio.model.computed",
			},
			"subscribes":    []string{"office.dashboard.refresh", "ims.profile.updated", "exchange.trade.executed"},
			"engine_stream": "engine.ingest",
		})
	})

	// ── Overview / snapshot / metadata ───────────────────────────────────
	mux.HandleFunc("/api/v1/portfolio/overview", func(w http.ResponseWriter, r *http.Request) {
		writeJSON(w, http.StatusOK, portfolioRust("overview", nil, map[string]interface{}{
			"module": "kogi.portfolio", "service": portfolioServiceName,
		}))
	})

	mux.HandleFunc("/api/v1/portfolio/snapshot", func(w http.ResponseWriter, r *http.Request) {
		writeJSON(w, http.StatusOK, portfolioRust("snapshot", nil,
			map[string]interface{}{"snapshot_id": "unavailable"}))
	})

	mux.HandleFunc("/api/v1/portfolio/metadata", func(w http.ResponseWriter, r *http.Request) {
		writeJSON(w, http.StatusOK, portfolioRust("portfolio_metadata", nil, map[string]interface{}{}))
	})

	// ── Components ────────────────────────────────────────────────────────
	mux.HandleFunc("/api/v1/portfolio/components", func(w http.ResponseWriter, r *http.Request) {
		switch r.Method {
		case http.MethodGet:
			writeJSON(w, http.StatusOK, map[string]interface{}{
				"components": portfolioRust("all_components", nil, []interface{}{}),
			})
		case http.MethodPost:
			var req createComponentReq
			if err := decodeBody(r, &req); err != nil {
				writeJSON(w, http.StatusBadRequest, map[string]string{"error": err.Error()})
				return
			}
			result, err := portfolioCallRust("create_component", req)
			if err != nil {
				writeJSON(w, http.StatusInternalServerError, map[string]string{"error": err.Error()})
				return
			}
			go portfolioPublish("portfolio.item.created",
				fmt.Sprintf(`{"name":%q,"type":%q}`, req.Name, req.ComponentType), nil)
			writeJSON(w, http.StatusCreated, result)
		default:
			writeJSON(w, http.StatusMethodNotAllowed, map[string]string{"error": "method_not_allowed"})
		}
	})

	// Specific component type listing — must be registered before the /{id} handler
	mux.HandleFunc("/api/v1/portfolio/components/type/", func(w http.ResponseWriter, r *http.Request) {
		ct := trimPrefix(r, "/api/v1/portfolio/components/type/")
		writeJSON(w, http.StatusOK, map[string]interface{}{
			"type":       ct,
			"components": portfolioRust("components_by_type", map[string]string{"component_type": ct}, []interface{}{}),
		})
	})

	// Individual component CRUD
	mux.HandleFunc("/api/v1/portfolio/components/", func(w http.ResponseWriter, r *http.Request) {
		id := trimPrefix(r, "/api/v1/portfolio/components/")
		if id == "" {
			writeJSON(w, http.StatusBadRequest, map[string]string{"error": "component id required"})
			return
		}
		switch r.Method {
		case http.MethodGet:
			res := portfolioRust("get_component", map[string]string{"id": id}, nil)
			if res == nil {
				writeJSON(w, http.StatusNotFound, map[string]string{"error": "not found"})
				return
			}
			writeJSON(w, http.StatusOK, res)
		case http.MethodPut:
			var req editComponentReq
			if err := decodeBody(r, &req); err != nil {
				writeJSON(w, http.StatusBadRequest, map[string]string{"error": err.Error()})
				return
			}
			req.ID = id
			result, err := portfolioCallRust("edit_component", req)
			if err != nil {
				writeJSON(w, http.StatusInternalServerError, map[string]string{"error": err.Error()})
				return
			}
			go portfolioPublish("portfolio.component.updated", fmt.Sprintf(`{"id":%q}`, id), nil)
			writeJSON(w, http.StatusOK, result)
		case http.MethodDelete:
			result, err := portfolioCallRust("remove_component", map[string]string{"id": id})
			if err != nil {
				writeJSON(w, http.StatusInternalServerError, map[string]string{"error": err.Error()})
				return
			}
			go portfolioPublish("portfolio.component.removed", fmt.Sprintf(`{"id":%q}`, id), nil)
			writeJSON(w, http.StatusOK, result)
		default:
			writeJSON(w, http.StatusMethodNotAllowed, map[string]string{"error": "method_not_allowed"})
		}
	})

	// ── Books ─────────────────────────────────────────────────────────────
	mux.HandleFunc("/api/v1/portfolio/books", func(w http.ResponseWriter, r *http.Request) {
		if r.Method != http.MethodPost {
			writeJSON(w, http.StatusMethodNotAllowed, map[string]string{"error": "method_not_allowed"})
			return
		}
		var req createBookReq
		if err := decodeBody(r, &req); err != nil {
			writeJSON(w, http.StatusBadRequest, map[string]string{"error": err.Error()})
			return
		}
		result, err := portfolioCallRust("create_book", req)
		if err != nil {
			writeJSON(w, http.StatusInternalServerError, map[string]string{"error": err.Error()})
			return
		}
		go portfolioPublish("portfolio.item.created",
			fmt.Sprintf(`{"name":%q,"type":"book","book_type":%q}`, req.Name, req.BookType), nil)
		writeJSON(w, http.StatusCreated, result)
	})

	// ── Active portfolio ──────────────────────────────────────────────────
	mux.HandleFunc("/api/v1/portfolio/active", func(w http.ResponseWriter, r *http.Request) {
		if r.Method != http.MethodPost {
			writeJSON(w, http.StatusMethodNotAllowed, map[string]string{"error": "method_not_allowed"})
			return
		}
		var req setActiveReq
		if err := decodeBody(r, &req); err != nil {
			writeJSON(w, http.StatusBadRequest, map[string]string{"error": err.Error()})
			return
		}
		result, err := portfolioCallRust("set_active_portfolio", req)
		if err != nil {
			writeJSON(w, http.StatusInternalServerError, map[string]string{"error": err.Error()})
			return
		}
		writeJSON(w, http.StatusOK, result)
	})

	// ── Graph: hierarchy ──────────────────────────────────────────────────
	mux.HandleFunc("/api/v1/portfolio/graph/hierarchy", func(w http.ResponseWriter, r *http.Request) {
		var req hierarchyReq
		if err := decodeBody(r, &req); err != nil {
			writeJSON(w, http.StatusBadRequest, map[string]string{"error": err.Error()})
			return
		}
		var action string
		switch r.Method {
		case http.MethodPost:
			action = "add_hierarchy"
		case http.MethodDelete:
			action = "remove_hierarchy"
		default:
			writeJSON(w, http.StatusMethodNotAllowed, map[string]string{"error": "method_not_allowed"})
			return
		}
		result, err := portfolioCallRust(action, req)
		if err != nil {
			writeJSON(w, http.StatusInternalServerError, map[string]string{"error": err.Error()})
			return
		}
		go portfolioPublish("portfolio.graph.changed",
			fmt.Sprintf(`{"action":%q,"parent_id":%q,"child_id":%q}`, action, req.ParentID, req.ChildID), nil)
		writeJSON(w, http.StatusOK, result)
	})

	// ── Graph: dependency ─────────────────────────────────────────────────
	mux.HandleFunc("/api/v1/portfolio/graph/dependency", func(w http.ResponseWriter, r *http.Request) {
		var req dependencyReq
		if err := decodeBody(r, &req); err != nil {
			writeJSON(w, http.StatusBadRequest, map[string]string{"error": err.Error()})
			return
		}
		var action string
		switch r.Method {
		case http.MethodPost:
			action = "add_dependency"
		case http.MethodDelete:
			action = "remove_dependency"
		default:
			writeJSON(w, http.StatusMethodNotAllowed, map[string]string{"error": "method_not_allowed"})
			return
		}
		result, err := portfolioCallRust(action, req)
		if err != nil {
			writeJSON(w, http.StatusInternalServerError, map[string]string{"error": err.Error()})
			return
		}
		go portfolioPublish("portfolio.graph.changed",
			fmt.Sprintf(`{"action":%q,"from":%q,"to":%q}`, action, req.FromID, req.ToID), nil)
		writeJSON(w, http.StatusOK, result)
	})

	// ── Graph: peer link ─────────────────────────────────────────────────
	mux.HandleFunc("/api/v1/portfolio/graph/link", func(w http.ResponseWriter, r *http.Request) {
		var req linkReq
		if err := decodeBody(r, &req); err != nil {
			writeJSON(w, http.StatusBadRequest, map[string]string{"error": err.Error()})
			return
		}
		var action string
		switch r.Method {
		case http.MethodPost:
			action = "add_link"
		case http.MethodDelete:
			action = "remove_link"
		default:
			writeJSON(w, http.StatusMethodNotAllowed, map[string]string{"error": "method_not_allowed"})
			return
		}
		result, err := portfolioCallRust(action, req)
		if err != nil {
			writeJSON(w, http.StatusInternalServerError, map[string]string{"error": err.Error()})
			return
		}
		go portfolioPublish("portfolio.graph.changed",
			fmt.Sprintf(`{"action":%q,"a":%q,"b":%q}`, action, req.AID, req.BID), nil)
		writeJSON(w, http.StatusOK, result)
	})

	// ── Graph: container membership ───────────────────────────────────────
	mux.HandleFunc("/api/v1/portfolio/graph/member", func(w http.ResponseWriter, r *http.Request) {
		var req memberReq
		if err := decodeBody(r, &req); err != nil {
			writeJSON(w, http.StatusBadRequest, map[string]string{"error": err.Error()})
			return
		}
		var action string
		switch r.Method {
		case http.MethodPost:
			action = "add_member"
		case http.MethodDelete:
			action = "remove_member"
		default:
			writeJSON(w, http.StatusMethodNotAllowed, map[string]string{"error": "method_not_allowed"})
			return
		}
		result, err := portfolioCallRust(action, req)
		if err != nil {
			writeJSON(w, http.StatusInternalServerError, map[string]string{"error": err.Error()})
			return
		}
		go portfolioPublish("portfolio.graph.changed",
			fmt.Sprintf(`{"action":%q,"container":%q,"item":%q}`, action, req.ContainerID, req.ItemID), nil)
		writeJSON(w, http.StatusOK, result)
	})

	// ── Graph traversal ───────────────────────────────────────────────────
	mux.HandleFunc("/api/v1/portfolio/graph/subtree/", func(w http.ResponseWriter, r *http.Request) {
		id := trimPrefix(r, "/api/v1/portfolio/graph/subtree/")
		writeJSON(w, http.StatusOK, map[string]interface{}{
			"root":    id,
			"subtree": portfolioRust("subtree", map[string]string{"id": id}, []interface{}{}),
		})
	})

	mux.HandleFunc("/api/v1/portfolio/graph/dependencies/", func(w http.ResponseWriter, r *http.Request) {
		id := trimPrefix(r, "/api/v1/portfolio/graph/dependencies/")
		writeJSON(w, http.StatusOK, map[string]interface{}{
			"root":         id,
			"dependencies": portfolioRust("transitive_dependencies", map[string]string{"id": id}, []interface{}{}),
		})
	})

	mux.HandleFunc("/api/v1/portfolio/graph/order", func(w http.ResponseWriter, r *http.Request) {
		writeJSON(w, http.StatusOK, map[string]interface{}{
			"order": portfolioRust("dependency_order", nil, []interface{}{}),
		})
	})

	// ── Snapshots ─────────────────────────────────────────────────────────
	mux.HandleFunc("/api/v1/portfolio/snapshots", func(w http.ResponseWriter, r *http.Request) {
		switch r.Method {
		case http.MethodGet:
			writeJSON(w, http.StatusOK, map[string]interface{}{
				"snapshots": portfolioRust("list_snapshots", nil, []interface{}{}),
			})
		case http.MethodPost:
			var req saveSnapshotReq
			_ = decodeBody(r, &req)
			result, err := portfolioCallRust("save_snapshot", req)
			if err != nil {
				writeJSON(w, http.StatusInternalServerError, map[string]string{"error": err.Error()})
				return
			}
			go portfolioPublish("portfolio.snapshot.saved", `{}`, nil)
			writeJSON(w, http.StatusCreated, result)
		default:
			writeJSON(w, http.StatusMethodNotAllowed, map[string]string{"error": "method_not_allowed"})
		}
	})

	mux.HandleFunc("/api/v1/portfolio/snapshots/", func(w http.ResponseWriter, r *http.Request) {
		// Pattern: /api/v1/portfolio/snapshots/{id}/restore
		parts := strings.SplitN(trimPrefix(r, "/api/v1/portfolio/snapshots/"), "/", 2)
		if len(parts) != 2 || parts[1] != "restore" {
			writeJSON(w, http.StatusBadRequest, map[string]string{"error": "use /{id}/restore"})
			return
		}
		result, err := portfolioCallRust("restore_snapshot", map[string]string{"snapshot_id": parts[0]})
		if err != nil {
			writeJSON(w, http.StatusInternalServerError, map[string]string{"error": err.Error()})
			return
		}
		writeJSON(w, http.StatusOK, result)
	})

	// ── Checkpoints ───────────────────────────────────────────────────────
	mux.HandleFunc("/api/v1/portfolio/checkpoints", func(w http.ResponseWriter, r *http.Request) {
		switch r.Method {
		case http.MethodGet:
			writeJSON(w, http.StatusOK, map[string]interface{}{
				"checkpoints": portfolioRust("list_checkpoints", nil, []interface{}{}),
			})
		case http.MethodPost:
			var req saveCheckpointReq
			if err := decodeBody(r, &req); err != nil {
				writeJSON(w, http.StatusBadRequest, map[string]string{"error": err.Error()})
				return
			}
			result, err := portfolioCallRust("save_checkpoint", req)
			if err != nil {
				writeJSON(w, http.StatusInternalServerError, map[string]string{"error": err.Error()})
				return
			}
			go portfolioPublish("portfolio.checkpoint.created",
				fmt.Sprintf(`{"label":%q}`, req.Label), nil)
			writeJSON(w, http.StatusCreated, result)
		default:
			writeJSON(w, http.StatusMethodNotAllowed, map[string]string{"error": "method_not_allowed"})
		}
	})

	mux.HandleFunc("/api/v1/portfolio/checkpoints/", func(w http.ResponseWriter, r *http.Request) {
		parts := strings.SplitN(trimPrefix(r, "/api/v1/portfolio/checkpoints/"), "/", 2)
		if len(parts) != 2 || parts[1] != "restore" {
			writeJSON(w, http.StatusBadRequest, map[string]string{"error": "use /{id}/restore"})
			return
		}
		result, err := portfolioCallRust("restore_checkpoint", map[string]string{"checkpoint_id": parts[0]})
		if err != nil {
			writeJSON(w, http.StatusInternalServerError, map[string]string{"error": err.Error()})
			return
		}
		writeJSON(w, http.StatusOK, result)
	})

	// ── Query ─────────────────────────────────────────────────────────────
	mux.HandleFunc("/api/v1/portfolio/query", func(w http.ResponseWriter, r *http.Request) {
		pql := r.URL.Query().Get("pql")
		if pql == "" {
			pql = r.URL.Query().Get("q")
		}
		writeJSON(w, http.StatusOK, map[string]interface{}{
			"query":   pql,
			"results": portfolioRust("query_pql", map[string]string{"pql": pql}, []interface{}{}),
		})
	})

	// ── Event log ─────────────────────────────────────────────────────────
	mux.HandleFunc("/api/v1/portfolio/events", func(w http.ResponseWriter, r *http.Request) {
		writeJSON(w, http.StatusOK, map[string]interface{}{
			"events": portfolioRust("event_log", nil, []interface{}{}),
		})
	})

	// ── Governance: policy ────────────────────────────────────────────────
	mux.HandleFunc("/api/v1/portfolio/governance/policy/attach", func(w http.ResponseWriter, r *http.Request) {
		if r.Method != http.MethodPost {
			writeJSON(w, http.StatusMethodNotAllowed, map[string]string{"error": "method_not_allowed"})
			return
		}
		var req policyReq
		if err := decodeBody(r, &req); err != nil {
			writeJSON(w, http.StatusBadRequest, map[string]string{"error": err.Error()})
			return
		}
		result, err := portfolioCallRust("attach_policy", req)
		if err != nil {
			writeJSON(w, http.StatusInternalServerError, map[string]string{"error": err.Error()})
			return
		}
		writeJSON(w, http.StatusOK, result)
	})

	mux.HandleFunc("/api/v1/portfolio/governance/policy/detach", func(w http.ResponseWriter, r *http.Request) {
		if r.Method != http.MethodPost {
			writeJSON(w, http.StatusMethodNotAllowed, map[string]string{"error": "method_not_allowed"})
			return
		}
		var req policyReq
		if err := decodeBody(r, &req); err != nil {
			writeJSON(w, http.StatusBadRequest, map[string]string{"error": err.Error()})
			return
		}
		result, err := portfolioCallRust("detach_policy", req)
		if err != nil {
			writeJSON(w, http.StatusInternalServerError, map[string]string{"error": err.Error()})
			return
		}
		writeJSON(w, http.StatusOK, result)
	})

	// ── Governance: approval workflow ─────────────────────────────────────
	mux.HandleFunc("/api/v1/portfolio/governance/approval/request", func(w http.ResponseWriter, r *http.Request) {
		if r.Method != http.MethodPost {
			writeJSON(w, http.StatusMethodNotAllowed, map[string]string{"error": "method_not_allowed"})
			return
		}
		var req approvalReqBody
		if err := decodeBody(r, &req); err != nil {
			writeJSON(w, http.StatusBadRequest, map[string]string{"error": err.Error()})
			return
		}
		result, err := portfolioCallRust("request_approval", req)
		if err != nil {
			writeJSON(w, http.StatusInternalServerError, map[string]string{"error": err.Error()})
			return
		}
		go portfolioPublish("portfolio.governance.approval.requested",
			fmt.Sprintf(`{"component_id":%q,"reason":%q}`, req.ComponentID, req.Reason), nil)
		writeJSON(w, http.StatusCreated, result)
	})

	mux.HandleFunc("/api/v1/portfolio/governance/approval/", func(w http.ResponseWriter, r *http.Request) {
		// POST /api/v1/portfolio/governance/approval/{id}/resolve
		parts := strings.SplitN(trimPrefix(r, "/api/v1/portfolio/governance/approval/"), "/", 2)
		if len(parts) != 2 || parts[1] != "resolve" || r.Method != http.MethodPost {
			writeJSON(w, http.StatusBadRequest, map[string]string{"error": "POST /{id}/resolve"})
			return
		}
		var req approvalResolveBody
		if err := decodeBody(r, &req); err != nil {
			writeJSON(w, http.StatusBadRequest, map[string]string{"error": err.Error()})
			return
		}
		req.RequestID = parts[0]
		result, err := portfolioCallRust("resolve_approval", req)
		if err != nil {
			writeJSON(w, http.StatusInternalServerError, map[string]string{"error": err.Error()})
			return
		}
		go portfolioPublish("portfolio.governance.approval.resolved",
			fmt.Sprintf(`{"request_id":%q,"approved":%v}`, req.RequestID, req.Approved), nil)
		writeJSON(w, http.StatusOK, result)
	})

	// ── Governance: resource allocation ───────────────────────────────────
	mux.HandleFunc("/api/v1/portfolio/governance/resource/allocate", func(w http.ResponseWriter, r *http.Request) {
		if r.Method != http.MethodPost {
			writeJSON(w, http.StatusMethodNotAllowed, map[string]string{"error": "method_not_allowed"})
			return
		}
		var req allocateResourceReq
		if err := decodeBody(r, &req); err != nil {
			writeJSON(w, http.StatusBadRequest, map[string]string{"error": err.Error()})
			return
		}
		result, err := portfolioCallRust("allocate_resource", req)
		if err != nil {
			writeJSON(w, http.StatusInternalServerError, map[string]string{"error": err.Error()})
			return
		}
		go portfolioPublish("portfolio.governance.resource.allocated",
			fmt.Sprintf(`{"component_id":%q,"kind":%q,"total":%v}`, req.ComponentID, req.Kind, req.Total), nil)
		writeJSON(w, http.StatusCreated, result)
	})

	mux.HandleFunc("/api/v1/portfolio/governance/resource/consume", func(w http.ResponseWriter, r *http.Request) {
		if r.Method != http.MethodPost {
			writeJSON(w, http.StatusMethodNotAllowed, map[string]string{"error": "method_not_allowed"})
			return
		}
		var req recordConsumptionReq
		if err := decodeBody(r, &req); err != nil {
			writeJSON(w, http.StatusBadRequest, map[string]string{"error": err.Error()})
			return
		}
		result, err := portfolioCallRust("record_consumption", req)
		if err != nil {
			writeJSON(w, http.StatusInternalServerError, map[string]string{"error": err.Error()})
			return
		}
		go portfolioPublish("portfolio.governance.resource.consumed",
			fmt.Sprintf(`{"component_id":%q,"amount":%v}`, req.ComponentID, req.Amount), nil)
		writeJSON(w, http.StatusOK, result)
	})

	mux.HandleFunc("/api/v1/portfolio/governance/resource/overruns", func(w http.ResponseWriter, r *http.Request) {
		writeJSON(w, http.StatusOK, map[string]interface{}{
			"overruns": portfolioRust("overrun_allocations", nil, []interface{}{}),
		})
	})

	// Must come after more-specific /allocate, /consume, /overruns routes
	mux.HandleFunc("/api/v1/portfolio/governance/resource/", func(w http.ResponseWriter, r *http.Request) {
		id := trimPrefix(r, "/api/v1/portfolio/governance/resource/")
		res := portfolioRust("get_resource_allocation", map[string]string{"component_id": id}, nil)
		if res == nil {
			writeJSON(w, http.StatusNotFound, map[string]string{"error": "allocation not found"})
			return
		}
		writeJSON(w, http.StatusOK, res)
	})

	// ── Computational models ──────────────────────────────────────────────

	mux.HandleFunc("/api/v1/portfolio/models/health/", func(w http.ResponseWriter, r *http.Request) {
		id := trimPrefix(r, "/api/v1/portfolio/models/health/")
		res := portfolioRust("compute_portfolio_health", map[string]string{"portfolio_id": id}, nil)
		if res == nil {
			writeJSON(w, http.StatusNotFound, map[string]string{"error": "not found or not a portfolio type"})
			return
		}
		go portfolioPublish("portfolio.model.computed",
			fmt.Sprintf(`{"model":"portfolio_health","id":%q}`, id), nil)
		writeJSON(w, http.StatusOK, res)
	})

	mux.HandleFunc("/api/v1/portfolio/models/project", func(w http.ResponseWriter, r *http.Request) {
		if r.Method != http.MethodPost {
			writeJSON(w, http.StatusMethodNotAllowed, map[string]string{"error": "method_not_allowed"})
			return
		}
		var req projectMetricsReq
		if err := decodeBody(r, &req); err != nil {
			writeJSON(w, http.StatusBadRequest, map[string]string{"error": err.Error()})
			return
		}
		writeJSON(w, http.StatusOK, portfolioRust("compute_project_metrics", req, map[string]interface{}{"error": "unavailable"}))
	})

	mux.HandleFunc("/api/v1/portfolio/models/program", func(w http.ResponseWriter, r *http.Request) {
		if r.Method != http.MethodPost {
			writeJSON(w, http.StatusMethodNotAllowed, map[string]string{"error": "method_not_allowed"})
			return
		}
		var body interface{}
		_ = decodeBody(r, &body)
		writeJSON(w, http.StatusOK, portfolioRust("compute_program_alignment", body, map[string]interface{}{"error": "unavailable"}))
	})

	mux.HandleFunc("/api/v1/portfolio/models/subportfolio/", func(w http.ResponseWriter, r *http.Request) {
		id := trimPrefix(r, "/api/v1/portfolio/models/subportfolio/")
		res := portfolioRust("compute_subportfolio_rollup", map[string]string{"subportfolio_id": id}, nil)
		if res == nil {
			writeJSON(w, http.StatusNotFound, map[string]string{"error": "not found"})
			return
		}
		writeJSON(w, http.StatusOK, res)
	})

	mux.HandleFunc("/api/v1/portfolio/models/resource/", func(w http.ResponseWriter, r *http.Request) {
		id := trimPrefix(r, "/api/v1/portfolio/models/resource/")
		res := portfolioRust("compute_resource_utilisation", map[string]string{"resource_id": id}, nil)
		if res == nil {
			writeJSON(w, http.StatusNotFound, map[string]string{"error": "not found or not a resource"})
			return
		}
		writeJSON(w, http.StatusOK, res)
	})

	mux.HandleFunc("/api/v1/portfolio/models/asset", func(w http.ResponseWriter, r *http.Request) {
		if r.Method != http.MethodPost {
			writeJSON(w, http.StatusMethodNotAllowed, map[string]string{"error": "method_not_allowed"})
			return
		}
		var req assetValueReq
		if err := decodeBody(r, &req); err != nil {
			writeJSON(w, http.StatusBadRequest, map[string]string{"error": err.Error()})
			return
		}
		writeJSON(w, http.StatusOK, portfolioRust("compute_asset_value", req, map[string]interface{}{"error": "unavailable"}))
	})

	mux.HandleFunc("/api/v1/portfolio/models/artifact/", func(w http.ResponseWriter, r *http.Request) {
		id := trimPrefix(r, "/api/v1/portfolio/models/artifact/")
		req := artifactMaturityReq{ArtifactID: id, MaxFreshDays: 30}
		if r.Method == http.MethodPost {
			_ = decodeBody(r, &req)
			req.ArtifactID = id
		}
		res := portfolioRust("compute_artifact_maturity", req, nil)
		if res == nil {
			writeJSON(w, http.StatusNotFound, map[string]string{"error": "not found or not an artifact"})
			return
		}
		writeJSON(w, http.StatusOK, res)
	})

	mux.HandleFunc("/api/v1/portfolio/models/binder/", func(w http.ResponseWriter, r *http.Request) {
		id := trimPrefix(r, "/api/v1/portfolio/models/binder/")
		req := binderCoverageReq{BinderID: id}
		if r.Method == http.MethodPost {
			_ = decodeBody(r, &req)
			req.BinderID = id
		}
		res := portfolioRust("compute_binder_coverage", req, nil)
		if res == nil {
			writeJSON(w, http.StatusNotFound, map[string]string{"error": "not found or not a binder"})
			return
		}
		writeJSON(w, http.StatusOK, res)
	})

	mux.HandleFunc("/api/v1/portfolio/models/book", func(w http.ResponseWriter, r *http.Request) {
		if r.Method != http.MethodPost {
			writeJSON(w, http.StatusMethodNotAllowed, map[string]string{"error": "method_not_allowed"})
			return
		}
		var req bookConsistencyReq
		if err := decodeBody(r, &req); err != nil {
			writeJSON(w, http.StatusBadRequest, map[string]string{"error": err.Error()})
			return
		}
		writeJSON(w, http.StatusOK, portfolioRust("compute_book_consistency", req, map[string]interface{}{"error": "unavailable"}))
	})

	mux.HandleFunc("/api/v1/portfolio/models/folder/", func(w http.ResponseWriter, r *http.Request) {
		id := trimPrefix(r, "/api/v1/portfolio/models/folder/")
		req := folderOrganisationReq{FolderID: id, DepthThreshold: 4}
		if r.Method == http.MethodPost {
			_ = decodeBody(r, &req)
			req.FolderID = id
		}
		res := portfolioRust("compute_folder_organisation", req, nil)
		if res == nil {
			writeJSON(w, http.StatusNotFound, map[string]string{"error": "not found or not a folder"})
			return
		}
		writeJSON(w, http.StatusOK, res)
	})

	mux.HandleFunc("/api/v1/portfolio/models/record/", func(w http.ResponseWriter, r *http.Request) {
		id := trimPrefix(r, "/api/v1/portfolio/models/record/")
		res := portfolioRust("compute_record_integrity", map[string]string{"record_id": id}, nil)
		if res == nil {
			writeJSON(w, http.StatusNotFound, map[string]string{"error": "not found or not a record"})
			return
		}
		writeJSON(w, http.StatusOK, res)
	})

	// ── Root redirect ────────────────────────────────────────────────────
	mux.HandleFunc("/api/v1/portfolio/root", func(w http.ResponseWriter, r *http.Request) {
		writeJSON(w, http.StatusOK, portfolioRust("snapshot", nil, map[string]interface{}{"snapshot_id": "unavailable"}))
	})

	// ── Pub/sub introspection (proxied from gateway) ──────────────────────

	// Bus metrics: forwards gateway's /pubsub/metrics
	mux.HandleFunc("/api/v1/portfolio/pubsub/metrics", func(w http.ResponseWriter, r *http.Request) {
		portfolioProxyGateway(w, "/api/v1/gateway/pubsub/metrics", map[string]interface{}{
			"error": "gateway unavailable",
		})
	})

	// Dead letters for portfolio namespace
	mux.HandleFunc("/api/v1/portfolio/pubsub/dead-letters", func(w http.ResponseWriter, r *http.Request) {
		limit := r.URL.Query().Get("limit")
		if limit == "" {
			limit = "20"
		}
		portfolioProxyGateway(w,
			"/api/v1/gateway/pubsub/dead-letters?limit="+limit,
			map[string]interface{}{"dead_letters": []interface{}{}})
	})

	// Replay: forwards to gateway replay filtered to portfolio prefix
	mux.HandleFunc("/api/v1/portfolio/pubsub/replay", func(w http.ResponseWriter, r *http.Request) {
		from := r.URL.Query().Get("from")
		to := r.URL.Query().Get("to")
		topic := r.URL.Query().Get("topic")
		q := "prefix=portfolio."
		if topic != "" {
			q = "topic=" + topic
		}
		if from != "" {
			q += "&from=" + from
		}
		if to != "" {
			q += "&to=" + to
		}
		portfolioProxyGateway(w,
			"/api/v1/gateway/pubsub/replay?"+q,
			map[string]interface{}{"events": []interface{}{}})
	})

	// Mesh status: what the gateway mesh registry knows about this service
	mux.HandleFunc("/api/v1/portfolio/mesh", func(w http.ResponseWriter, r *http.Request) {
		portfolioProxyGateway(w,
			"/api/v1/gateway/components/id/"+portfolioServiceID,
			map[string]interface{}{"error": "gateway unavailable"})
	})

	// Network messages sent to/from this service
	mux.HandleFunc("/api/v1/portfolio/mesh/messages", func(w http.ResponseWriter, r *http.Request) {
		limit := r.URL.Query().Get("limit")
		if limit == "" {
			limit = "50"
		}
		portfolioProxyGateway(w,
			"/api/v1/gateway/network/history?source="+portfolioServiceID+"&limit="+limit,
			map[string]interface{}{"messages": []interface{}{}})
	})

	log.Printf("%s listening on %s", portfolioServiceName, addr)
	log.Fatal(http.ListenAndServe(addr, mux))
}
