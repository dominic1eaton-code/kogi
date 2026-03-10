package main

import (
    "encoding/json"
    "log"
    "net/http"
    "os"
    "strings"
)

func main() {
    mux := http.NewServeMux()

    mux.HandleFunc("/health", func(w http.ResponseWriter, r *http.Request) {
        writeJSON(w, http.StatusOK, map[string]string{"status": "ok", "service": "developer-service"})
    })

    mux.HandleFunc("/api/v1/developer/runtime", func(w http.ResponseWriter, r *http.Request) {
        writeJSON(w, http.StatusOK, map[string]interface{}{
            "service":         "developer-service",
            "component_id":    "kogi.services.developer",
            "module_id":       "kogi.developer",
            "gateway":         "http://127.0.0.1:8090",
            "network_manager": "kogi-go-network",
            "publishes": []string{
                "developer.sdk.published",
                "developer.extension.updated",
            },
            "subscribes": []string{
                "community.message.posted",
                "organizations.role.updated",
            },
            "engine_stream": "engine.ingest",
        })
    })

    mux.HandleFunc("/api/v1/developer/sdk", func(w http.ResponseWriter, r *http.Request) {
        writeJSON(w, http.StatusOK, map[string]interface{}{
            "sdks": []map[string]string{
                {"id": "sdk-001", "name": "Kogi API", "status": "active"},
                {"id": "sdk-002", "name": "Kogi Events", "status": "active"},
            },
        })
    })

    addr := resolveAddr("9011", "KOGI_DEVELOPER_PORT")
    log.Printf("developer-service listening on %s", addr)
    log.Fatal(http.ListenAndServe(addr, mux))
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
