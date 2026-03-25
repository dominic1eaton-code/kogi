package main

import (
    "fmt"
    "log"
    "net/http"
    "strings"
    "time"

    "kogi.network/lib/ops"
)

const (
    marketplaceServiceID   = "kogi.network.marketplace"
    marketplaceServiceName = "marketplace-service"
    marketplaceDefaultPort = "9008"
    marketplaceGateway     = "http://127.0.0.1:8090"
)

var marketplaceRuntime *ops.Runtime

func marketplacePublish(topic, payload string, meta map[string]string) {
    if marketplaceRuntime != nil {
        marketplaceRuntime.Publish(topic, marketplaceServiceID, "", payload)
    }
    publish(marketplaceGateway, marketplaceServiceID, topic, payload, meta)
}

func marketplaceHealthLoop() {
    healthLoop(marketplaceGateway, marketplaceServiceID, 30*time.Second, func() (bool, string) {
        bin, err := marketplaceResolveBinary()
        return err == nil, bin
    })
}

func marketplaceDeadLetterMonitor() {
    pollDeadLetters(marketplaceGateway,
        func(topic string) bool { return strings.HasPrefix(topic, "marketplace.") },
        func(id, topic string) { log.Printf("[marketplace-svc] dead-letter id=%s topic=%s", id, topic) },
    )
}

func marketplaceInboundPoll(topics []string) {
    pollInbound(marketplaceGateway, topics)
}

func marketplaceRegisterWithGateway(selfEndpoint string) {
    registerWithGateway(
        marketplaceGateway, marketplaceServiceID, selfEndpoint, "/health",
        []string{
            "marketplace.listing.published",
            "marketplace.listing.updated",
            "marketplace.listing.archived",
            "marketplace.deal.created",
            "marketplace.deal.updated",
        },
        []string{
            "portfolio.item.created",
            "portfolio.component.updated",
            "exchange.trade.executed",
            "office.dashboard.refresh",
        },
        []struct{ Prefix, Consumer string }{
            {"portfolio.", marketplaceServiceID},
            {"exchange.", marketplaceServiceID},
            {"office.", marketplaceServiceID},
        },
    )
}

func main() {
    marketplaceRuntime = ops.Init(marketplaceServiceName)
    marketplaceRuntime.State("init")
    mux := http.NewServeMux()
    marketplaceRuntime.State("configure")
    marketplaceRuntime.WatchSignals()

    addr := resolveAddr(marketplaceDefaultPort, "KOGI_MARKETPLACE_PORT")
    selfEndpoint := fmt.Sprintf("http://127.0.0.1%s", addr)
    go func() {
        time.Sleep(600 * time.Millisecond)
        marketplaceRegisterWithGateway(selfEndpoint)
        go marketplaceInboundPoll([]string{
            "portfolio.item.created",
            "portfolio.component.updated",
            "exchange.trade.executed",
            "office.dashboard.refresh",
        })
        go marketplaceHealthLoop()
        go marketplaceDeadLetterMonitor()
    }()

    mux.HandleFunc("/health", func(w http.ResponseWriter, r *http.Request) {
        writeJSON(w, http.StatusOK, map[string]string{"status": "ok", "service": marketplaceServiceName})
    })

    mux.HandleFunc("/api/v1/marketplace/runtime", func(w http.ResponseWriter, r *http.Request) {
        bin, _ := marketplaceResolveBinary()
        writeJSON(w, http.StatusOK, map[string]interface{}{
            "service":         marketplaceServiceName,
            "component_id":    marketplaceServiceID,
            "module_id":       "kogi.marketplace",
            "gateway":         marketplaceGateway,
            "network_manager": "kogi-go-network",
            "rust_bridge":     "kogi_marketplace.dll / kogi-marketplace-system",
            "rust_binary":     bin,
            "dll_config": map[string]string{
                "marketplace_dll": "KOGI_MARKETPLACE_DLL",
            },
            "publishes": []string{
                "marketplace.listing.published",
                "marketplace.listing.updated",
                "marketplace.listing.archived",
                "marketplace.deal.created",
                "marketplace.deal.updated",
            },
            "subscribes": []string{
                "portfolio.item.created",
                "portfolio.component.updated",
                "exchange.trade.executed",
                "office.dashboard.refresh",
            },
            "engine_stream": "engine.ingest",
        })
    })

    mux.HandleFunc("/api/v1/marketplace/state", func(w http.ResponseWriter, r *http.Request) {
        writeJSON(w, http.StatusOK, marketplaceRust("kogi_marketplace_state", nil, map[string]interface{}{}))
    })

    mux.HandleFunc("/api/v1/marketplace/init", func(w http.ResponseWriter, r *http.Request) {
        if r.Method != http.MethodPost {
            writeJSON(w, http.StatusMethodNotAllowed, map[string]string{"error": "POST required"})
            return
        }
        var payload map[string]interface{}
        if err := decodeBody(r, &payload); err != nil {
            writeJSON(w, http.StatusBadRequest, map[string]string{"error": err.Error()})
            return
        }
        result, err := marketplaceCallRust("kogi_marketplace_init", payload)
        if err != nil {
            writeJSON(w, http.StatusInternalServerError, map[string]string{"error": err.Error()})
            return
        }
        writeJSON(w, http.StatusOK, result)
    })

    mux.HandleFunc("/api/v1/marketplace/refresh", func(w http.ResponseWriter, r *http.Request) {
        if r.Method != http.MethodPost {
            writeJSON(w, http.StatusMethodNotAllowed, map[string]string{"error": "POST required"})
            return
        }
        result, err := marketplaceCallRust("kogi_marketplace_refresh", map[string]interface{}{})
        if err != nil {
            writeJSON(w, http.StatusInternalServerError, map[string]string{"error": err.Error()})
            return
        }
        writeJSON(w, http.StatusOK, result)
    })

    mux.HandleFunc("/api/v1/marketplace/listings", func(w http.ResponseWriter, r *http.Request) {
        if r.Method != http.MethodGet {
            writeJSON(w, http.StatusMethodNotAllowed, map[string]string{"error": "GET required"})
            return
        }
        writeJSON(w, http.StatusOK, marketplaceRust("kogi_marketplace_listings", nil, map[string]interface{}{"listings": []interface{}{}}))
    })

    mux.HandleFunc("/api/v1/marketplace/listings/publish", func(w http.ResponseWriter, r *http.Request) {
        if r.Method != http.MethodPost {
            writeJSON(w, http.StatusMethodNotAllowed, map[string]string{"error": "POST required"})
            return
        }
        var payload map[string]interface{}
        if err := decodeBody(r, &payload); err != nil {
            writeJSON(w, http.StatusBadRequest, map[string]string{"error": err.Error()})
            return
        }
        result, err := marketplaceCallRust("kogi_marketplace_publish_listing", payload)
        if err != nil {
            writeJSON(w, http.StatusInternalServerError, map[string]string{"error": err.Error()})
            return
        }
        if listing, ok := result.(map[string]interface{}); ok {
            if id, ok := listing["id"].(string); ok {
                go marketplacePublish("marketplace.listing.published", fmt.Sprintf(`{"id":%q}`, id), nil)
            }
        }
        writeJSON(w, http.StatusCreated, result)
    })

    mux.HandleFunc("/api/v1/marketplace/listings/owner/", func(w http.ResponseWriter, r *http.Request) {
        if r.Method != http.MethodGet {
            writeJSON(w, http.StatusMethodNotAllowed, map[string]string{"error": "GET required"})
            return
        }
        ownerID := trimPrefix(r, "/api/v1/marketplace/listings/owner/")
        if ownerID == "" {
            writeJSON(w, http.StatusBadRequest, map[string]string{"error": "owner_id required"})
            return
        }
        writeJSON(w, http.StatusOK, marketplaceRust("kogi_marketplace_listings_by_owner",
            map[string]string{"owner_id": ownerID},
            map[string]interface{}{"listings": []interface{}{}}))
    })

    mux.HandleFunc("/api/v1/marketplace/listings/", func(w http.ResponseWriter, r *http.Request) {
        if r.Method != http.MethodGet {
            writeJSON(w, http.StatusMethodNotAllowed, map[string]string{"error": "GET required"})
            return
        }
        listingID := trimPrefix(r, "/api/v1/marketplace/listings/")
        if listingID == "" {
            writeJSON(w, http.StatusBadRequest, map[string]string{"error": "listing_id required"})
            return
        }
        writeJSON(w, http.StatusOK, marketplaceRust("kogi_marketplace_listing",
            map[string]string{"listing_id": listingID},
            map[string]interface{}{"error": "not_found"}))
    })

    mux.HandleFunc("/api/v1/marketplace/dashboard", func(w http.ResponseWriter, r *http.Request) {
        writeJSON(w, http.StatusOK, marketplaceRust("kogi_marketplace_dashboard_snapshot", nil, map[string]interface{}{"error": "unavailable"}))
    })

    mux.HandleFunc("/api/v1/marketplace/market/overview", func(w http.ResponseWriter, r *http.Request) {
        writeJSON(w, http.StatusOK, marketplaceRust("kogi_marketplace_market_overview_snapshot", nil, map[string]interface{}{"error": "unavailable"}))
    })

    mux.HandleFunc("/api/v1/marketplace/market/browse", func(w http.ResponseWriter, r *http.Request) {
        writeJSON(w, http.StatusOK, marketplaceRust("kogi_marketplace_market_browse_snapshot", nil, map[string]interface{}{"error": "unavailable"}))
    })

    mux.HandleFunc("/api/v1/marketplace/market/labor", func(w http.ResponseWriter, r *http.Request) {
        writeJSON(w, http.StatusOK, marketplaceRust("kogi_marketplace_market_labor_snapshot", nil, map[string]interface{}{"error": "unavailable"}))
    })

    mux.HandleFunc("/api/v1/marketplace/market/grants", func(w http.ResponseWriter, r *http.Request) {
        writeJSON(w, http.StatusOK, marketplaceRust("kogi_marketplace_market_grants_snapshot", nil, map[string]interface{}{"error": "unavailable"}))
    })

    mux.HandleFunc("/api/v1/marketplace/market/crm", func(w http.ResponseWriter, r *http.Request) {
        writeJSON(w, http.StatusOK, marketplaceRust("kogi_marketplace_market_crm_snapshot", nil, map[string]interface{}{"error": "unavailable"}))
    })

    mux.HandleFunc("/api/v1/marketplace/listings/catalog", func(w http.ResponseWriter, r *http.Request) {
        writeJSON(w, http.StatusOK, marketplaceRust("kogi_marketplace_listings_catalog_snapshot", nil, map[string]interface{}{"error": "unavailable"}))
    })

    mux.HandleFunc("/api/v1/marketplace/listings/detail", func(w http.ResponseWriter, r *http.Request) {
        writeJSON(w, http.StatusOK, marketplaceRust("kogi_marketplace_listings_detail_snapshot", nil, map[string]interface{}{"error": "unavailable"}))
    })

    mux.HandleFunc("/api/v1/marketplace/listings/mine", func(w http.ResponseWriter, r *http.Request) {
        writeJSON(w, http.StatusOK, marketplaceRust("kogi_marketplace_listings_mine_snapshot", nil, map[string]interface{}{"error": "unavailable"}))
    })

    mux.HandleFunc("/api/v1/marketplace/campaigns/overview", func(w http.ResponseWriter, r *http.Request) {
        writeJSON(w, http.StatusOK, marketplaceRust("kogi_marketplace_campaigns_overview_snapshot", nil, map[string]interface{}{"error": "unavailable"}))
    })

    mux.HandleFunc("/api/v1/marketplace/campaigns/discover", func(w http.ResponseWriter, r *http.Request) {
        writeJSON(w, http.StatusOK, marketplaceRust("kogi_marketplace_campaigns_discover_snapshot", nil, map[string]interface{}{"error": "unavailable"}))
    })

    marketplaceRuntime.State("running")
    marketplaceRuntime.Status("ok", "listening="+addr)
    log.Printf("%s listening on %s", marketplaceServiceName, addr)
    log.Fatal(http.ListenAndServe(addr, ops.WithHTTPDebug(marketplaceRuntime, mux)))
}
