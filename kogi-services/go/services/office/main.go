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
        writeJSON(w, http.StatusOK, map[string]string{"status": "ok", "service": "office-service"})
    })

    mux.HandleFunc("/api/v1/office", func(w http.ResponseWriter, r *http.Request) {
        writeJSON(w, http.StatusOK, map[string]interface{}{
            "module":      "kogi.office",
            "application": "Kogi Office",
            "service":     "kogi-services/go/services/office",
            "views": []map[string]string{
                {"id": "dashboard", "title": "Office Dashboard", "status": "active"},
                {"id": "portfolio", "title": "Office Portfolio", "status": "active"},
                {"id": "timeline", "title": "Office Timeline", "status": "active"},
                {"id": "workspace", "title": "Office Workspace", "status": "active"},
                {"id": "assistant", "title": "Office Assistant", "status": "active"},
            },
            "integrations": []string{"jira", "monday", "base44", "claude", "chatgpt", "grok", "openai", "gitlab", "github"},
        })
    })

    mux.HandleFunc("/api/v1/office/dashboard", func(w http.ResponseWriter, r *http.Request) {
        writeJSON(w, http.StatusOK, map[string]interface{}{
            "view": "dashboard",
            "active_projects": []map[string]interface{}{
                {"id": "proj-kogi-mvp", "name": "Kogi MVP Prototype", "status": "active", "progress_percent": 68},
                {"id": "proj-api-fabric", "name": "API Fabric Reconcile", "status": "active", "progress_percent": 44},
            },
            "active_programs": []map[string]interface{}{
                {"id": "prog-platform-launch", "name": "Platform Launch 2026", "status": "active"},
                {"id": "prog-coop-network", "name": "Cooperative Network Pilot", "status": "active"},
            },
            "portfolio_attention": []map[string]string{
                {"item_id": "asset-risk-model", "item_type": "asset", "title": "Risk model needs review", "priority": "high"},
                {"item_id": "doc-compliance-pack", "item_type": "document", "title": "Compliance packet expires in 4 days", "priority": "critical"},
            },
            "notifications": []map[string]string{
                {"id": "notif-001", "category": "task", "message": "3 blocked stories need triage"},
                {"id": "notif-002", "category": "integration", "message": "Jira sync delayed for 2 projects"},
            },
            "direct_messages": []map[string]string{
                {"id": "dm-001", "from": "Ari Program Lead", "subject": "Roadmap checkpoint"},
                {"id": "dm-002", "from": "Mina Investor", "subject": "Due diligence docs"},
            },
            "event_feed": []map[string]string{
                {"id": "feed-001", "source": "events", "message": "Weekly governance sync starts in 2h"},
                {"id": "feed-002", "source": "community", "message": "New cooperative member onboarding request"},
                {"id": "feed-003", "source": "marketplace", "message": "Resource listing matched your workspace filter"},
                {"id": "feed-004", "source": "exchange", "message": "Deal escrow verification completed"},
            },
            "personas_roles": []map[string]interface{}{
                {"persona": "investor", "roles": []string{"owner", "approver"}},
                {"persona": "developer", "roles": []string{"builder", "maintainer"}},
                {"persona": "donor", "roles": []string{"sponsor"}},
            },
            "quick_links": []map[string]string{
                {"id": "ql-portfolio", "label": "Open Portfolio", "target": "/office/portfolio"},
                {"id": "ql-workspace", "label": "Open Workspace", "target": "/office/workspace"},
                {"id": "ql-timeline", "label": "Open Timeline", "target": "/office/timeline"},
                {"id": "ql-assistant", "label": "Open Assistant", "target": "/office/assistant"},
            },
        })
    })

    mux.HandleFunc("/api/v1/office/portfolio", func(w http.ResponseWriter, r *http.Request) {
        writeJSON(w, http.StatusOK, map[string]interface{}{
            "view":       "portfolio",
            "views":      []string{"tiled", "tree", "modular_grid"},
            "item_types": []string{"project", "program", "resource", "asset", "capital", "investment", "solution", "document", "misc", "custom"},
            "items": []map[string]interface{}{
                {"id": "port-proj-001", "type": "project", "name": "Kogi Kernel Runtime", "status": "active"},
                {"id": "port-prog-001", "type": "program", "name": "Independent Worker Launch", "status": "active"},
                {"id": "port-asset-001", "type": "asset", "name": "Design System Library", "status": "active"},
                {"id": "port-doc-001", "type": "document", "name": "Platform Compliance Book", "status": "review"},
            },
            "focus_item": map[string]interface{}{
                "id":          "port-proj-001",
                "type":        "project",
                "name":        "Kogi Kernel Runtime",
                "description": "Kernel orchestration and module isolation controls.",
                "containers": []map[string]string{
                    {"type": "item_binder", "id": "binder-kernel"},
                    {"type": "item_book", "id": "book-kernel-ops"},
                    {"type": "item_notebook", "id": "notebook-kernel-debug"},
                    {"type": "item_playbook", "id": "playbook-kernel-release"},
                    {"type": "item_folders", "id": "folder-kernel"},
                    {"type": "item_files", "id": "files-kernel"},
                    {"type": "item_version_control", "id": "git-kernel-repo"},
                    {"type": "item_metadata", "id": "meta-kernel-runtime"},
                },
            },
        })
    })

    mux.HandleFunc("/api/v1/office/timeline", func(w http.ResponseWriter, r *http.Request) {
        writeJSON(w, http.StatusOK, map[string]interface{}{
            "view": "timeline",
            "calendars": []map[string]interface{}{
                {"id": "cal-personal", "name": "Personal Calendar", "events": 14},
                {"id": "cal-work", "name": "Work Calendar", "events": 26},
                {"id": "cal-community", "name": "Community Calendar", "events": 9},
            },
            "schedules": []map[string]string{
                {"id": "sched-ops", "name": "Operations Schedule", "cadence": "weekly"},
                {"id": "sched-delivery", "name": "Delivery Schedule", "cadence": "daily"},
            },
            "roadmaps": []map[string]interface{}{
                {"id": "rm-platform", "name": "Platform Roadmap", "milestones": 12},
                {"id": "rm-office", "name": "Office Module Roadmap", "milestones": 7},
            },
            "gantts": []map[string]interface{}{
                {"id": "gantt-launch", "name": "Launch Gantt", "active_tasks": 23},
                {"id": "gantt-integrations", "name": "Integrations Gantt", "active_tasks": 11},
            },
            "personal_timelines": []map[string]interface{}{
                {"id": "pt-growth", "name": "Personal Growth Timeline", "tracks": []string{"skills", "health", "network"}},
            },
        })
    })

    mux.HandleFunc("/api/v1/office/workspace", func(w http.ResponseWriter, r *http.Request) {
        writeJSON(w, http.StatusOK, map[string]interface{}{
            "view":    "workspace",
            "domains": []string{"personal_work", "operations", "tactics", "strategy", "governance"},
            "user_stories": []map[string]interface{}{
                {"id": "story-101", "title": "As a worker, I track project blockers", "status": "in_progress"},
                {"id": "story-102", "title": "As an operator, I align tactics to strategy", "status": "ready"},
            },
            "work_packages": []map[string]string{
                {"id": "wp-001", "name": "Kernel isolation instrumentation", "status": "active"},
                {"id": "wp-002", "name": "Office dashboard feed aggregation", "status": "active"},
            },
            "content_management": map[string]interface{}{
                "binders":   []string{"Delivery Binder", "Governance Binder"},
                "books":     []string{"Operating Manual", "Integration Book"},
                "notebooks": []string{"Daily Ops Notebook", "Experiment Notes"},
                "folders":   []string{"/office/projects", "/office/programs", "/office/resources"},
            },
            "tools": map[string]interface{}{
                "toolchains": []string{"build-chain", "release-chain", "observability-chain"},
                "toolkits":   []string{"analysis-toolkit", "delivery-toolkit"},
                "toolsets":   []string{"planning", "governance", "execution"},
                "links": []map[string]string{
                    {"label": "Jira", "target": "https://jira.example.local"},
                    {"label": "Monday", "target": "https://monday.example.local"},
                    {"label": "GitHub", "target": "https://github.example.local"},
                },
            },
        })
    })

    mux.HandleFunc("/api/v1/office/assistant", func(w http.ResponseWriter, r *http.Request) {
        writeJSON(w, http.StatusOK, map[string]interface{}{
            "view":         "assistant",
            "assistant_id": "office-assistant-001",
            "chat_context_window": map[string]interface{}{
                "active_profile":      "work",
                "context_tokens_used": 1640,
                "context_sources":     []string{"workspace", "portfolio", "timeline", "messages"},
            },
            "discover": []string{
                "New cooperative projects in your domain",
                "Recommended workflow templates for governance",
            },
            "recommendations": []map[string]string{
                {"id": "rec-001", "title": "Resolve 2 high-priority attention items", "reason": "critical deadline approaching"},
                {"id": "rec-002", "title": "Sync roadmap milestones with timeline calendar", "reason": "drift detected in gantt"},
            },
            "subscriptions": []map[string]string{
                {"id": "sub-001", "topic": "marketplace.resource_matches", "status": "active"},
                {"id": "sub-002", "topic": "exchange.deal_updates", "status": "active"},
                {"id": "sub-003", "topic": "community.room_updates", "status": "active"},
            },
            "explore": []string{
                "strategy templates",
                "portfolio governance packs",
                "workspace automation recipes",
            },
            "for_you": []map[string]string{
                {"id": "fy-001", "title": "Top 3 stories to complete this week"},
                {"id": "fy-002", "title": "Suggested integration: GitHub issue sync"},
                {"id": "fy-003", "title": "Budget variance alert in operations wallet"},
            },
        })
    })

    addr := resolveAddr("9006", "KOGI_OFFICE_PORT")
    log.Printf("office-service listening on %s", addr)
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
