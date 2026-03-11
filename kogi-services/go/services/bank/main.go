package main

import (
    "encoding/json"
    "log"
    "net/http"
    "os"
    "strings"

    "kogi.services/lib/ops"
)

func main() {
    rt := ops.Init("bank-service")
    rt.State("init")
    mux := http.NewServeMux()
    rt.State("configure")
    rt.WatchSignals()

    mux.HandleFunc("/health", func(w http.ResponseWriter, r *http.Request) {
        writeJSON(w, http.StatusOK, map[string]string{"status": "ok", "service": "bank-service"})
    })

    mux.HandleFunc("/api/v1/bank/runtime", func(w http.ResponseWriter, r *http.Request) {
        writeJSON(w, http.StatusOK, map[string]interface{}{
            "service":         "bank-service",
            "component_id":    "kogi.services.bank",
            "module_id":       "kogi.bank",
            "gateway":         "http://127.0.0.1:8090",
            "network_manager": "kogi-go-network",
            "publishes": []string{
                "bank.wallet.updated",
                "bank.ledger.posted",
            },
            "subscribes": []string{
                "exchange.trade.executed",
                "portfolio.item.created",
            },
            "engine_stream": "engine.ingest",
        })
    })

    mux.HandleFunc("/api/v1/bank/ledger", func(w http.ResponseWriter, r *http.Request) {
        writeJSON(w, http.StatusOK, map[string]interface{}{
            "ledger_id": "ledger-001",
            "balances": map[string]float64{
                "operating": 14500.20,
                "reserve":   4200.00,
                "escrow":    860.50,
            },
            "status": "active",
        })
    })

    addr := resolveAddr("9007", "KOGI_BANK_PORT")
    rt.State("running")
    rt.Status("ok", "listening="+addr)
    log.Printf("bank-service listening on %s", addr)
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
