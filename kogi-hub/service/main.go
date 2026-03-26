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
    hubServiceName = "kogi-hub-service"
    hubDefaultPort = "9016"
)

var hubDLLConfig = serviceutil.RustBridgeConfig{
    Mode:          serviceutil.RustModeDLL,
    EnvBinKey:     "KOGI_HUB_DLL",
    WellKnownName: "kogi_hub",
    RepoSubPaths: []string{
        filepath.Join("kogi-hub", "target", "release"),
        filepath.Join("kogi-hub", "target", "debug"),
        filepath.Join("kogi-hub"),
    },
    ServiceLabel: "kogi-hub",
    Timeout:      5 * time.Second,
}

var hubExeConfig = serviceutil.RustBridgeConfig{
    Mode:          serviceutil.RustModeExe,
    EnvBinKey:     "KOGI_HUB_SYSTEM_BIN",
    WellKnownName: "kogi-hub-system",
    RepoSubPaths: []string{
        filepath.Join("kogi-hub", "target", "debug"),
        filepath.Join("kogi-hub", "target", "release"),
        filepath.Join("kogi-hub"),
    },
    ServiceLabel: "kogi-hub-exe",
    Timeout:      5 * time.Second,
}

func callHub(funcName string, payload interface{}, fallback interface{}) interface{} {
    res, err := serviceutil.CallRust(hubDLLConfig, funcName, payload)
    if err != nil {
        res, err = serviceutil.CallRust(hubExeConfig, funcName, payload)
        if err != nil {
            log.Printf("[%s] rust call %s err=%v", hubServiceName, funcName, err)
            return fallback
        }
    }
    return res
}

func callHubDirect(funcName string, payload interface{}) (interface{}, error) {
    res, err := serviceutil.CallRust(hubDLLConfig, funcName, payload)
    if err != nil {
        return serviceutil.CallRust(hubExeConfig, funcName, payload)
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
        result := callHub("hub_health", emptyPayload(), map[string]interface{}{
            "ok":      false,
            "service": hubServiceName,
        })
        writeJSON(w, http.StatusOK, result)
    })

    mux.HandleFunc("/api/v1/hub/init", func(w http.ResponseWriter, r *http.Request) {
        if r.Method != http.MethodPost {
            writeJSON(w, http.StatusMethodNotAllowed, map[string]string{"error": "POST required"})
            return
        }
        var payload map[string]interface{}
        if err := readJSON(r, &payload); err != nil {
            writeJSON(w, http.StatusBadRequest, map[string]string{"error": err.Error()})
            return
        }
        result := callHub("hub_init", payload, map[string]interface{}{"ok": false})
        writeJSON(w, http.StatusOK, result)
    })

    mux.HandleFunc("/api/v1/hub/state", func(w http.ResponseWriter, r *http.Request) {
        result := callHub("hub_state", emptyPayload(), map[string]interface{}{})
        writeJSON(w, http.StatusOK, result)
    })

    mux.HandleFunc("/api/v1/hub/refresh", func(w http.ResponseWriter, r *http.Request) {
        if r.Method != http.MethodPost {
            writeJSON(w, http.StatusMethodNotAllowed, map[string]string{"error": "POST required"})
            return
        }
        result := callHub("hub_refresh", emptyPayload(), map[string]interface{}{})
        writeJSON(w, http.StatusOK, result)
    })

    mux.HandleFunc("/api/v1/hub/entities", func(w http.ResponseWriter, r *http.Request) {
        result := callHub("hub_entities", emptyPayload(), map[string]interface{}{"entities": []interface{}{}})
        writeJSON(w, http.StatusOK, result)
    })

    mux.HandleFunc("/api/v1/hub/members", func(w http.ResponseWriter, r *http.Request) {
        result := callHub("hub_members", emptyPayload(), map[string]interface{}{"members": []interface{}{}})
        writeJSON(w, http.StatusOK, result)
    })

    mux.HandleFunc("/api/v1/hub/governance/data", func(w http.ResponseWriter, r *http.Request) {
        result := callHub("hub_governance", emptyPayload(), map[string]interface{}{})
        writeJSON(w, http.StatusOK, result)
    })

    mux.HandleFunc("/api/v1/hub/voting/data", func(w http.ResponseWriter, r *http.Request) {
        result := callHub("hub_voting", emptyPayload(), map[string]interface{}{})
        writeJSON(w, http.StatusOK, result)
    })

    mux.HandleFunc("/api/v1/hub/contracts", func(w http.ResponseWriter, r *http.Request) {
        result := callHub("hub_contracts", emptyPayload(), map[string]interface{}{"contracts": []interface{}{}})
        writeJSON(w, http.StatusOK, result)
    })

    mux.HandleFunc("/api/v1/hub/rights", func(w http.ResponseWriter, r *http.Request) {
        result := callHub("hub_rights", emptyPayload(), map[string]interface{}{"rights": []interface{}{}})
        writeJSON(w, http.StatusOK, result)
    })

    mux.HandleFunc("/api/v1/hub/ip", func(w http.ResponseWriter, r *http.Request) {
        result := callHub("hub_ip_assets", emptyPayload(), map[string]interface{}{"assets": []interface{}{}})
        writeJSON(w, http.StatusOK, result)
    })

    mux.HandleFunc("/api/v1/hub/allocations", func(w http.ResponseWriter, r *http.Request) {
        result := callHub("hub_allocations", emptyPayload(), map[string]interface{}{"allocations": []interface{}{}})
        writeJSON(w, http.StatusOK, result)
    })

    mux.HandleFunc("/api/v1/hub/distributions", func(w http.ResponseWriter, r *http.Request) {
        result := callHub("hub_distributions", emptyPayload(), map[string]interface{}{"distributions": []interface{}{}})
        writeJSON(w, http.StatusOK, result)
    })

    mux.HandleFunc("/api/v1/hub/restitutions", func(w http.ResponseWriter, r *http.Request) {
        result := callHub("hub_restitutions", emptyPayload(), map[string]interface{}{"restitutions": []interface{}{}})
        writeJSON(w, http.StatusOK, result)
    })

    mux.HandleFunc("/api/v1/hub/negotiations/data", func(w http.ResponseWriter, r *http.Request) {
        result := callHub("hub_negotiations", emptyPayload(), map[string]interface{}{"negotiations": []interface{}{}})
        writeJSON(w, http.StatusOK, result)
    })

    mux.HandleFunc("/api/v1/hub/dashboard", func(w http.ResponseWriter, r *http.Request) {
        result := callHub("hub_dashboard_snapshot", emptyPayload(), map[string]interface{}{"error": "unavailable"})
        writeJSON(w, http.StatusOK, result)
    })

    mux.HandleFunc("/api/v1/hub/governance", func(w http.ResponseWriter, r *http.Request) {
        result := callHub("hub_governance_snapshot", emptyPayload(), map[string]interface{}{"error": "unavailable"})
        writeJSON(w, http.StatusOK, result)
    })

    mux.HandleFunc("/api/v1/hub/voting", func(w http.ResponseWriter, r *http.Request) {
        result := callHub("hub_voting_snapshot", emptyPayload(), map[string]interface{}{"error": "unavailable"})
        writeJSON(w, http.StatusOK, result)
    })

    mux.HandleFunc("/api/v1/hub/allocation", func(w http.ResponseWriter, r *http.Request) {
        result := callHub("hub_allocation_snapshot", emptyPayload(), map[string]interface{}{"error": "unavailable"})
        writeJSON(w, http.StatusOK, result)
    })

    mux.HandleFunc("/api/v1/hub/distribution", func(w http.ResponseWriter, r *http.Request) {
        result := callHub("hub_distribution_snapshot", emptyPayload(), map[string]interface{}{"error": "unavailable"})
        writeJSON(w, http.StatusOK, result)
    })

    mux.HandleFunc("/api/v1/hub/collaboration", func(w http.ResponseWriter, r *http.Request) {
        result := callHub("hub_collaboration_snapshot", emptyPayload(), map[string]interface{}{"error": "unavailable"})
        writeJSON(w, http.StatusOK, result)
    })

    mux.HandleFunc("/api/v1/hub/restitution", func(w http.ResponseWriter, r *http.Request) {
        result := callHub("hub_restitution_snapshot", emptyPayload(), map[string]interface{}{"error": "unavailable"})
        writeJSON(w, http.StatusOK, result)
    })

    mux.HandleFunc("/api/v1/hub/negotiations", func(w http.ResponseWriter, r *http.Request) {
        result := callHub("hub_negotiations_snapshot", emptyPayload(), map[string]interface{}{"error": "unavailable"})
        writeJSON(w, http.StatusOK, result)
    })

    mux.HandleFunc("/api/v1/hub/teams", func(w http.ResponseWriter, r *http.Request) {
        result := callHub("hub_teams_snapshot", emptyPayload(), map[string]interface{}{"error": "unavailable"})
        writeJSON(w, http.StatusOK, result)
    })

    mux.HandleFunc("/api/v1/hub/organizations", func(w http.ResponseWriter, r *http.Request) {
        result := callHub("hub_organizations_snapshot", emptyPayload(), map[string]interface{}{"error": "unavailable"})
        writeJSON(w, http.StatusOK, result)
    })

    mux.HandleFunc("/api/v1/hub/collectives", func(w http.ResponseWriter, r *http.Request) {
        result := callHub("hub_collectives_snapshot", emptyPayload(), map[string]interface{}{"error": "unavailable"})
        writeJSON(w, http.StatusOK, result)
    })

    mux.HandleFunc("/api/v1/hub/cooperatives", func(w http.ResponseWriter, r *http.Request) {
        result := callHub("hub_cooperatives_snapshot", emptyPayload(), map[string]interface{}{"error": "unavailable"})
        writeJSON(w, http.StatusOK, result)
    })

    mux.HandleFunc("/api/v1/hub/federations", func(w http.ResponseWriter, r *http.Request) {
        result := callHub("hub_federations_snapshot", emptyPayload(), map[string]interface{}{"error": "unavailable"})
        writeJSON(w, http.StatusOK, result)
    })

    mux.HandleFunc("/api/v1/hub/autonomous", func(w http.ResponseWriter, r *http.Request) {
        result := callHub("hub_autonomous_snapshot", emptyPayload(), map[string]interface{}{"error": "unavailable"})
        writeJSON(w, http.StatusOK, result)
    })

    mux.HandleFunc("/api/v1/hub/open-source", func(w http.ResponseWriter, r *http.Request) {
        result := callHub("hub_open_source_snapshot", emptyPayload(), map[string]interface{}{"error": "unavailable"})
        writeJSON(w, http.StatusOK, result)
    })

    mux.HandleFunc("/api/v1/hub/group-economics", func(w http.ResponseWriter, r *http.Request) {
        result := callHub("hub_group_economics_snapshot", emptyPayload(), map[string]interface{}{"error": "unavailable"})
        writeJSON(w, http.StatusOK, result)
    })

    mux.HandleFunc("/api/v1/hub/resource-crowdfund", func(w http.ResponseWriter, r *http.Request) {
        result := callHub("hub_resource_crowdfund_snapshot", emptyPayload(), map[string]interface{}{"error": "unavailable"})
        writeJSON(w, http.StatusOK, result)
    })

    mux.HandleFunc("/api/v1/hub/community-showcase", func(w http.ResponseWriter, r *http.Request) {
        result := callHub("hub_community_showcase_snapshot", emptyPayload(), map[string]interface{}{"error": "unavailable"})
        writeJSON(w, http.StatusOK, result)
    })

    mux.HandleFunc("/api/v1/hub/contracts/overview", func(w http.ResponseWriter, r *http.Request) {
        result := callHub("hub_contracts_snapshot", emptyPayload(), map[string]interface{}{"error": "unavailable"})
        writeJSON(w, http.StatusOK, result)
    })

    mux.HandleFunc("/api/v1/hub/ip/overview", func(w http.ResponseWriter, r *http.Request) {
        result := callHub("hub_ip_snapshot", emptyPayload(), map[string]interface{}{"error": "unavailable"})
        writeJSON(w, http.StatusOK, result)
    })

    port := serviceutil.ResolveAddr(hubDefaultPort, "KOGI_HUB_PORT")
    addr := serviceutil.ToListenAddr(port)
    log.Printf("[%s] listening on %s", hubServiceName, addr)
    if err := http.ListenAndServe(addr, mux); err != nil {
        log.Printf("[%s] server error: %v", hubServiceName, err)
        os.Exit(1)
    }
}
