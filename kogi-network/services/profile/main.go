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
    rt := ops.Init("profile-service")
    rt.State("init")
    mux := http.NewServeMux()
    rt.State("configure")
    rt.WatchSignals()

    mux.HandleFunc("/health", func(w http.ResponseWriter, r *http.Request) {
        writeJSON(w, http.StatusOK, map[string]string{"status": "ok", "service": "profile-service"})
    })

    mux.HandleFunc("/api/v1/profile/runtime", func(w http.ResponseWriter, r *http.Request) {
        writeJSON(w, http.StatusOK, map[string]interface{}{
            "service":         "profile-service",
            "component_id":    "kogi.network.profile",
            "module_id":       "kogi.profile",
            "gateway":         "http://127.0.0.1:8090",
            "network_manager": "kogi-go-network",
            "publishes": []string{
                "profile.settings.updated",
                "profile.persona.updated",
            },
            "subscribes": []string{
                "ims.profile.updated",
                "auth.session.created",
            },
            "engine_stream": "engine.ingest",
        })
    })

    mux.HandleFunc("/api/v1/profile/settings", func(w http.ResponseWriter, r *http.Request) {
        writeJSON(w, http.StatusOK, map[string]interface{}{
            "profile_id": "profile-work-001",
            "settings": map[string]interface{}{
                "default_workspace": "work",
                "timezone":          "America/Chicago",
                "theme":             "light",
            },
        })
    })

    addr := resolveAddr("9012", "KOGI_PROFILE_PORT")
    rt.State("running")
    rt.Status("ok", "listening="+addr)
    log.Printf("profile-service listening on %s", addr)
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
