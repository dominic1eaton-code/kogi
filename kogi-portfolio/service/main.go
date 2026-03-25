package main

import (
	"log"
	"net/http"
	"os"
	"path/filepath"
	"time"

	"kogi.network/lib/serviceutil"
)

const (
	portfolioServiceName = "kogi-portfolio-service"
	portfolioDefaultPort = "9012"
)

var portfolioDLLConfig = serviceutil.RustBridgeConfig{
	Mode:          serviceutil.RustModeDLL,
	EnvBinKey:     "KOGI_PORTFOLIO_DLL",
	WellKnownName: "kogi_portfolio",
	RepoSubPaths: []string{
		filepath.Join("kogi-portfolio", "target", "release"),
		filepath.Join("kogi-portfolio", "target", "debug"),
		filepath.Join("kogi-portfolio"),
	},
	ServiceLabel: "kogi-portfolio",
	Timeout:      5 * time.Second,
}

var portfolioExeConfig = serviceutil.RustBridgeConfig{
	Mode:          serviceutil.RustModeExe,
	EnvBinKey:     "KOGI_PORTFOLIO_SYSTEM_BIN",
	WellKnownName: "kogi-portfolio-system",
	RepoSubPaths: []string{
		filepath.Join("kogi-portfolio", "target", "debug"),
		filepath.Join("kogi-portfolio", "target", "release"),
		filepath.Join("kogi-portfolio"),
	},
	ServiceLabel: "kogi-portfolio-exe",
	Timeout:      5 * time.Second,
}

func callPortfolio(funcName string, payload interface{}, fallback interface{}) interface{} {
	res, err := serviceutil.CallRust(portfolioDLLConfig, funcName, payload)
	if err != nil {
		res, err = serviceutil.CallRust(portfolioExeConfig, funcName, payload)
		if err != nil {
			log.Printf("[%s] rust call %s err=%v", portfolioServiceName, funcName, err)
			return fallback
		}
	}
	return res
}

func readJSON(r *http.Request, dst interface{}) error {
	return serviceutil.DecodeBody(r, dst)
}

func writeJSON(w http.ResponseWriter, status int, payload interface{}) {
	serviceutil.WriteJSON(w, status, payload)
}

func emptyPayload() map[string]interface{} {
	return map[string]interface{}{}
}

func rootPayload(root string) map[string]interface{} {
	if root == "" {
		return emptyPayload()
	}
	return map[string]interface{}{"root_component_id": root}
}

func main() {
	mux := http.NewServeMux()

	mux.HandleFunc("/health", func(w http.ResponseWriter, r *http.Request) {
		result := callPortfolio("portfolio_health", emptyPayload(), map[string]interface{}{
			"ok": false,
			"service": portfolioServiceName,
		})
		writeJSON(w, http.StatusOK, result)
	})

	mux.HandleFunc("/api/v1/portfolio/init", func(w http.ResponseWriter, r *http.Request) {
		if r.Method != http.MethodPost {
			writeJSON(w, http.StatusMethodNotAllowed, map[string]string{"error": "POST required"})
			return
		}
		var payload map[string]interface{}
		if err := readJSON(r, &payload); err != nil {
			writeJSON(w, http.StatusBadRequest, map[string]string{"error": err.Error()})
			return
		}
		result := callPortfolio("portfolio_init", payload, map[string]interface{}{"ok": false})
		writeJSON(w, http.StatusOK, result)
	})

	mux.HandleFunc("/api/v1/portfolio/state", func(w http.ResponseWriter, r *http.Request) {
		result := callPortfolio("portfolio_state", emptyPayload(), map[string]interface{}{})
		writeJSON(w, http.StatusOK, result)
	})

	mux.HandleFunc("/api/v1/portfolio/items", func(w http.ResponseWriter, r *http.Request) {
		result := callPortfolio("portfolio_items_view", emptyPayload(), map[string]interface{}{})
		writeJSON(w, http.StatusOK, result)
	})

	mux.HandleFunc("/api/v1/portfolio/dashboard", func(w http.ResponseWriter, r *http.Request) {
		result := callPortfolio("portfolio_dashboard_snapshot", emptyPayload(), map[string]interface{}{})
		writeJSON(w, http.StatusOK, result)
	})

	mux.HandleFunc("/api/v1/portfolio/analytics", func(w http.ResponseWriter, r *http.Request) {
		result := callPortfolio("portfolio_analytics_snapshot", emptyPayload(), map[string]interface{}{})
		writeJSON(w, http.StatusOK, result)
	})

	mux.HandleFunc("/api/v1/portfolio/registry", func(w http.ResponseWriter, r *http.Request) {
		result := callPortfolio("portfolio_registry_snapshot", emptyPayload(), map[string]interface{}{})
		writeJSON(w, http.StatusOK, result)
	})

	mux.HandleFunc("/api/v1/portfolio/link-forest", func(w http.ResponseWriter, r *http.Request) {
		root := r.URL.Query().Get("root_component_id")
		result := callPortfolio("portfolio_link_forest", rootPayload(root), map[string]interface{}{})
		writeJSON(w, http.StatusOK, result)
	})

	mux.HandleFunc("/api/v1/portfolio/link-forest/rows", func(w http.ResponseWriter, r *http.Request) {
		root := r.URL.Query().Get("root_component_id")
		result := callPortfolio("portfolio_link_forest_rows", rootPayload(root), []interface{}{})
		writeJSON(w, http.StatusOK, result)
	})

	mux.HandleFunc("/api/v1/portfolio/link-forest/sheet", func(w http.ResponseWriter, r *http.Request) {
		root := r.URL.Query().Get("root_component_id")
		result := callPortfolio("portfolio_link_forest_sheet", rootPayload(root), map[string]interface{}{})
		writeJSON(w, http.StatusOK, result)
	})

	port := serviceutil.ResolveAddr(portfolioDefaultPort, "KOGI_PORTFOLIO_PORT")
	addr := serviceutil.ToListenAddr(port)
	log.Printf("[%s] listening on %s", portfolioServiceName, addr)
	if err := http.ListenAndServe(addr, mux); err != nil {
		log.Printf("[%s] server error: %v", portfolioServiceName, err)
		os.Exit(1)
	}
}
