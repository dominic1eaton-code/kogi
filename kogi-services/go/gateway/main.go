package main

import (
    "encoding/json"
    "log"
    "net/http"
    "os"
    "strings"
    "time"

    "kogi.services/lib/eventbus"
)

type gatewayInfo struct {
    Name      string   `json:"name"`
    Version   string   `json:"version"`
    Services  []string `json:"services"`
    Timestamp string   `json:"timestamp"`
}

func main() {
    bus := eventbus.New()
    bus.Subscribe("gateway.request", func(e eventbus.Event) {
        log.Printf("event topic=%s payload=%s", e.Topic, e.Payload)
    })

    mux := http.NewServeMux()

    mux.HandleFunc("/health", func(w http.ResponseWriter, r *http.Request) {
        writeJSON(w, http.StatusOK, map[string]interface{}{"status": "ok", "service": "kogi-go-gateway"})
    })

    mux.HandleFunc("/api/v1/gateway/info", func(w http.ResponseWriter, r *http.Request) {
        bus.Publish("gateway.request", r.URL.Path)
        writeJSON(w, http.StatusOK, gatewayInfo{
            Name:      "kogi-go-gateway",
            Version:   "0.1.0",
            Services:  []string{"auth", "portfolio", "exchange", "ims", "office"},
            Timestamp: time.Now().UTC().Format(time.RFC3339),
        })
    })

    mux.HandleFunc("/api/v1/gateway/routes", func(w http.ResponseWriter, r *http.Request) {
        writeJSON(w, http.StatusOK, map[string]interface{}{
            "gateway": "kogi-go-gateway",
            "routes": map[string]string{
                "auth":      "/services/auth",
                "portfolio": "/services/portfolio",
                "exchange":  "/services/exchange",
                "ims":       "/services/ims",
                "office":    "/services/office",
            },
            "discovery": "kogi office module enabled",
        })
    })

    addr := resolveAddr("8090", "KOGI_GATEWAY_PORT")
    log.Printf("kogi-go-gateway listening on %s", addr)
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
