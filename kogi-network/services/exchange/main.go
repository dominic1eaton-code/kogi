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
	rt := ops.Init("exchange-service")
	rt.State("init")
	mux := http.NewServeMux()
	rt.State("configure")
	rt.WatchSignals()

	mux.HandleFunc("/health", func(w http.ResponseWriter, r *http.Request) {
		writeJSON(w, http.StatusOK, map[string]string{"status": "ok", "service": "exchange-service"})
	})

	mux.HandleFunc("/api/v1/exchange/runtime", func(w http.ResponseWriter, r *http.Request) {
		writeJSON(w, http.StatusOK, map[string]interface{}{
			"service":         "exchange-service",
			"component_id":    "kogi.network.exchange",
			"module_id":       "kogi.exchange",
			"gateway":         "http://127.0.0.1:8090",
			"network_manager": "kogi-go-network",
			"publishes": []string{
				"exchange.order.created",
				"exchange.trade.executed",
			},
			"subscribes": []string{
				"portfolio.item.created",
				"office.dashboard.refresh",
			},
			"engine_stream": "engine.ingest",
		})
	})

	mux.HandleFunc("/api/v1/exchange/wallet", func(w http.ResponseWriter, r *http.Request) {
		writeJSON(w, http.StatusOK, map[string]interface{}{
			"wallet_id": "wal-001",
			"balances": map[string]float64{
				"fiat_available": 1200.25,
				"fiat_reserved":  300.00,
				"credits":        90.0,
			},
		})
	})

	addr := resolveAddr("9004", "KOGI_EXCHANGE_PORT")
	rt.State("running")
	rt.Status("ok", "listening="+addr)
	log.Printf("exchange-service listening on %s", addr)
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
