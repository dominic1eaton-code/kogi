package main

import (
    "encoding/json"
    "log"
    "net/http"
    "os"
    "strings"

    "kogi.network/lib/ops"
)

type organization struct {
    ID     string `json:"id"`
    Name   string `json:"name"`
    Status string `json:"status"`
}

func main() {
    rt := ops.Init("organizations-service")
    rt.State("init")
    mux := http.NewServeMux()
    rt.State("configure")
    rt.WatchSignals()

    mux.HandleFunc("/health", func(w http.ResponseWriter, r *http.Request) {
        writeJSON(w, http.StatusOK, map[string]string{"status": "ok", "service": "organizations-service"})
    })

    mux.HandleFunc("/api/v1/organizations/runtime", func(w http.ResponseWriter, r *http.Request) {
        writeJSON(w, http.StatusOK, map[string]interface{}{
            "service":         "organizations-service",
            "component_id":    "kogi.network.organizations",
            "module_id":       "kogi.organizations",
            "gateway":         "http://127.0.0.1:8090",
            "network_manager": "kogi-go-network",
            "publishes": []string{
                "organizations.role.updated",
                "organizations.proposal.created",
            },
            "subscribes": []string{
                "bank.ledger.posted",
                "community.room.updated",
            },
            "engine_stream": "engine.ingest",
        })
    })

    mux.HandleFunc("/api/v1/organizations/teams", func(w http.ResponseWriter, r *http.Request) {
        writeJSON(w, http.StatusOK, map[string]interface{}{
            "organizations": []organization{
                {ID: "org-001", Name: "Kogi Cooperative", Status: "active"},
                {ID: "org-002", Name: "Independent Workers Guild", Status: "active"},
            },
        })
    })

    addr := resolveAddr("9013", "KOGI_ORGANIZATIONS_PORT")
    rt.State("running")
    rt.Status("ok", "listening="+addr)
    log.Printf("organizations-service listening on %s", addr)
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
