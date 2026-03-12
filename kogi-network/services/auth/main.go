package main

import (
	"encoding/json"
	"log"
	"net/http"
	"os"
	"strings"

	"kogi.network/lib/ops"
)

func main() {
	rt := ops.Init("auth-service")
	rt.State("init")
	mux := http.NewServeMux()
	rt.State("configure")
	rt.WatchSignals()

	mux.HandleFunc("/health", func(w http.ResponseWriter, r *http.Request) {
		writeJSON(w, http.StatusOK, map[string]string{"status": "ok", "service": "auth-service"})
	})
	mux.HandleFunc("/api/v1/auth/runtime", func(w http.ResponseWriter, r *http.Request) {
		writeJSON(w, http.StatusOK, map[string]interface{}{
			"service":         "auth-service",
			"component_id":    "kogi.network.auth",
			"gateway":         "http://127.0.0.1:8090",
			"network_manager": "kogi-go-network",
			"publishes": []string{
				"auth.login.request",
				"auth.session.created",
			},
			"subscribes": []string{
				"ims.identity.created",
				"host.orchestrator.health.checked",
			},
			"engine_stream": "engine.ingest",
		})
	})
	mux.HandleFunc("/api/v1/auth/provision", func(w http.ResponseWriter, r *http.Request) {
		writeJSON(w, http.StatusAccepted, map[string]string{
			"result": "queued",
			"flow":   "account->workspace->portfolio->wallet",
		})
	})

	addr := resolveAddr("9001", "KOGI_AUTH_PORT")
	rt.State("running")
	rt.Status("ok", "listening="+addr)
	log.Printf("auth-service listening on %s", addr)
	log.Fatal(http.ListenAndServe(addr, ops.WithHTTPDebug(rt, mux)))
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
