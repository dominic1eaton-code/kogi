package main

import (
    "encoding/json"
    "log"
    "net/http"
    "os"
    "strings"

    "kogi.services/lib/ops"
)

type studioIdea struct {
    ID     string `json:"id"`
    Title  string `json:"title"`
    Status string `json:"status"`
}

func main() {
    rt := ops.Init("studio-service")
    rt.State("init")
    mux := http.NewServeMux()
    rt.State("configure")
    rt.WatchSignals()

    mux.HandleFunc("/health", func(w http.ResponseWriter, r *http.Request) {
        writeJSON(w, http.StatusOK, map[string]string{"status": "ok", "service": "studio-service"})
    })

    mux.HandleFunc("/api/v1/studio/runtime", func(w http.ResponseWriter, r *http.Request) {
        writeJSON(w, http.StatusOK, map[string]interface{}{
            "service":         "studio-service",
            "component_id":    "kogi.services.studio",
            "module_id":       "kogi.studio",
            "gateway":         "http://127.0.0.1:8090",
            "network_manager": "kogi-go-network",
            "publishes": []string{
                "studio.idea.created",
                "studio.prototype.updated",
            },
            "subscribes": []string{
                "developer.extension.updated",
                "office.workspace.story.created",
            },
            "engine_stream": "engine.ingest",
        })
    })

    mux.HandleFunc("/api/v1/studio/ideas", func(w http.ResponseWriter, r *http.Request) {
        writeJSON(w, http.StatusOK, map[string]interface{}{
            "ideas": []studioIdea{
                {ID: "idea-001", Title: "Autonomous workspace assistant", Status: "exploring"},
                {ID: "idea-002", Title: "Portfolio storytelling kit", Status: "active"},
            },
        })
    })

    addr := resolveAddr("9009", "KOGI_STUDIO_PORT")
    rt.State("running")
    rt.Status("ok", "listening="+addr)
    log.Printf("studio-service listening on %s", addr)
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
