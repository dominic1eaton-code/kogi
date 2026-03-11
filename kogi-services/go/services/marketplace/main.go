package main

import (
    "encoding/json"
    "log"
    "net/http"
    "os"
    "strings"

    "kogi.services/lib/ops"
)

type listing struct {
    ID     string `json:"id"`
    Title  string `json:"title"`
    Status string `json:"status"`
}

func main() {
    rt := ops.Init("marketplace-service")
    rt.State("init")
    mux := http.NewServeMux()
    rt.State("configure")
    rt.WatchSignals()

    mux.HandleFunc("/health", func(w http.ResponseWriter, r *http.Request) {
        writeJSON(w, http.StatusOK, map[string]string{"status": "ok", "service": "marketplace-service"})
    })

    mux.HandleFunc("/api/v1/marketplace/runtime", func(w http.ResponseWriter, r *http.Request) {
        writeJSON(w, http.StatusOK, map[string]interface{}{
            "service":         "marketplace-service",
            "component_id":    "kogi.services.marketplace",
            "module_id":       "kogi.marketplace",
            "gateway":         "http://127.0.0.1:8090",
            "network_manager": "kogi-go-network",
            "publishes": []string{
                "marketplace.listing.created",
                "marketplace.match.found",
            },
            "subscribes": []string{
                "profile.settings.updated",
                "community.room.updated",
            },
            "engine_stream": "engine.ingest",
        })
    })

    mux.HandleFunc("/api/v1/marketplace/listings", func(w http.ResponseWriter, r *http.Request) {
        writeJSON(w, http.StatusOK, map[string]interface{}{
            "listings": []listing{
                {ID: "lst-001", Title: "Design system audit", Status: "open"},
                {ID: "lst-002", Title: "Operations playbook", Status: "active"},
            },
        })
    })

    addr := resolveAddr("9008", "KOGI_MARKETPLACE_PORT")
    rt.State("running")
    rt.Status("ok", "listening="+addr)
    log.Printf("marketplace-service listening on %s", addr)
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
