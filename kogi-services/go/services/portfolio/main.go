package main

// portfolio_service.go
//
// HTTP service that bridges the Go network layer to the Rust PortfolioSystem
// via kogi_office.dll (or kogi-portfolio-system exe as a fallback).
//
// All utility functions (Rust bridge, DLL loading, HTTP helpers, gateway
// pub/sub, health reporting, file-system helpers) live in utility.go.
//
// Endpoints exposed (all under /api/v1/portfolio/…):
//   GET  /health
//   GET  /api/v1/portfolio/runtime
//   GET  /api/v1/portfolio/overview
//   GET  /api/v1/portfolio/snapshot
//   GET  /api/v1/portfolio/metadata
//   GET  /api/v1/portfolio/components
//   POST /api/v1/portfolio/components
//   GET  /api/v1/portfolio/components/{id}
//   PUT  /api/v1/portfolio/components/{id}
//   DELETE /api/v1/portfolio/components/{id}
//   GET  /api/v1/portfolio/components/type/{type}
//   POST /api/v1/portfolio/books
//   POST /api/v1/portfolio/active
//   POST /api/v1/portfolio/graph/hierarchy
//   DELETE /api/v1/portfolio/graph/hierarchy
//   POST /api/v1/portfolio/graph/dependency
//   DELETE /api/v1/portfolio/graph/dependency
//   POST /api/v1/portfolio/graph/link
//   DELETE /api/v1/portfolio/graph/link
//   POST /api/v1/portfolio/graph/member
//   DELETE /api/v1/portfolio/graph/member
//   GET  /api/v1/portfolio/graph/subtree/{id}
//   GET  /api/v1/portfolio/graph/dependencies/{id}
//   GET  /api/v1/portfolio/graph/order
//   GET  /api/v1/portfolio/snapshots
//   POST /api/v1/portfolio/snapshots
//   POST /api/v1/portfolio/snapshots/{id}/restore
//   GET  /api/v1/portfolio/checkpoints
//   POST /api/v1/portfolio/checkpoints
//   POST /api/v1/portfolio/checkpoints/{id}/restore
//   GET  /api/v1/portfolio/query   ?pql=…
//   GET  /api/v1/portfolio/events
//   POST /api/v1/portfolio/governance/policy/attach
//   POST /api/v1/portfolio/governance/policy/detach
//   POST /api/v1/portfolio/governance/approval/request
//   POST /api/v1/portfolio/governance/approval/{id}/resolve
//   POST /api/v1/portfolio/governance/resource/allocate
//   POST /api/v1/portfolio/governance/resource/consume
//   GET  /api/v1/portfolio/governance/resource/overruns
//   GET  /api/v1/portfolio/governance/resource/{id}
//   GET  /api/v1/portfolio/models/health/{id}
//   POST /api/v1/portfolio/models/project
//   POST /api/v1/portfolio/models/program
//   GET  /api/v1/portfolio/models/subportfolio/{id}
//   GET  /api/v1/portfolio/models/resource/{id}
//   POST /api/v1/portfolio/models/asset
//   GET/POST /api/v1/portfolio/models/artifact/{id}
//   GET/POST /api/v1/portfolio/models/binder/{id}
//   POST /api/v1/portfolio/models/book
//   GET/POST /api/v1/portfolio/models/folder/{id}
//   GET  /api/v1/portfolio/models/record/{id}
//   GET  /api/v1/portfolio/pubsub/metrics
//   GET  /api/v1/portfolio/pubsub/dead-letters
//   GET  /api/v1/portfolio/pubsub/replay
//   GET  /api/v1/portfolio/mesh
//   GET  /api/v1/portfolio/mesh/messages

import (
	"bytes"
	"fmt"
	"io"
	"log"
	"net/http"
	"strings"
	"time"
)

const (
	portfolioServiceID   = "kogi.services.portfolio"
	portfolioServiceName = "portfolio-service"
	portfolioDefaultPort = "9002"
	portfolioGateway     = "http://127.0.0.1:8090"
)

// portfolioPublish fires a pub/sub event to the gateway.
func portfolioPublish(topic, payload string, meta map[string]string) {
	publish(portfolioGateway, portfolioServiceID, topic, payload, meta)
}

// portfolioGatewaySubscribePrefix registers a prefix subscription.
func portfolioGatewaySubscribePrefix(prefix, consumer string) {
	gatewaySubscribePrefix(portfolioGateway, prefix, consumer)
}

// portfolioReportHealth sends a health record to the gateway.
func portfolioReportHealth(healthy bool, note string) {
	reportHealth(portfolioGateway, portfolioServiceID, healthy, note)
}

// portfolioHealthLoop reports health every 30 seconds.
func portfolioHealthLoop() {
	healthLoop(portfolioGateway, portfolioServiceID, 30*time.Second, func() (bool, string) {
		bin, err := portfolioResolveBinary()
		return err == nil, bin
	})
}

// portfolioDeadLetterMonitor polls the gateway dead-letter queue.
func portfolioDeadLetterMonitor() {
	pollDeadLetters(portfolioGateway,
		func(topic string) bool { return strings.HasPrefix(topic, "portfolio.") },
		func(id, topic string) { log.Printf("[portfolio-svc] dead-letter id=%s topic=%s", id, topic) },
	)
}

// portfolioInboundPoll polls the gateway receive endpoint for specific topics.
func portfolioInboundPoll(topics []string) {
	pollInbound(portfolioGateway, topics)
}

// portfolioRegisterWithGateway registers and subscribes this service.
func portfolioRegisterWithGateway(selfEndpoint string) {
	registerWithGateway(
		portfolioGateway, portfolioServiceID, selfEndpoint, "/health",
		[]string{
			"portfolio.item.created", "portfolio.component.updated",
			"portfolio.component.removed", "portfolio.graph.changed",
			"portfolio.snapshot.saved", "portfolio.checkpoint.created",
			"portfolio.governance.resource.allocated", "portfolio.governance.resource.consumed",
			"portfolio.governance.approval.requested", "portfolio.governance.approval.resolved",
			"portfolio.model.computed",
		},
		[]string{"office.dashboard.refresh", "ims.profile.updated", "exchange.trade.executed"},
		[]struct{ Prefix, Consumer string }{
			{"office.", portfolioServiceID},
			{"ims.", portfolioServiceID},
			{"exchange.trade", portfolioServiceID},
		},
	)
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
			"rust_bridge": "kogi_office.dll / kogi-portfolio-system", "rust_binary": bin,
			"dll_config": map[string]string{
				"office_dll":    PortfolioDLLConfig.EnvBinKey,
				"portfolio_dll": PortfolioDLLConfig.EnvBinKey,
			},
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
		writeJSON(w, http.StatusOK, portfolioRust("kogi_office_portfolio_snapshot", nil,
			map[string]interface{}{"module": "kogi.portfolio", "service": portfolioServiceName}))
	})

	mux.HandleFunc("/api/v1/portfolio/snapshot", func(w http.ResponseWriter, r *http.Request) {
		writeJSON(w, http.StatusOK, portfolioRust("kogi_portfolio_snapshot", nil,
			map[string]interface{}{"snapshot_id": "unavailable"}))
	})

	mux.HandleFunc("/api/v1/portfolio/metadata", func(w http.ResponseWriter, r *http.Request) {
		writeJSON(w, http.StatusOK, portfolioRust("kogi_portfolio_metadata", nil, map[string]interface{}{}))
	})

	// ── Components ────────────────────────────────────────────────────────
	mux.HandleFunc("/api/v1/portfolio/components", func(w http.ResponseWriter, r *http.Request) {
		switch r.Method {
		case http.MethodGet:
			writeJSON(w, http.StatusOK, map[string]interface{}{
				"components": portfolioRust("kogi_portfolio_all_components", nil, []interface{}{}),
			})
		case http.MethodPost:
			var req createComponentReq
			if err := decodeBody(r, &req); err != nil {
				writeJSON(w, http.StatusBadRequest, map[string]string{"error": err.Error()})
				return
			}
			result, err := portfolioCallRust("kogi_portfolio_create_component", req)
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

	mux.HandleFunc("/api/v1/portfolio/components/type/", func(w http.ResponseWriter, r *http.Request) {
		ct := trimPrefix(r, "/api/v1/portfolio/components/type/")
		writeJSON(w, http.StatusOK, map[string]interface{}{
			"type":       ct,
			"components": portfolioRust("kogi_portfolio_components_by_type", map[string]string{"component_type": ct}, []interface{}{}),
		})
	})

	mux.HandleFunc("/api/v1/portfolio/components/", func(w http.ResponseWriter, r *http.Request) {
		id := trimPrefix(r, "/api/v1/portfolio/components/")
		if id == "" {
			writeJSON(w, http.StatusBadRequest, map[string]string{"error": "component id required"})
			return
		}
		switch r.Method {
		case http.MethodGet:
			res := portfolioRust("kogi_portfolio_get_component", map[string]string{"id": id}, nil)
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
			result, err := portfolioCallRust("kogi_portfolio_edit_component", req)
			if err != nil {
				writeJSON(w, http.StatusInternalServerError, map[string]string{"error": err.Error()})
				return
			}
			go portfolioPublish("portfolio.component.updated", fmt.Sprintf(`{"id":%q}`, id), nil)
			writeJSON(w, http.StatusOK, result)
		case http.MethodDelete:
			result, err := portfolioCallRust("kogi_portfolio_remove_component", map[string]string{"id": id})
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
		result, err := portfolioCallRust("kogi_portfolio_create_book", req)
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
		result, err := portfolioCallRust("kogi_portfolio_set_active", map[string]string{"id": req.ComponentID})
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
		var fn string
		switch r.Method {
		case http.MethodPost:
			fn = "kogi_portfolio_add_hierarchy"
		case http.MethodDelete:
			fn = "kogi_portfolio_remove_hierarchy"
		default:
			writeJSON(w, http.StatusMethodNotAllowed, map[string]string{"error": "method_not_allowed"})
			return
		}
		result, err := portfolioCallRust(fn, req)
		if err != nil {
			writeJSON(w, http.StatusInternalServerError, map[string]string{"error": err.Error()})
			return
		}
		go portfolioPublish("portfolio.graph.changed",
			fmt.Sprintf(`{"action":%q,"parent_id":%q,"child_id":%q}`, fn, req.ParentID, req.ChildID), nil)
		writeJSON(w, http.StatusOK, result)
	})

	// ── Graph: dependency ─────────────────────────────────────────────────
	mux.HandleFunc("/api/v1/portfolio/graph/dependency", func(w http.ResponseWriter, r *http.Request) {
		var req dependencyReq
		if err := decodeBody(r, &req); err != nil {
			writeJSON(w, http.StatusBadRequest, map[string]string{"error": err.Error()})
			return
		}
		var fn string
		switch r.Method {
		case http.MethodPost:
			fn = "kogi_portfolio_add_dependency"
		case http.MethodDelete:
			fn = "kogi_portfolio_remove_dependency"
		default:
			writeJSON(w, http.StatusMethodNotAllowed, map[string]string{"error": "method_not_allowed"})
			return
		}
		result, err := portfolioCallRust(fn, req)
		if err != nil {
			writeJSON(w, http.StatusInternalServerError, map[string]string{"error": err.Error()})
			return
		}
		go portfolioPublish("portfolio.graph.changed",
			fmt.Sprintf(`{"action":%q,"from":%q,"to":%q}`, fn, req.FromID, req.ToID), nil)
		writeJSON(w, http.StatusOK, result)
	})

	// ── Graph: peer link ─────────────────────────────────────────────────
	mux.HandleFunc("/api/v1/portfolio/graph/link", func(w http.ResponseWriter, r *http.Request) {
		var req linkReq
		if err := decodeBody(r, &req); err != nil {
			writeJSON(w, http.StatusBadRequest, map[string]string{"error": err.Error()})
			return
		}
		var fn string
		switch r.Method {
		case http.MethodPost:
			fn = "kogi_portfolio_add_link"
		case http.MethodDelete:
			fn = "kogi_portfolio_remove_link"
		default:
			writeJSON(w, http.StatusMethodNotAllowed, map[string]string{"error": "method_not_allowed"})
			return
		}
		result, err := portfolioCallRust(fn, req)
		if err != nil {
			writeJSON(w, http.StatusInternalServerError, map[string]string{"error": err.Error()})
			return
		}
		go portfolioPublish("portfolio.graph.changed",
			fmt.Sprintf(`{"action":%q,"a":%q,"b":%q}`, fn, req.AID, req.BID), nil)
		writeJSON(w, http.StatusOK, result)
	})

	// ── Graph: container membership ───────────────────────────────────────
	mux.HandleFunc("/api/v1/portfolio/graph/member", func(w http.ResponseWriter, r *http.Request) {
		var req memberReq
		if err := decodeBody(r, &req); err != nil {
			writeJSON(w, http.StatusBadRequest, map[string]string{"error": err.Error()})
			return
		}
		var fn string
		switch r.Method {
		case http.MethodPost:
			fn = "kogi_portfolio_add_member"
		case http.MethodDelete:
			fn = "kogi_portfolio_remove_member"
		default:
			writeJSON(w, http.StatusMethodNotAllowed, map[string]string{"error": "method_not_allowed"})
			return
		}
		result, err := portfolioCallRust(fn, req)
		if err != nil {
			writeJSON(w, http.StatusInternalServerError, map[string]string{"error": err.Error()})
			return
		}
		go portfolioPublish("portfolio.graph.changed",
			fmt.Sprintf(`{"action":%q,"container":%q,"item":%q}`, fn, req.ContainerID, req.ItemID), nil)
		writeJSON(w, http.StatusOK, result)
	})

	// ── Graph traversal ───────────────────────────────────────────────────
	mux.HandleFunc("/api/v1/portfolio/graph/subtree/", func(w http.ResponseWriter, r *http.Request) {
		id := trimPrefix(r, "/api/v1/portfolio/graph/subtree/")
		writeJSON(w, http.StatusOK, portfolioRust("kogi_portfolio_subtree",
			map[string]string{"id": id}, map[string]interface{}{"root": id, "subtree": []interface{}{}}))
	})

	mux.HandleFunc("/api/v1/portfolio/graph/dependencies/", func(w http.ResponseWriter, r *http.Request) {
		id := trimPrefix(r, "/api/v1/portfolio/graph/dependencies/")
		writeJSON(w, http.StatusOK, portfolioRust("kogi_portfolio_transitive_dependencies",
			map[string]string{"id": id}, map[string]interface{}{"root": id, "dependencies": []interface{}{}}))
	})

	mux.HandleFunc("/api/v1/portfolio/graph/order", func(w http.ResponseWriter, r *http.Request) {
		writeJSON(w, http.StatusOK, portfolioRust("kogi_portfolio_dependency_order", nil,
			map[string]interface{}{"order": []interface{}{}}))
	})

	// ── Snapshots ─────────────────────────────────────────────────────────
	mux.HandleFunc("/api/v1/portfolio/snapshots", func(w http.ResponseWriter, r *http.Request) {
		switch r.Method {
		case http.MethodGet:
			writeJSON(w, http.StatusOK, map[string]interface{}{
				"snapshots": portfolioRust("kogi_portfolio_list_snapshots", nil, []interface{}{}),
			})
		case http.MethodPost:
			var req saveSnapshotReq
			_ = decodeBody(r, &req)
			result, err := portfolioCallRust("kogi_portfolio_save_snapshot", req)
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
		parts := strings.SplitN(trimPrefix(r, "/api/v1/portfolio/snapshots/"), "/", 2)
		if len(parts) != 2 || parts[1] != "restore" {
			writeJSON(w, http.StatusBadRequest, map[string]string{"error": "use /{id}/restore"})
			return
		}
		result, err := portfolioCallRust("kogi_portfolio_restore_snapshot", map[string]string{"snapshot_id": parts[0]})
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
				"checkpoints": portfolioRust("kogi_portfolio_list_checkpoints", nil, []interface{}{}),
			})
		case http.MethodPost:
			var req saveCheckpointReq
			if err := decodeBody(r, &req); err != nil {
				writeJSON(w, http.StatusBadRequest, map[string]string{"error": err.Error()})
				return
			}
			result, err := portfolioCallRust("kogi_portfolio_save_checkpoint", req)
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
		result, err := portfolioCallRust("kogi_portfolio_restore_checkpoint", map[string]string{"checkpoint_id": parts[0]})
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
		writeJSON(w, http.StatusOK, portfolioRust("kogi_portfolio_query_pql",
			map[string]string{"pql": pql},
			map[string]interface{}{"query": pql, "results": []interface{}{}}))
	})

	// ── Event log ─────────────────────────────────────────────────────────
	mux.HandleFunc("/api/v1/portfolio/events", func(w http.ResponseWriter, r *http.Request) {
		writeJSON(w, http.StatusOK, portfolioRust("kogi_portfolio_event_log", nil,
			map[string]interface{}{"events": []interface{}{}}))
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
		result, err := portfolioCallRust("kogi_portfolio_attach_policy", req)
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
		result, err := portfolioCallRust("kogi_portfolio_detach_policy", req)
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
		result, err := portfolioCallRust("kogi_portfolio_request_approval", req)
		if err != nil {
			writeJSON(w, http.StatusInternalServerError, map[string]string{"error": err.Error()})
			return
		}
		go portfolioPublish("portfolio.governance.approval.requested",
			fmt.Sprintf(`{"component_id":%q,"reason":%q}`, req.ComponentID, req.Reason), nil)
		writeJSON(w, http.StatusCreated, result)
	})

	mux.HandleFunc("/api/v1/portfolio/governance/approval/", func(w http.ResponseWriter, r *http.Request) {
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
		result, err := portfolioCallRust("kogi_portfolio_resolve_approval", req)
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
		result, err := portfolioCallRust("kogi_portfolio_allocate_resource", req)
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
		result, err := portfolioCallRust("kogi_portfolio_record_consumption", req)
		if err != nil {
			writeJSON(w, http.StatusInternalServerError, map[string]string{"error": err.Error()})
			return
		}
		go portfolioPublish("portfolio.governance.resource.consumed",
			fmt.Sprintf(`{"component_id":%q,"amount":%v}`, req.ComponentID, req.Amount), nil)
		writeJSON(w, http.StatusOK, result)
	})

	mux.HandleFunc("/api/v1/portfolio/governance/resource/overruns", func(w http.ResponseWriter, r *http.Request) {
		writeJSON(w, http.StatusOK, portfolioRust("kogi_portfolio_overrun_allocations", nil,
			map[string]interface{}{"overruns": []interface{}{}}))
	})

	mux.HandleFunc("/api/v1/portfolio/governance/resource/", func(w http.ResponseWriter, r *http.Request) {
		id := trimPrefix(r, "/api/v1/portfolio/governance/resource/")
		res := portfolioRust("kogi_portfolio_get_resource_allocation", map[string]string{"component_id": id}, nil)
		if res == nil {
			writeJSON(w, http.StatusNotFound, map[string]string{"error": "allocation not found"})
			return
		}
		writeJSON(w, http.StatusOK, res)
	})

	// ── Computational models ──────────────────────────────────────────────

	mux.HandleFunc("/api/v1/portfolio/models/health/", func(w http.ResponseWriter, r *http.Request) {
		id := trimPrefix(r, "/api/v1/portfolio/models/health/")
		res := portfolioRust("kogi_portfolio_compute_health", map[string]string{"portfolio_id": id}, nil)
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
		writeJSON(w, http.StatusOK, portfolioRust("kogi_portfolio_compute_project_metrics", req,
			map[string]interface{}{"error": "unavailable"}))
	})

	mux.HandleFunc("/api/v1/portfolio/models/program", func(w http.ResponseWriter, r *http.Request) {
		if r.Method != http.MethodPost {
			writeJSON(w, http.StatusMethodNotAllowed, map[string]string{"error": "method_not_allowed"})
			return
		}
		var body interface{}
		_ = decodeBody(r, &body)
		writeJSON(w, http.StatusOK, portfolioRust("kogi_portfolio_compute_program_alignment", body,
			map[string]interface{}{"error": "unavailable"}))
	})

	mux.HandleFunc("/api/v1/portfolio/models/subportfolio/", func(w http.ResponseWriter, r *http.Request) {
		id := trimPrefix(r, "/api/v1/portfolio/models/subportfolio/")
		res := portfolioRust("kogi_portfolio_compute_subportfolio_rollup",
			map[string]string{"subportfolio_id": id}, nil)
		if res == nil {
			writeJSON(w, http.StatusNotFound, map[string]string{"error": "not found"})
			return
		}
		writeJSON(w, http.StatusOK, res)
	})

	mux.HandleFunc("/api/v1/portfolio/models/resource/", func(w http.ResponseWriter, r *http.Request) {
		id := trimPrefix(r, "/api/v1/portfolio/models/resource/")
		res := portfolioRust("kogi_portfolio_compute_resource_utilisation",
			map[string]string{"resource_id": id}, nil)
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
		writeJSON(w, http.StatusOK, portfolioRust("kogi_portfolio_compute_asset_value", req,
			map[string]interface{}{"error": "unavailable"}))
	})

	mux.HandleFunc("/api/v1/portfolio/models/artifact/", func(w http.ResponseWriter, r *http.Request) {
		id := trimPrefix(r, "/api/v1/portfolio/models/artifact/")
		req := artifactMaturityReq{ArtifactID: id, MaxFreshDays: 30}
		if r.Method == http.MethodPost {
			_ = decodeBody(r, &req)
			req.ArtifactID = id
		}
		res := portfolioRust("kogi_portfolio_compute_artifact_maturity", req, nil)
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
		res := portfolioRust("kogi_portfolio_compute_binder_coverage", req, nil)
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
		writeJSON(w, http.StatusOK, portfolioRust("kogi_portfolio_compute_book_consistency", req,
			map[string]interface{}{"error": "unavailable"}))
	})

	mux.HandleFunc("/api/v1/portfolio/models/folder/", func(w http.ResponseWriter, r *http.Request) {
		id := trimPrefix(r, "/api/v1/portfolio/models/folder/")
		req := folderOrganisationReq{FolderID: id, DepthThreshold: 4}
		if r.Method == http.MethodPost {
			_ = decodeBody(r, &req)
			req.FolderID = id
		}
		res := portfolioRust("kogi_portfolio_compute_folder_organisation", req, nil)
		if res == nil {
			writeJSON(w, http.StatusNotFound, map[string]string{"error": "not found or not a folder"})
			return
		}
		writeJSON(w, http.StatusOK, res)
	})

	mux.HandleFunc("/api/v1/portfolio/models/record/", func(w http.ResponseWriter, r *http.Request) {
		id := trimPrefix(r, "/api/v1/portfolio/models/record/")
		res := portfolioRust("kogi_portfolio_compute_record_integrity",
			map[string]string{"record_id": id}, nil)
		if res == nil {
			writeJSON(w, http.StatusNotFound, map[string]string{"error": "not found or not a record"})
			return
		}
		writeJSON(w, http.StatusOK, res)
	})

	// ── Pub/sub introspection (proxied from gateway) ──────────────────────

	mux.HandleFunc("/api/v1/portfolio/pubsub/metrics", func(w http.ResponseWriter, r *http.Request) {
		portfolioProxyGateway(w, "/api/v1/gateway/pubsub/metrics",
			map[string]interface{}{"error": "gateway unavailable"})
	})

	mux.HandleFunc("/api/v1/portfolio/pubsub/dead-letters", func(w http.ResponseWriter, r *http.Request) {
		limit := r.URL.Query().Get("limit")
		if limit == "" {
			limit = "20"
		}
		portfolioProxyGateway(w, "/api/v1/gateway/pubsub/dead-letters?limit="+limit,
			map[string]interface{}{"dead_letters": []interface{}{}})
	})

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
		portfolioProxyGateway(w, "/api/v1/gateway/pubsub/replay?"+q,
			map[string]interface{}{"events": []interface{}{}})
	})

	mux.HandleFunc("/api/v1/portfolio/mesh", func(w http.ResponseWriter, r *http.Request) {
		portfolioProxyGateway(w, "/api/v1/gateway/components/id/"+portfolioServiceID,
			map[string]interface{}{"error": "gateway unavailable"})
	})

	mux.HandleFunc("/api/v1/portfolio/mesh/messages", func(w http.ResponseWriter, r *http.Request) {
		limit := r.URL.Query().Get("limit")
		if limit == "" {
			limit = "50"
		}
		portfolioProxyGateway(w,
			"/api/v1/gateway/network/history?source="+portfolioServiceID+"&limit="+limit,
			map[string]interface{}{"messages": []interface{}{}})
	})

	// ── Root redirect ─────────────────────────────────────────────────────
	mux.HandleFunc("/api/v1/portfolio/root", func(w http.ResponseWriter, r *http.Request) {
		writeJSON(w, http.StatusOK, portfolioRust("kogi_portfolio_snapshot", nil,
			map[string]interface{}{"snapshot_id": "unavailable"}))
	})

	log.Printf("%s listening on %s", portfolioServiceName, addr)
	log.Fatal(http.ListenAndServe(addr, mux))
}

// resolveAddr and helpers for this service use the shared utility versions.
// The unused import guard — bytes and io are used transitively via utility.go.
var _ = bytes.NewReader
var _ = io.ReadAll
