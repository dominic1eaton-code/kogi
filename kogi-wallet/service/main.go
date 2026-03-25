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
	walletServiceName = "kogi-wallet-service"
	walletDefaultPort = "9013"
)

var walletDLLConfig = serviceutil.RustBridgeConfig{
	Mode:          serviceutil.RustModeDLL,
	EnvBinKey:     "KOGI_WALLET_DLL",
	WellKnownName: "kogi_wallet",
	RepoSubPaths: []string{
		filepath.Join("kogi-wallet", "target", "release"),
		filepath.Join("kogi-wallet", "target", "debug"),
		filepath.Join("kogi-wallet"),
	},
	ServiceLabel: "kogi-wallet",
	Timeout:      5 * time.Second,
}

var walletExeConfig = serviceutil.RustBridgeConfig{
	Mode:          serviceutil.RustModeExe,
	EnvBinKey:     "KOGI_WALLET_SYSTEM_BIN",
	WellKnownName: "kogi-wallet-system",
	RepoSubPaths: []string{
		filepath.Join("kogi-wallet", "target", "debug"),
		filepath.Join("kogi-wallet", "target", "release"),
		filepath.Join("kogi-wallet"),
	},
	ServiceLabel: "kogi-wallet-exe",
	Timeout:      5 * time.Second,
}

func callWallet(funcName string, payload interface{}, fallback interface{}) interface{} {
	res, err := serviceutil.CallRust(walletDLLConfig, funcName, payload)
	if err != nil {
		res, err = serviceutil.CallRust(walletExeConfig, funcName, payload)
		if err != nil {
			log.Printf("[%s] rust call %s err=%v", walletServiceName, funcName, err)
			return fallback
		}
	}
	return res
}

func writeJSON(w http.ResponseWriter, status int, payload interface{}) {
	serviceutil.WriteJSON(w, status, payload)
}

func readJSON(r *http.Request, dst interface{}) error {
	return serviceutil.DecodeBody(r, dst)
}

func emptyPayload() map[string]interface{} {
	return map[string]interface{}{}
}

func main() {
	mux := http.NewServeMux()

	mux.HandleFunc("/health", func(w http.ResponseWriter, r *http.Request) {
		result := callWallet("wallet_health", emptyPayload(), map[string]interface{}{
			"ok": false,
			"service": walletServiceName,
		})
		writeJSON(w, http.StatusOK, result)
	})

	mux.HandleFunc("/api/v1/wallet/init", func(w http.ResponseWriter, r *http.Request) {
		if r.Method != http.MethodPost {
			writeJSON(w, http.StatusMethodNotAllowed, map[string]string{"error": "POST required"})
			return
		}
		var payload map[string]interface{}
		if err := readJSON(r, &payload); err != nil {
			writeJSON(w, http.StatusBadRequest, map[string]string{"error": err.Error()})
			return
		}
		result := callWallet("wallet_init", payload, map[string]interface{}{"ok": false})
		writeJSON(w, http.StatusOK, result)
	})

	mux.HandleFunc("/api/v1/wallet/dashboard", func(w http.ResponseWriter, r *http.Request) {
		result := callWallet("wallet_dashboard_snapshot", emptyPayload(), map[string]interface{}{})
		writeJSON(w, http.StatusOK, result)
	})

	mux.HandleFunc("/api/v1/wallet/banking", func(w http.ResponseWriter, r *http.Request) {
		result := callWallet("wallet_banking_snapshot", emptyPayload(), map[string]interface{}{})
		writeJSON(w, http.StatusOK, result)
	})

	mux.HandleFunc("/api/v1/wallet/ledger", func(w http.ResponseWriter, r *http.Request) {
		result := callWallet("wallet_ledger_snapshot", emptyPayload(), map[string]interface{}{})
		writeJSON(w, http.StatusOK, result)
	})

	mux.HandleFunc("/api/v1/wallet/escrow", func(w http.ResponseWriter, r *http.Request) {
		result := callWallet("wallet_escrow_snapshot", emptyPayload(), map[string]interface{}{})
		writeJSON(w, http.StatusOK, result)
	})

	mux.HandleFunc("/api/v1/wallet/invoices", func(w http.ResponseWriter, r *http.Request) {
		result := callWallet("wallet_invoices_snapshot", emptyPayload(), map[string]interface{}{})
		writeJSON(w, http.StatusOK, result)
	})

	mux.HandleFunc("/api/v1/wallet/investments", func(w http.ResponseWriter, r *http.Request) {
		result := callWallet("wallet_investments_snapshot", emptyPayload(), map[string]interface{}{})
		writeJSON(w, http.StatusOK, result)
	})

	mux.HandleFunc("/api/v1/wallet/funding", func(w http.ResponseWriter, r *http.Request) {
		result := callWallet("wallet_funding_snapshot", emptyPayload(), map[string]interface{}{})
		writeJSON(w, http.StatusOK, result)
	})

	mux.HandleFunc("/api/v1/wallet/benefits", func(w http.ResponseWriter, r *http.Request) {
		result := callWallet("wallet_benefits_snapshot", emptyPayload(), map[string]interface{}{})
		writeJSON(w, http.StatusOK, result)
	})

	mux.HandleFunc("/api/v1/wallet/grants", func(w http.ResponseWriter, r *http.Request) {
		result := callWallet("wallet_grants_snapshot", emptyPayload(), map[string]interface{}{})
		writeJSON(w, http.StatusOK, result)
	})

	mux.HandleFunc("/api/v1/wallet/group-economics", func(w http.ResponseWriter, r *http.Request) {
		result := callWallet("wallet_group_economics_snapshot", emptyPayload(), map[string]interface{}{})
		writeJSON(w, http.StatusOK, result)
	})

	mux.HandleFunc("/api/v1/wallet/campaigns", func(w http.ResponseWriter, r *http.Request) {
		result := callWallet("wallet_campaigns_snapshot", emptyPayload(), map[string]interface{}{})
		writeJSON(w, http.StatusOK, result)
	})

	mux.HandleFunc("/api/v1/wallet/debts", func(w http.ResponseWriter, r *http.Request) {
		result := callWallet("wallet_debts_snapshot", emptyPayload(), map[string]interface{}{})
		writeJSON(w, http.StatusOK, result)
	})

	mux.HandleFunc("/api/v1/wallet/taxes", func(w http.ResponseWriter, r *http.Request) {
		result := callWallet("wallet_taxes_snapshot", emptyPayload(), map[string]interface{}{})
		writeJSON(w, http.StatusOK, result)
	})

	mux.HandleFunc("/api/v1/wallet/wallets", func(w http.ResponseWriter, r *http.Request) {
		result := callWallet("wallet_wallets", emptyPayload(), []interface{}{})
		writeJSON(w, http.StatusOK, result)
	})

	mux.HandleFunc("/api/v1/wallet/accounts", func(w http.ResponseWriter, r *http.Request) {
		result := callWallet("wallet_accounts", emptyPayload(), []interface{}{})
		writeJSON(w, http.StatusOK, result)
	})

	mux.HandleFunc("/api/v1/wallet/wallets/overview", func(w http.ResponseWriter, r *http.Request) {
		result := callWallet("wallet_wallets_overview", emptyPayload(), map[string]interface{}{})
		writeJSON(w, http.StatusOK, result)
	})

	port := serviceutil.ResolveAddr(walletDefaultPort, "KOGI_WALLET_PORT")
	addr := serviceutil.ToListenAddr(port)
	log.Printf("[%s] listening on %s", walletServiceName, addr)
	if err := http.ListenAndServe(addr, mux); err != nil {
		log.Printf("[%s] server error: %v", walletServiceName, err)
		os.Exit(1)
	}
}
