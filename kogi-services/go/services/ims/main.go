package main

import (
	"encoding/json"
	"log"
	"net/http"
	"os"
	"strings"

	"kogi.services/lib/ops"
)

type workerIdentity struct {
	ID          string   `json:"id"`
	Name        string   `json:"name"`
	Personas    []string `json:"personas"`
	Roles       []string `json:"roles"`
	WorkerTypes []string `json:"worker_types"`
	Status      string   `json:"status"`
}

type workerProfile struct {
	ID           string                 `json:"id"`
	IdentityID   string                 `json:"identity_id"`
	ProfileType  string                 `json:"profile_type"`
	Name         string                 `json:"name"`
	Accounts     []string               `json:"accounts"`
	Portfolios   []string               `json:"portfolios"`
	Integrations []string               `json:"integrations"`
	Tools        []string               `json:"tools"`
	Projects     []string               `json:"projects"`
	Programs     []string               `json:"programs"`
	Settings     map[string]interface{} `json:"settings"`
	Options      map[string]interface{} `json:"options"`
	Parameters   map[string]interface{} `json:"parameters"`
}

func main() {
	rt := ops.Init("ims-service")
	rt.State("init")

	identities := []workerIdentity{
		{
			ID:          "ident-001",
			Name:        "Dominic Worker",
			Personas:    []string{"investor", "developer", "donor"},
			Roles:       []string{"owner", "admin", "contributor"},
			WorkerTypes: []string{"entrepreneur", "consultant", "full_time_worker"},
			Status:      "active",
		},
	}

	profiles := []workerProfile{
		{
			ID:          "profile-personal-001",
			IdentityID:  "ident-001",
			ProfileType: "personal",
			Name:        "Personal Profile",
			Accounts:    []string{"gmail", "personal-wallet"},
			Portfolios:  []string{"life-portfolio"},
			Integrations: []string{
				"google-drive",
				"instagram",
			},
			Tools:      []string{"calendar", "notes"},
			Projects:   []string{"fitness-tracker"},
			Programs:   []string{"personal-growth"},
			Settings:   map[string]interface{}{"theme": "light", "timezone": "America/Chicago"},
			Options:    map[string]interface{}{"notifications": true},
			Parameters: map[string]interface{}{"focus_hours": 2},
		},
		{
			ID:          "profile-work-001",
			IdentityID:  "ident-001",
			ProfileType: "work",
			Name:        "Work Profile",
			Accounts:    []string{"github", "upwork", "stripe"},
			Portfolios:  []string{"consulting-portfolio", "kogi-platform"},
			Integrations: []string{
				"jira",
				"monday",
				"openai",
			},
			Tools:      []string{"kanban", "sprint-planner", "dev-console"},
			Projects:   []string{"kogi-mvp", "client-alpha"},
			Programs:   []string{"consulting-practice"},
			Settings:   map[string]interface{}{"default_workspace": "work"},
			Options:    map[string]interface{}{"auto_time_tracking": true},
			Parameters: map[string]interface{}{"billing_rate": 115},
		},
		{
			ID:          "profile-community-001",
			IdentityID:  "ident-001",
			ProfileType: "community",
			Name:        "Community Profile",
			Accounts:    []string{"discord", "youtube"},
			Portfolios:  []string{"open-source-portfolio"},
			Integrations: []string{
				"slack",
				"zoom",
			},
			Tools:      []string{"community-room", "events"},
			Projects:   []string{"co-op-launch"},
			Programs:   []string{"mutual-aid-network"},
			Settings:   map[string]interface{}{"public_visibility": true},
			Options:    map[string]interface{}{"allow_dm": true},
			Parameters: map[string]interface{}{"weekly_events": 3},
		},
	}

	mux := http.NewServeMux()
	rt.State("configure")
	rt.WatchSignals()

	mux.HandleFunc("/health", func(w http.ResponseWriter, r *http.Request) {
		writeJSON(w, http.StatusOK, map[string]string{"status": "ok", "service": "ims-service"})
	})

	mux.HandleFunc("/api/v1/ims/runtime", func(w http.ResponseWriter, r *http.Request) {
		writeJSON(w, http.StatusOK, map[string]interface{}{
			"service":         "ims-service",
			"component_id":    "kogi.services.ims",
			"gateway":         "http://127.0.0.1:8090",
			"network_manager": "kogi-go-network",
			"publishes": []string{
				"ims.identity.created",
				"ims.profile.updated",
			},
			"subscribes": []string{
				"auth.session.created",
				"office.dashboard.refresh",
			},
			"engine_stream": "engine.ingest",
		})
	})

	mux.HandleFunc("/api/v1/ims/identities", func(w http.ResponseWriter, r *http.Request) {
		writeJSON(w, http.StatusOK, map[string]interface{}{"identities": identities})
	})

	mux.HandleFunc("/api/v1/ims/profiles", func(w http.ResponseWriter, r *http.Request) {
		identityID := r.URL.Query().Get("identity_id")
		if identityID == "" {
			writeJSON(w, http.StatusOK, map[string]interface{}{"profiles": profiles})
			return
		}

		filtered := make([]workerProfile, 0)
		for _, p := range profiles {
			if p.IdentityID == identityID {
				filtered = append(filtered, p)
			}
		}
		writeJSON(w, http.StatusOK, map[string]interface{}{"profiles": filtered})
	})

	mux.HandleFunc("/api/v1/ims/autonomy", func(w http.ResponseWriter, r *http.Request) {
		writeJSON(w, http.StatusOK, map[string]interface{}{
			"abstractions": []string{
				"identity_management",
				"workspace_organization",
				"connection_registry",
				"contact_directory",
				"asset_vault",
			},
			"description": "Worker autonomy through system-level abstractions and profile-specific controls.",
		})
	})

	addr := resolveAddr("9005", "KOGI_IMS_PORT")
	rt.State("running")
	rt.Status("ok", "listening="+addr)
	log.Printf("ims-service listening on %s", addr)
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
