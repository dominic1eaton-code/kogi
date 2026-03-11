package main

import (
	"bytes"
	"context"
	"encoding/json"
	"errors"
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

	"kogi.services/lib/ops"
)

type databaseActor struct {
	ID     string   `json:"id"`
	Role   string   `json:"role"`
	Token  string   `json:"token"`
	Scopes []string `json:"scopes"`
}

type databaseRequest struct {
	RequestID     string            `json:"request_id"`
	Action        string            `json:"action"`
	Actor         *databaseActor    `json:"actor,omitempty"`
	Collection    string            `json:"collection,omitempty"`
	RecordID      string            `json:"record_id,omitempty"`
	Query         string            `json:"query,omitempty"`
	SQL           string            `json:"sql,omitempty"`
	Payload       map[string]any    `json:"payload,omitempty"`
	Options       map[string]string `json:"options,omitempty"`
	CorrelationID string            `json:"correlation_id,omitempty"`
}

var databaseRuntime *ops.Runtime

func main() {
	databaseRuntime = ops.Init("database-service")
	databaseRuntime.State("init")
	mux := http.NewServeMux()
	databaseRuntime.State("configure")
	databaseRuntime.WatchSignals()

	mux.HandleFunc("/health", func(w http.ResponseWriter, r *http.Request) {
		writeJSON(w, http.StatusOK, map[string]string{"status": "ok", "service": "database-service"})
	})

	mux.HandleFunc("/api/v1/database/runtime", func(w http.ResponseWriter, r *http.Request) {
		system := systemStatus(r.Context())
		writeJSON(w, http.StatusOK, map[string]interface{}{
			"service":         "database-service",
			"component_id":    "kogi.services.database",
			"module_id":       "kogi.database",
			"engine":          "postgres",
			"schema":          "kogi-database/postgres/schema.sql",
			"gateway":         "http://127.0.0.1:8090",
			"network_manager": "kogi-go-network",
			"system":          system,
			"system_bridge":   "rust",
			"publishes": []string{
				"database.query.executed",
				"database.ledger.updated",
				"database.snapshot.created",
				"database.backup.completed",
			},
			"subscribes": []string{
				"bank.ledger.posted",
				"exchange.trade.executed",
			},
		})
	})

	mux.HandleFunc("/api/v1/database/query", handleAction("query", http.MethodPost))
	mux.HandleFunc("/api/v1/database/records/create", handleAction("create", http.MethodPost))
	mux.HandleFunc("/api/v1/database/records/read", handleAction("read", http.MethodPost))
	mux.HandleFunc("/api/v1/database/records/update", handleAction("update", http.MethodPost))
	mux.HandleFunc("/api/v1/database/records/delete", handleAction("delete", http.MethodPost))
	mux.HandleFunc("/api/v1/database/snapshot", handleAction("snapshot", http.MethodPost))
	mux.HandleFunc("/api/v1/database/checkpoint", handleAction("checkpoint", http.MethodPost))
	mux.HandleFunc("/api/v1/database/backup", handleAction("backup", http.MethodPost))
	mux.HandleFunc("/api/v1/database/restore", handleAction("restore", http.MethodPost))
	mux.HandleFunc("/api/v1/database/scale", handleAction("scale", http.MethodPost))
	mux.HandleFunc("/api/v1/database/optimize", handleAction("optimize", http.MethodPost))
	mux.HandleFunc("/api/v1/database/access", handleAction("access_control", http.MethodPost))
	mux.HandleFunc("/api/v1/database/concurrency", handleAction("concurrency", http.MethodPost))
	mux.HandleFunc("/api/v1/database/storage", handleAction("storage", http.MethodGet))
	mux.HandleFunc("/api/v1/database/operation", handleOperation())

	addr := resolveAddr("9015", "KOGI_DATABASE_PORT")
	databaseRuntime.State("running")
	databaseRuntime.Status("ok", "listening="+addr)
	log.Printf("database-service listening on %s", addr)
	log.Fatal(http.ListenAndServe(addr, ops.WithHTTPDebug(databaseRuntime, mux)))
}

func handleAction(action string, method string) http.HandlerFunc {
	return func(w http.ResponseWriter, r *http.Request) {
		if r.Method != method {
			writeJSON(w, http.StatusMethodNotAllowed, map[string]string{"error": "method_not_allowed"})
			return
		}

		req := databaseRequest{Action: action}
		if r.Method == http.MethodPost {
			if err := decodeJSON(r, &req); err != nil {
				writeJSON(w, http.StatusBadRequest, map[string]string{"error": err.Error()})
				return
			}
			req.Action = action
		}

		req.Actor = resolveActor(r, req.Actor)
		req.RequestID = ensureRequestID(req.RequestID)
		if action == "query" && req.Query == "" && req.SQL == "" {
			req.SQL = "select 1"
		}

		response := executeDatabaseRequest(r.Context(), req)
		writeJSON(w, http.StatusOK, response)
	}
}

func handleOperation() http.HandlerFunc {
	return func(w http.ResponseWriter, r *http.Request) {
		if r.Method != http.MethodPost {
			writeJSON(w, http.StatusMethodNotAllowed, map[string]string{"error": "method_not_allowed"})
			return
		}

		var req databaseRequest
		if err := decodeJSON(r, &req); err != nil {
			writeJSON(w, http.StatusBadRequest, map[string]string{"error": err.Error()})
			return
		}
		if req.Action == "" {
			writeJSON(w, http.StatusBadRequest, map[string]string{"error": "action is required"})
			return
		}

		req.Actor = resolveActor(r, req.Actor)
		req.RequestID = ensureRequestID(req.RequestID)
		response := executeDatabaseRequest(r.Context(), req)
		writeJSON(w, http.StatusOK, response)
	}
}

func executeDatabaseRequest(ctx context.Context, req databaseRequest) map[string]interface{} {
	timeoutCtx, cancel := context.WithTimeout(ctx, 3*time.Second)
	defer cancel()

	response, err := callDatabaseSystem(timeoutCtx, req)
	if err != nil {
		if databaseRuntime != nil {
			databaseRuntime.Debugf("message receive action=%s request_id=%s status=degraded err=%s", req.Action, req.RequestID, err.Error())
		}
		response = fallbackResponse(req, err)
	} else if databaseRuntime != nil {
		databaseRuntime.Debugf("message receive action=%s request_id=%s status=ok", req.Action, req.RequestID)
	}
	attachServiceMetadata(response)
	return response
}

func systemStatus(ctx context.Context) map[string]interface{} {
	req := databaseRequest{
		Action: "status",
		Actor: &databaseActor{
			ID:   "database-service",
			Role: "service",
		},
	}
	req.RequestID = ensureRequestID(req.RequestID)
	response, err := callDatabaseSystem(ctx, req)
	if err != nil {
		return fallbackResponse(req, err)
	}
	return response
}

func callDatabaseSystem(ctx context.Context, req databaseRequest) (map[string]interface{}, error) {
	binary, err := resolveSystemBinary()
	if err != nil {
		return nil, err
	}

	payload, err := json.Marshal(req)
	if err != nil {
		return nil, err
	}
	if databaseRuntime != nil {
		databaseRuntime.Message("send", "database."+req.Action, "database-service", "kogi.database", string(payload))
	}

	cmd := exec.CommandContext(ctx, binary)
	cmd.Stdin = bytes.NewReader(payload)
	var stdout bytes.Buffer
	var stderr bytes.Buffer
	cmd.Stdout = &stdout
	cmd.Stderr = &stderr
	if err := cmd.Run(); err != nil {
		return nil, fmt.Errorf("database system failed: %w %s", err, strings.TrimSpace(stderr.String()))
	}

	var response map[string]interface{}
	if err := json.Unmarshal(stdout.Bytes(), &response); err != nil {
		return nil, fmt.Errorf("invalid database system response: %w", err)
	}
	if stderr.Len() > 0 {
		response["system_warning"] = strings.TrimSpace(stderr.String())
	}
	return response, nil
}

func resolveSystemBinary() (string, error) {
	if path := os.Getenv("KOGI_DATABASE_SYSTEM_BIN"); path != "" {
		return path, nil
	}
	if path, err := exec.LookPath("kogi-database-system"); err == nil {
		return path, nil
	}

	if root, ok := findRepoRoot(); ok {
		exeName := "kogi-database-system"
		if runtime.GOOS == "windows" {
			exeName += ".exe"
		}
		candidates := []string{
			filepath.Join(root, "kogi-modules", "database", "target", "debug", exeName),
			filepath.Join(root, "kogi-modules", "database", "target", "release", exeName),
			filepath.Join(root, "kogi-modules", "database", exeName),
		}
		for _, candidate := range candidates {
			if fileExists(candidate) {
				return candidate, nil
			}
		}
	}

	return "", errors.New("kogi-database-system binary not found; set KOGI_DATABASE_SYSTEM_BIN or build kogi-modules/database")
}

func findRepoRoot() (string, bool) {
	cwd, err := os.Getwd()
	if err != nil {
		return "", false
	}

	current := cwd
	for i := 0; i < 8; i++ {
		if pathExists(filepath.Join(current, "kogi-modules")) || pathExists(filepath.Join(current, "go.work")) {
			return current, true
		}
		parent := filepath.Dir(current)
		if parent == current {
			break
		}
		current = parent
	}
	return "", false
}

func pathExists(path string) bool {
	_, err := os.Stat(path)
	return err == nil
}

func fileExists(path string) bool {
	info, err := os.Stat(path)
	if err != nil {
		return false
	}
	return !info.IsDir()
}

func fallbackResponse(req databaseRequest, err error) map[string]interface{} {
	return map[string]interface{}{
		"status":      "degraded",
		"action":      req.Action,
		"request_id":  ensureRequestID(req.RequestID),
		"message":     "database system unavailable",
		"error":       err.Error(),
		"timestamp_ms": time.Now().UTC().UnixMilli(),
		"data": map[string]interface{}{
			"fallback": true,
		},
	}
}

func attachServiceMetadata(response map[string]interface{}) {
	response["service"] = "database-service"
	response["component_id"] = "kogi.services.database"
	response["module_id"] = "kogi.database"
}

func resolveActor(r *http.Request, actor *databaseActor) *databaseActor {
	if actor != nil && (actor.ID != "" || actor.Role != "" || actor.Token != "") {
		return actor
	}

	resolved := &databaseActor{
		ID:    r.Header.Get("X-Actor-Id"),
		Role:  r.Header.Get("X-Actor-Role"),
		Token: r.Header.Get("X-Actor-Token"),
	}
	if scopes := r.Header.Get("X-Actor-Scopes"); scopes != "" {
		for _, scope := range strings.Split(scopes, ",") {
			trimmed := strings.TrimSpace(scope)
			if trimmed != "" {
				resolved.Scopes = append(resolved.Scopes, trimmed)
			}
		}
	}

	if resolved.ID == "" {
		resolved.ID = "database-service"
	}
	if resolved.Role == "" {
		resolved.Role = "service"
	}
	return resolved
}

func ensureRequestID(value string) string {
	if value != "" {
		return value
	}
	return fmt.Sprintf("req-%d", time.Now().UTC().UnixNano())
}

func decodeJSON(r *http.Request, target interface{}) error {
	defer r.Body.Close()
	decoder := json.NewDecoder(r.Body)
	if err := decoder.Decode(target); err != nil {
		if errors.Is(err, io.EOF) {
			return nil
		}
		return fmt.Errorf("invalid JSON: %w", err)
	}
	return nil
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
