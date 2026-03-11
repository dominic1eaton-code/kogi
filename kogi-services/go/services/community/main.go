package main

import (
    "encoding/json"
    "log"
    "net/http"
    "os"
    "strings"

    "kogi.services/lib/ops"
)

type communityRoom struct {
    ID     string `json:"id"`
    Name   string `json:"name"`
    Status string `json:"status"`
}

func main() {
    rt := ops.Init("community-service")
    rt.State("init")
    mux := http.NewServeMux()
    rt.State("configure")
    rt.WatchSignals()

    mux.HandleFunc("/health", func(w http.ResponseWriter, r *http.Request) {
        writeJSON(w, http.StatusOK, map[string]string{"status": "ok", "service": "community-service"})
    })

    mux.HandleFunc("/api/v1/community/runtime", func(w http.ResponseWriter, r *http.Request) {
        writeJSON(w, http.StatusOK, map[string]interface{}{
            "service":         "community-service",
            "component_id":    "kogi.services.community",
            "module_id":       "kogi.community",
            "gateway":         "http://127.0.0.1:8090",
            "network_manager": "kogi-go-network",
            "publishes": []string{
                "community.room.updated",
                "community.message.posted",
            },
            "subscribes": []string{
                "profile.persona.updated",
                "marketplace.listing.created",
            },
            "engine_stream": "engine.ingest",
        })
    })

    mux.HandleFunc("/api/v1/community/rooms", func(w http.ResponseWriter, r *http.Request) {
        writeJSON(w, http.StatusOK, map[string]interface{}{
            "rooms": []communityRoom{
                {ID: "room-001", Name: "Kogi Founders", Status: "active"},
                {ID: "room-002", Name: "Coop Ops", Status: "active"},
            },
        })
    })

    addr := resolveAddr("9010", "KOGI_COMMUNITY_PORT")
    rt.State("running")
    rt.Status("ok", "listening="+addr)
    log.Printf("community-service listening on %s", addr)
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
