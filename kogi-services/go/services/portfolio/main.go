package main

import (
    "encoding/json"
    "log"
    "net/http"
    "os"
    "strings"
)

type portfolioItem struct {
    ID     string `json:"id"`
    Type   string `json:"type"`
    Name   string `json:"name"`
    Status string `json:"status"`
}

func main() {
    mux := http.NewServeMux()

    mux.HandleFunc("/health", func(w http.ResponseWriter, r *http.Request) {
        writeJSON(w, http.StatusOK, map[string]string{"status": "ok", "service": "portfolio-service"})
    })

    mux.HandleFunc("/api/v1/portfolio/root", func(w http.ResponseWriter, r *http.Request) {
        writeJSON(w, http.StatusOK, map[string]interface{}{
            "workspace_id": "ws-root",
            "items": []portfolioItem{
                {ID: "port-1", Type: "project", Name: "Kogi MVP", Status: "active"},
                {ID: "port-2", Type: "asset", Name: "Design Tokens", Status: "active"},
            },
        })
    })

    addr := resolveAddr("9002", "KOGI_PORTFOLIO_PORT")
    log.Printf("portfolio-service listening on %s", addr)
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
