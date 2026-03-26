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
	spacesServiceName = "kogi-spaces-service"
	spacesDefaultPort = "9014"
)

var spacesDLLConfig = serviceutil.RustBridgeConfig{
	Mode:          serviceutil.RustModeDLL,
	EnvBinKey:     "KOGI_SPACES_DLL",
	WellKnownName: "kogi_spaces",
	RepoSubPaths: []string{
		filepath.Join("kogi-spaces", "target", "release"),
		filepath.Join("kogi-spaces", "target", "debug"),
		filepath.Join("kogi-spaces"),
	},
	ServiceLabel: "kogi-spaces",
	Timeout:      5 * time.Second,
}

var spacesExeConfig = serviceutil.RustBridgeConfig{
	Mode:          serviceutil.RustModeExe,
	EnvBinKey:     "KOGI_SPACES_SYSTEM_BIN",
	WellKnownName: "kogi-spaces-system",
	RepoSubPaths: []string{
		filepath.Join("kogi-spaces", "target", "debug"),
		filepath.Join("kogi-spaces", "target", "release"),
		filepath.Join("kogi-spaces"),
	},
	ServiceLabel: "kogi-spaces-exe",
	Timeout:      5 * time.Second,
}

func callSpaces(funcName string, payload interface{}, fallback interface{}) interface{} {
	res, err := serviceutil.CallRust(spacesDLLConfig, funcName, payload)
	if err != nil {
		res, err = serviceutil.CallRust(spacesExeConfig, funcName, payload)
		if err != nil {
			log.Printf("[%s] rust call %s err=%v", spacesServiceName, funcName, err)
			return fallback
		}
	}
	return res
}

func callSpacesDirect(funcName string, payload interface{}) (interface{}, error) {
	res, err := serviceutil.CallRust(spacesDLLConfig, funcName, payload)
	if err != nil {
		return serviceutil.CallRust(spacesExeConfig, funcName, payload)
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
		result := callSpaces("spaces_health", emptyPayload(), map[string]interface{}{
			"ok":      false,
			"service": spacesServiceName,
		})
		writeJSON(w, http.StatusOK, result)
	})

	mux.HandleFunc("/api/v1/spaces/init", func(w http.ResponseWriter, r *http.Request) {
		if r.Method != http.MethodPost {
			writeJSON(w, http.StatusMethodNotAllowed, map[string]string{"error": "POST required"})
			return
		}
		var payload map[string]interface{}
		if err := readJSON(r, &payload); err != nil {
			writeJSON(w, http.StatusBadRequest, map[string]string{"error": err.Error()})
			return
		}
		result := callSpaces("spaces_init", payload, map[string]interface{}{"ok": false})
		writeJSON(w, http.StatusOK, result)
	})

	mux.HandleFunc("/api/v1/spaces/state", func(w http.ResponseWriter, r *http.Request) {
		result := callSpaces("spaces_state", emptyPayload(), map[string]interface{}{})
		writeJSON(w, http.StatusOK, result)
	})

	mux.HandleFunc("/api/v1/spaces/refresh", func(w http.ResponseWriter, r *http.Request) {
		if r.Method != http.MethodPost {
			writeJSON(w, http.StatusMethodNotAllowed, map[string]string{"error": "POST required"})
			return
		}
		result := callSpaces("spaces_refresh", emptyPayload(), map[string]interface{}{})
		writeJSON(w, http.StatusOK, result)
	})

	mux.HandleFunc("/api/v1/spaces", func(w http.ResponseWriter, r *http.Request) {
		if r.URL.Path != "/api/v1/spaces" {
			http.NotFound(w, r)
			return
		}
		result := callSpaces("spaces_spaces", emptyPayload(), map[string]interface{}{"spaces": []interface{}{}})
		writeJSON(w, http.StatusOK, result)
	})

	mux.HandleFunc("/api/v1/spaces/owner/", func(w http.ResponseWriter, r *http.Request) {
		ownerID := serviceutil.TrimPrefix(r, "/api/v1/spaces/owner/")
		if ownerID == "" {
			writeJSON(w, http.StatusBadRequest, map[string]string{"error": "owner_id required"})
			return
		}
		result := callSpaces("spaces_spaces_by_owner",
			map[string]string{"owner_id": ownerID},
			map[string]interface{}{"spaces": []interface{}{}})
		writeJSON(w, http.StatusOK, result)
	})

	mux.HandleFunc("/api/v1/spaces/publish", func(w http.ResponseWriter, r *http.Request) {
		if r.Method != http.MethodPost {
			writeJSON(w, http.StatusMethodNotAllowed, map[string]string{"error": "POST required"})
			return
		}
		var payload map[string]interface{}
		if err := readJSON(r, &payload); err != nil {
			writeJSON(w, http.StatusBadRequest, map[string]string{"error": err.Error()})
			return
		}
		result, err := callSpacesDirect("spaces_publish_space", payload)
		if err != nil {
			writeJSON(w, http.StatusInternalServerError, map[string]string{"error": err.Error()})
			return
		}
		writeJSON(w, http.StatusCreated, result)
	})

	mux.HandleFunc("/api/v1/spaces/rooms", func(w http.ResponseWriter, r *http.Request) {
		result := callSpaces("spaces_rooms", emptyPayload(), map[string]interface{}{"rooms": []interface{}{}})
		writeJSON(w, http.StatusOK, result)
	})

	mux.HandleFunc("/api/v1/spaces/channels", func(w http.ResponseWriter, r *http.Request) {
		result := callSpaces("spaces_channels", emptyPayload(), map[string]interface{}{"channels": []interface{}{}})
		writeJSON(w, http.StatusOK, result)
	})

	mux.HandleFunc("/api/v1/spaces/events", func(w http.ResponseWriter, r *http.Request) {
		result := callSpaces("spaces_events", emptyPayload(), map[string]interface{}{"events": []interface{}{}})
		writeJSON(w, http.StatusOK, result)
	})

	mux.HandleFunc("/api/v1/spaces/workspaces", func(w http.ResponseWriter, r *http.Request) {
		result := callSpaces("spaces_workspaces", emptyPayload(), map[string]interface{}{"workspaces": []interface{}{}})
		writeJSON(w, http.StatusOK, result)
	})

	mux.HandleFunc("/api/v1/spaces/", func(w http.ResponseWriter, r *http.Request) {
		trimmed := serviceutil.TrimPrefix(r, "/api/v1/spaces/")
		if trimmed == "" || trimmed == " " {
			http.NotFound(w, r)
			return
		}
		parts := splitPath(trimmed)
		if len(parts) == 1 {
			spaceID := parts[0]
			result := callSpaces("spaces_space",
				map[string]string{"space_id": spaceID},
				map[string]interface{}{"error": "not_found"})
			writeJSON(w, http.StatusOK, result)
			return
		}
		if len(parts) == 2 && parts[1] == "members" {
			spaceID := parts[0]
			result := callSpaces("spaces_members",
				map[string]string{"space_id": spaceID},
				map[string]interface{}{"members": []interface{}{}})
			writeJSON(w, http.StatusOK, result)
			return
		}
		http.NotFound(w, r)
	})

	mux.HandleFunc("/api/v1/spaces/dashboard", func(w http.ResponseWriter, r *http.Request) {
		result := callSpaces("spaces_dashboard_snapshot", emptyPayload(), map[string]interface{}{"error": "unavailable"})
		writeJSON(w, http.StatusOK, result)
	})

	mux.HandleFunc("/api/v1/spaces/feed", func(w http.ResponseWriter, r *http.Request) {
		result := callSpaces("spaces_feed_snapshot", emptyPayload(), map[string]interface{}{"error": "unavailable"})
		writeJSON(w, http.StatusOK, result)
	})

	mux.HandleFunc("/api/v1/spaces/timeline", func(w http.ResponseWriter, r *http.Request) {
		result := callSpaces("spaces_timeline_snapshot", emptyPayload(), map[string]interface{}{"error": "unavailable"})
		writeJSON(w, http.StatusOK, result)
	})

	mux.HandleFunc("/api/v1/spaces/rooms/overview", func(w http.ResponseWriter, r *http.Request) {
		result := callSpaces("spaces_rooms_snapshot", emptyPayload(), map[string]interface{}{"error": "unavailable"})
		writeJSON(w, http.StatusOK, result)
	})

	mux.HandleFunc("/api/v1/spaces/channels/overview", func(w http.ResponseWriter, r *http.Request) {
		result := callSpaces("spaces_channels_snapshot", emptyPayload(), map[string]interface{}{"error": "unavailable"})
		writeJSON(w, http.StatusOK, result)
	})

	mux.HandleFunc("/api/v1/spaces/events/overview", func(w http.ResponseWriter, r *http.Request) {
		result := callSpaces("spaces_events_snapshot", emptyPayload(), map[string]interface{}{"error": "unavailable"})
		writeJSON(w, http.StatusOK, result)
	})

	mux.HandleFunc("/api/v1/spaces/network/overview", func(w http.ResponseWriter, r *http.Request) {
		result := callSpaces("spaces_network_overview_snapshot", emptyPayload(), map[string]interface{}{"error": "unavailable"})
		writeJSON(w, http.StatusOK, result)
	})

	mux.HandleFunc("/api/v1/spaces/network/linknet", func(w http.ResponseWriter, r *http.Request) {
		result := callSpaces("spaces_network_linknet_snapshot", emptyPayload(), map[string]interface{}{"error": "unavailable"})
		writeJSON(w, http.StatusOK, result)
	})

	mux.HandleFunc("/api/v1/spaces/network/linktree", func(w http.ResponseWriter, r *http.Request) {
		result := callSpaces("spaces_network_linktree_snapshot", emptyPayload(), map[string]interface{}{"error": "unavailable"})
		writeJSON(w, http.StatusOK, result)
	})

	mux.HandleFunc("/api/v1/spaces/network/linkforest", func(w http.ResponseWriter, r *http.Request) {
		result := callSpaces("spaces_network_linkforest_snapshot", emptyPayload(), map[string]interface{}{"error": "unavailable"})
		writeJSON(w, http.StatusOK, result)
	})

	port := serviceutil.ResolveAddr(spacesDefaultPort, "KOGI_SPACES_PORT")
	addr := serviceutil.ToListenAddr(port)
	log.Printf("[%s] listening on %s", spacesServiceName, addr)
	if err := http.ListenAndServe(addr, mux); err != nil {
		log.Printf("[%s] server error: %v", spacesServiceName, err)
		os.Exit(1)
	}
}

func splitPath(path string) []string {
	var parts []string
	current := ""
	for _, ch := range path {
		if ch == '/' {
			if current != "" {
				parts = append(parts, current)
				current = ""
			}
			continue
		}
		current += string(ch)
	}
	if current != "" {
		parts = append(parts, current)
	}
	return parts
}
