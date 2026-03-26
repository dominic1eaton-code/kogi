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
	officeServiceName = "kogi-office-service"
	officeDefaultPort = "9015"
)

var officeDLLConfig = serviceutil.RustBridgeConfig{
	Mode:          serviceutil.RustModeDLL,
	EnvBinKey:     "KOGI_OFFICE_DLL",
	WellKnownName: "kogi_office",
	RepoSubPaths: []string{
		filepath.Join("kogi-office", "target", "release"),
		filepath.Join("kogi-office", "target", "debug"),
		filepath.Join("kogi-office"),
	},
	ServiceLabel: "kogi-office",
	Timeout:      5 * time.Second,
}

var officeExeConfig = serviceutil.RustBridgeConfig{
	Mode:          serviceutil.RustModeExe,
	EnvBinKey:     "KOGI_OFFICE_SYSTEM_BIN",
	WellKnownName: "kogi-office-system",
	RepoSubPaths: []string{
		filepath.Join("kogi-office", "target", "debug"),
		filepath.Join("kogi-office", "target", "release"),
		filepath.Join("kogi-office"),
	},
	ServiceLabel: "kogi-office-exe",
	Timeout:      5 * time.Second,
}

func callOffice(funcName string, payload interface{}, fallback interface{}) interface{} {
	res, err := serviceutil.CallRust(officeDLLConfig, funcName, payload)
	if err != nil {
		res, err = serviceutil.CallRust(officeExeConfig, funcName, payload)
		if err != nil {
			log.Printf("[%s] rust call %s err=%v", officeServiceName, funcName, err)
			return fallback
		}
	}
	return res
}

func callOfficeDirect(funcName string, payload interface{}) (interface{}, error) {
	res, err := serviceutil.CallRust(officeDLLConfig, funcName, payload)
	if err != nil {
		return serviceutil.CallRust(officeExeConfig, funcName, payload)
	}
	return res, nil
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
		result := callOffice("office_health", emptyPayload(), map[string]interface{}{
			"ok":      false,
			"service": officeServiceName,
		})
		writeJSON(w, http.StatusOK, result)
	})

	mux.HandleFunc("/api/v1/office/init", func(w http.ResponseWriter, r *http.Request) {
		if r.Method != http.MethodPost {
			writeJSON(w, http.StatusMethodNotAllowed, map[string]string{"error": "POST required"})
			return
		}
		var payload map[string]interface{}
		if err := readJSON(r, &payload); err != nil {
			writeJSON(w, http.StatusBadRequest, map[string]string{"error": err.Error()})
			return
		}
		result := callOffice("office_init", payload, map[string]interface{}{"ok": false})
		writeJSON(w, http.StatusOK, result)
	})

	mux.HandleFunc("/api/v1/office/state", func(w http.ResponseWriter, r *http.Request) {
		result := callOffice("office_state", emptyPayload(), map[string]interface{}{})
		writeJSON(w, http.StatusOK, result)
	})

	mux.HandleFunc("/api/v1/office/refresh", func(w http.ResponseWriter, r *http.Request) {
		if r.Method != http.MethodPost {
			writeJSON(w, http.StatusMethodNotAllowed, map[string]string{"error": "POST required"})
			return
		}
		result := callOffice("office_refresh", emptyPayload(), map[string]interface{}{})
		writeJSON(w, http.StatusOK, result)
	})

	mux.HandleFunc("/api/v1/office/work/items", func(w http.ResponseWriter, r *http.Request) {
		result := callOffice("office_work_items", emptyPayload(), map[string]interface{}{"items": []interface{}{}})
		writeJSON(w, http.StatusOK, result)
	})

	mux.HandleFunc("/api/v1/office/work/management", func(w http.ResponseWriter, r *http.Request) {
		result := callOffice("office_work_management", emptyPayload(), map[string]interface{}{"ok": false})
		writeJSON(w, http.StatusOK, result)
	})

	mux.HandleFunc("/api/v1/office/overview", func(w http.ResponseWriter, r *http.Request) {
		result := callOffice("office_overview_snapshot", emptyPayload(), map[string]interface{}{"error": "unavailable"})
		writeJSON(w, http.StatusOK, result)
	})

	mux.HandleFunc("/api/v1/office/inbox", func(w http.ResponseWriter, r *http.Request) {
		result := callOffice("office_inbox_snapshot", emptyPayload(), map[string]interface{}{"error": "unavailable"})
		writeJSON(w, http.StatusOK, result)
	})

	mux.HandleFunc("/api/v1/office/schedule", func(w http.ResponseWriter, r *http.Request) {
		result := callOffice("office_schedule_snapshot", emptyPayload(), map[string]interface{}{"error": "unavailable"})
		writeJSON(w, http.StatusOK, result)
	})

	mux.HandleFunc("/api/v1/office/calendar", func(w http.ResponseWriter, r *http.Request) {
		result := callOffice("office_calendar_snapshot", emptyPayload(), map[string]interface{}{"error": "unavailable"})
		writeJSON(w, http.StatusOK, result)
	})

	mux.HandleFunc("/api/v1/office/contacts", func(w http.ResponseWriter, r *http.Request) {
		result := callOffice("office_contacts_snapshot", emptyPayload(), map[string]interface{}{"error": "unavailable"})
		writeJSON(w, http.StatusOK, result)
	})

	mux.HandleFunc("/api/v1/office/studio/overview", func(w http.ResponseWriter, r *http.Request) {
		result := callOffice("office_studio_overview_snapshot", emptyPayload(), map[string]interface{}{"error": "unavailable"})
		writeJSON(w, http.StatusOK, result)
	})

	mux.HandleFunc("/api/v1/office/studio/ideas", func(w http.ResponseWriter, r *http.Request) {
		result := callOffice("office_studio_ideas_snapshot", emptyPayload(), map[string]interface{}{"error": "unavailable"})
		writeJSON(w, http.StatusOK, result)
	})

	mux.HandleFunc("/api/v1/office/studio/concepts", func(w http.ResponseWriter, r *http.Request) {
		result := callOffice("office_studio_concepts_snapshot", emptyPayload(), map[string]interface{}{"error": "unavailable"})
		writeJSON(w, http.StatusOK, result)
	})

	mux.HandleFunc("/api/v1/office/studio/designs", func(w http.ResponseWriter, r *http.Request) {
		result := callOffice("office_studio_designs_snapshot", emptyPayload(), map[string]interface{}{"error": "unavailable"})
		writeJSON(w, http.StatusOK, result)
	})

	mux.HandleFunc("/api/v1/office/studio/blueprints", func(w http.ResponseWriter, r *http.Request) {
		result := callOffice("office_studio_blueprints_snapshot", emptyPayload(), map[string]interface{}{"error": "unavailable"})
		writeJSON(w, http.StatusOK, result)
	})

	mux.HandleFunc("/api/v1/office/studio/mockups", func(w http.ResponseWriter, r *http.Request) {
		result := callOffice("office_studio_mockups_snapshot", emptyPayload(), map[string]interface{}{"error": "unavailable"})
		writeJSON(w, http.StatusOK, result)
	})

	mux.HandleFunc("/api/v1/office/studio/prototypes", func(w http.ResponseWriter, r *http.Request) {
		result := callOffice("office_studio_prototypes_snapshot", emptyPayload(), map[string]interface{}{"error": "unavailable"})
		writeJSON(w, http.StatusOK, result)
	})

	mux.HandleFunc("/api/v1/office/studio/testing", func(w http.ResponseWriter, r *http.Request) {
		result := callOffice("office_studio_testing_snapshot", emptyPayload(), map[string]interface{}{"error": "unavailable"})
		writeJSON(w, http.StatusOK, result)
	})

	mux.HandleFunc("/api/v1/office/studio/notes", func(w http.ResponseWriter, r *http.Request) {
		result := callOffice("office_studio_notes_snapshot", emptyPayload(), map[string]interface{}{"error": "unavailable"})
		writeJSON(w, http.StatusOK, result)
	})

	mux.HandleFunc("/api/v1/office/studio/docs", func(w http.ResponseWriter, r *http.Request) {
		result := callOffice("office_studio_docs_snapshot", emptyPayload(), map[string]interface{}{"error": "unavailable"})
		writeJSON(w, http.StatusOK, result)
	})

	mux.HandleFunc("/api/v1/office/studio/content", func(w http.ResponseWriter, r *http.Request) {
		result := callOffice("office_studio_content_snapshot", emptyPayload(), map[string]interface{}{"error": "unavailable"})
		writeJSON(w, http.StatusOK, result)
	})

	mux.HandleFunc("/api/v1/office/work/backlog", func(w http.ResponseWriter, r *http.Request) {
		result := callOffice("office_work_backlog_snapshot", emptyPayload(), map[string]interface{}{"error": "unavailable"})
		writeJSON(w, http.StatusOK, result)
	})

	mux.HandleFunc("/api/v1/office/work/boards", func(w http.ResponseWriter, r *http.Request) {
		result := callOffice("office_work_boards_snapshot", emptyPayload(), map[string]interface{}{"error": "unavailable"})
		writeJSON(w, http.StatusOK, result)
	})

	mux.HandleFunc("/api/v1/office/work/timeline", func(w http.ResponseWriter, r *http.Request) {
		result := callOffice("office_work_timeline_snapshot", emptyPayload(), map[string]interface{}{"error": "unavailable"})
		writeJSON(w, http.StatusOK, result)
	})

	mux.HandleFunc("/api/v1/office/work/analytics", func(w http.ResponseWriter, r *http.Request) {
		result := callOffice("office_work_analytics_snapshot", emptyPayload(), map[string]interface{}{"error": "unavailable"})
		writeJSON(w, http.StatusOK, result)
	})

	mux.HandleFunc("/api/v1/office/work/resources", func(w http.ResponseWriter, r *http.Request) {
		result := callOffice("office_work_resources_snapshot", emptyPayload(), map[string]interface{}{"error": "unavailable"})
		writeJSON(w, http.StatusOK, result)
	})

	mux.HandleFunc("/api/v1/office/work/content", func(w http.ResponseWriter, r *http.Request) {
		result := callOffice("office_work_content_snapshot", emptyPayload(), map[string]interface{}{"error": "unavailable"})
		writeJSON(w, http.StatusOK, result)
	})

	mux.HandleFunc("/api/v1/office/work/governance", func(w http.ResponseWriter, r *http.Request) {
		result := callOffice("office_work_governance_snapshot", emptyPayload(), map[string]interface{}{"error": "unavailable"})
		writeJSON(w, http.StatusOK, result)
	})

	port := serviceutil.ResolveAddr(officeDefaultPort, "KOGI_OFFICE_PORT")
	addr := serviceutil.ToListenAddr(port)
	log.Printf("[%s] listening on %s", officeServiceName, addr)
	if err := http.ListenAndServe(addr, mux); err != nil {
		log.Printf("[%s] server error: %v", officeServiceName, err)
		os.Exit(1)
	}
}

