package main

import (
	"encoding/json"
	"fmt"
	"log"
	"net/http"
	"os"
	"strings"
	"time"

	"kogi.network/lib/ops"
)

type providerPlatform struct {
	ID             string            `json:"id"`
	Name           string            `json:"name"`
	Kind           string            `json:"kind"`
	Category       string            `json:"category"`
	Status         string            `json:"status"`
	HomeURL        string            `json:"home_url"`
	DocsURL        string            `json:"docs_url"`
	SupportContact string            `json:"support_contact"`
	Tags           []string          `json:"tags"`
	Metadata       map[string]string `json:"metadata"`
	CreatedAt      string            `json:"created_at"`
	UpdatedAt      string            `json:"updated_at"`
}

type providerVersion struct {
	ID            string   `json:"id"`
	ProviderID    string   `json:"provider_id"`
	Version       string   `json:"version"`
	Status        string   `json:"status"`
	ReleasedAt    string   `json:"released_at"`
	Notes         string   `json:"notes"`
	Compatibility []string `json:"compatibility"`
}

type providerResource struct {
	ID             string `json:"id"`
	ProviderID     string `json:"provider_id"`
	ResourceType   string `json:"resource_type"`
	Name           string `json:"name"`
	Status         string `json:"status"`
	Environment    string `json:"environment"`
	Endpoint       string `json:"endpoint"`
	CredentialsRef string `json:"credentials_ref"`
	LastChecked    string `json:"last_checked"`
}

type providerMetadata struct {
	ID         string `json:"id"`
	ProviderID string `json:"provider_id"`
	Key        string `json:"key"`
	Value      string `json:"value"`
	Scope      string `json:"scope"`
	UpdatedAt  string `json:"updated_at"`
}

type providerDataAsset struct {
	ID          string `json:"id"`
	ProviderID  string `json:"provider_id"`
	Dataset     string `json:"dataset"`
	Status      string `json:"status"`
	RecordCount int    `json:"record_count"`
	Storage     string `json:"storage"`
	LastSync    string `json:"last_sync"`
}

type providerRecord struct {
	ID             string              `json:"id"`
	Name           string              `json:"name"`
	PlatformID     string              `json:"platform_id"`
	Kind           string              `json:"kind"`
	Status         string              `json:"status"`
	Owner          string              `json:"owner"`
	PrimaryContact string              `json:"primary_contact"`
	Tags           []string            `json:"tags"`
	CurrentVersion string              `json:"current_version"`
	Versions       []providerVersion   `json:"versions"`
	Resources      []providerResource  `json:"resources"`
	Metadata       []providerMetadata  `json:"metadata"`
	DataAssets     []providerDataAsset `json:"data_assets"`
	CreatedAt      string              `json:"created_at"`
	UpdatedAt      string              `json:"updated_at"`
}

type providerTotals struct {
	Platforms       int `json:"platforms"`
	Providers       int `json:"providers"`
	Resources       int `json:"resources"`
	Versions        int `json:"versions"`
	MetadataEntries int `json:"metadata_entries"`
	DataAssets      int `json:"data_assets"`
	Affiliates      int `json:"affiliates"`
	AffiliateLinks  int `json:"affiliate_links"`
	ActiveProviders int `json:"active_providers"`
	ActivePlatforms int `json:"active_platforms"`
	ActiveAffiliates int `json:"active_affiliates"`
	ActiveAffiliateLinks int `json:"active_affiliate_links"`
}

type providerSnapshot struct {
	View      string              `json:"view"`
	Totals    providerTotals      `json:"totals"`
	Platforms []providerPlatform  `json:"platforms"`
	Providers []providerRecord    `json:"providers"`
	Resources []providerResource  `json:"resources"`
	Versions  []providerVersion   `json:"versions"`
	Metadata  []providerMetadata  `json:"metadata"`
	Data      []providerDataAsset `json:"data_assets"`
	Affiliates []affiliateRecord   `json:"affiliates"`
	AffiliateLinks []affiliateLink `json:"affiliate_links"`
}

type newPlatformRequest struct {
	Name           string            `json:"name"`
	Kind           string            `json:"kind"`
	Category       string            `json:"category"`
	Status         string            `json:"status"`
	HomeURL        string            `json:"home_url"`
	DocsURL        string            `json:"docs_url"`
	SupportContact string            `json:"support_contact"`
	Tags           []string          `json:"tags"`
	Metadata       map[string]string `json:"metadata"`
}

type newProviderRequest struct {
	Name           string   `json:"name"`
	PlatformID     string   `json:"platform_id"`
	Kind           string   `json:"kind"`
	Status         string   `json:"status"`
	Owner          string   `json:"owner"`
	PrimaryContact string   `json:"primary_contact"`
	Tags           []string `json:"tags"`
}

type newResourceRequest struct {
	ProviderID     string `json:"provider_id"`
	ResourceType   string `json:"resource_type"`
	Name           string `json:"name"`
	Status         string `json:"status"`
	Environment    string `json:"environment"`
	Endpoint       string `json:"endpoint"`
	CredentialsRef string `json:"credentials_ref"`
}

type newVersionRequest struct {
	ProviderID    string   `json:"provider_id"`
	Version       string   `json:"version"`
	Status        string   `json:"status"`
	ReleasedAt    string   `json:"released_at"`
	Notes         string   `json:"notes"`
	Compatibility []string `json:"compatibility"`
}

type newMetadataRequest struct {
	ProviderID string `json:"provider_id"`
	Key        string `json:"key"`
	Value      string `json:"value"`
	Scope      string `json:"scope"`
}

type newDataAssetRequest struct {
	ProviderID  string `json:"provider_id"`
	Dataset     string `json:"dataset"`
	Status      string `json:"status"`
	RecordCount int    `json:"record_count"`
	Storage     string `json:"storage"`
	LastSync    string `json:"last_sync"`
}

type affiliateRecord struct {
	ID        string            `json:"id"`
	Name      string            `json:"name"`
	Kind      string            `json:"kind"`
	Status    string            `json:"status"`
	Website   string            `json:"website"`
	Contact   string            `json:"contact"`
	Tags      []string          `json:"tags"`
	Metadata  map[string]string `json:"metadata"`
	CreatedAt string            `json:"created_at"`
	UpdatedAt string            `json:"updated_at"`
}

type affiliateLink struct {
	ID          string `json:"id"`
	ProviderID  string `json:"provider_id"`
	AffiliateID string `json:"affiliate_id"`
	Status      string `json:"status"`
	Channel     string `json:"channel"`
	TrackingURL string `json:"tracking_url"`
	ContractRef string `json:"contract_ref"`
	StartedAt   string `json:"started_at"`
	UpdatedAt   string `json:"updated_at"`
}

type newAffiliateRequest struct {
	Name     string            `json:"name"`
	Kind     string            `json:"kind"`
	Status   string            `json:"status"`
	Website  string            `json:"website"`
	Contact  string            `json:"contact"`
	Tags     []string          `json:"tags"`
	Metadata map[string]string `json:"metadata"`
}

type newAffiliateLinkRequest struct {
	ProviderID  string `json:"provider_id"`
	AffiliateID string `json:"affiliate_id"`
	Status      string `json:"status"`
	Channel     string `json:"channel"`
	TrackingURL string `json:"tracking_url"`
	ContractRef string `json:"contract_ref"`
}

func main() {
	rt := ops.Init("providers-service")
	rt.State("init")

	platforms := []providerPlatform{
		{
			ID:             "platform-001",
			Name:           "GitHub",
			Kind:           "platform",
			Category:       "developer",
			Status:         "active",
			HomeURL:        "https://github.com",
			DocsURL:        "https://docs.github.com",
			SupportContact: "support@github.com",
			Tags:           []string{"code", "repo", "api"},
			Metadata:       map[string]string{"auth": "oauth2"},
			CreatedAt:      defaultTimestamp(),
			UpdatedAt:      defaultTimestamp(),
		},
	}

	providers := []providerRecord{
		{
			ID:             "provider-001",
			Name:           "GitHub API",
			PlatformID:     "platform-001",
			Kind:           "api",
			Status:         "active",
			Owner:          "platform",
			PrimaryContact: "devrel@github.com",
			Tags:           []string{"oauth", "webhooks"},
			CurrentVersion: "v3",
			Versions: []providerVersion{
				{
					ID:            "version-001",
					ProviderID:    "provider-001",
					Version:       "v3",
					Status:        "stable",
					ReleasedAt:    defaultTimestamp(),
					Notes:         "Primary REST API",
					Compatibility: []string{"oauth2", "webhooks"},
				},
			},
			Resources: []providerResource{
				{
					ID:             "resource-001",
					ProviderID:     "provider-001",
					ResourceType:   "webhook",
					Name:           "repo-events",
					Status:         "active",
					Environment:    "production",
					Endpoint:       "https://api.github.com/hooks",
					CredentialsRef: "vault/github/webhook",
					LastChecked:    defaultTimestamp(),
				},
			},
			Metadata: []providerMetadata{
				{
					ID:         "meta-001",
					ProviderID: "provider-001",
					Key:        "rate_limit",
					Value:      "5000/hr",
					Scope:      "api",
					UpdatedAt:  defaultTimestamp(),
				},
			},
			DataAssets: []providerDataAsset{
				{
					ID:          "data-001",
					ProviderID:  "provider-001",
					Dataset:     "repo-sync",
					Status:      "synced",
					RecordCount: 320,
					Storage:     "kogi-data/providers/github",
					LastSync:    defaultTimestamp(),
				},
			},
			CreatedAt: defaultTimestamp(),
			UpdatedAt: defaultTimestamp(),
		},
	}

	nextPlatformID := 2
	nextProviderID := 2
	nextResourceID := 2
	nextVersionID := 2
	nextMetadataID := 2
	nextDataID := 2
	nextAffiliateID := 2
	nextAffiliateLinkID := 2

	affiliates := []affiliateRecord{
		{
			ID:        "affiliate-001",
			Name:      "Kogi Partner Network",
			Kind:      "partner",
			Status:    "active",
			Website:   "https://partners.kogi.local",
			Contact:   "partners@kogi.local",
			Tags:      []string{"affiliate", "partner"},
			Metadata:  map[string]string{"tier": "gold"},
			CreatedAt: defaultTimestamp(),
			UpdatedAt: defaultTimestamp(),
		},
	}

	affiliateLinks := []affiliateLink{
		{
			ID:          "aff-link-001",
			ProviderID:  "provider-001",
			AffiliateID: "affiliate-001",
			Status:      "active",
			Channel:     "referral",
			TrackingURL: "https://kogi.local/track/github",
			ContractRef: "contract-aff-001",
			StartedAt:   defaultTimestamp(),
			UpdatedAt:   defaultTimestamp(),
		},
	}

	mux := http.NewServeMux()
	rt.State("configure")
	rt.WatchSignals()

	mux.HandleFunc("/health", func(w http.ResponseWriter, r *http.Request) {
		writeJSON(w, http.StatusOK, map[string]string{"status": "ok", "service": "providers-service"})
	})

	mux.HandleFunc("/api/v1/providers/runtime", func(w http.ResponseWriter, r *http.Request) {
		writeJSON(w, http.StatusOK, map[string]interface{}{
			"service":         "providers-service",
			"component_id":    "kogi.network.providers",
			"gateway":         "http://127.0.0.1:8090",
			"network_manager": "kogi-go-network",
			"publishes": []string{
				"provider.platform.registered",
				"provider.registered",
				"provider.resource.added",
				"provider.version.published",
				"provider.metadata.updated",
				"provider.data.asset.updated",
				"provider.affiliate.registered",
				"provider.affiliate.linked",
			},
			"subscribes": []string{
				"provider.sync.requested",
			},
		})
	})

	mux.HandleFunc("/api/v1/providers", func(w http.ResponseWriter, r *http.Request) {
		if r.Method != http.MethodGet {
			writeJSON(w, http.StatusMethodNotAllowed, map[string]string{"error": "method_not_allowed"})
			return
		}
		writeJSON(w, http.StatusOK, snapshot(platforms, providers, affiliates, affiliateLinks))
	})

	mux.HandleFunc("/api/v1/providers/platforms", func(w http.ResponseWriter, r *http.Request) {
		switch r.Method {
		case http.MethodGet:
			writeJSON(w, http.StatusOK, map[string]interface{}{"platforms": platforms})
		case http.MethodPost:
			var req newPlatformRequest
			if err := decodeJSON(r, &req); err != nil {
				writeJSON(w, http.StatusBadRequest, map[string]string{"error": err.Error()})
				return
			}
			if req.Status == "" {
				req.Status = "active"
			}
			platform := providerPlatform{
				ID:             fmt.Sprintf("platform-%03d", nextPlatformID),
				Name:           req.Name,
				Kind:           req.Kind,
				Category:       req.Category,
				Status:         req.Status,
				HomeURL:        req.HomeURL,
				DocsURL:        req.DocsURL,
				SupportContact: req.SupportContact,
				Tags:           req.Tags,
				Metadata:       defaultMap(req.Metadata),
				CreatedAt:      defaultTimestamp(),
				UpdatedAt:      defaultTimestamp(),
			}
			nextPlatformID++
			platforms = append(platforms, platform)
			writeJSON(w, http.StatusCreated, map[string]interface{}{"platform": platform})
		default:
			writeJSON(w, http.StatusMethodNotAllowed, map[string]string{"error": "method_not_allowed"})
		}
	})

	mux.HandleFunc("/api/v1/providers/providers", func(w http.ResponseWriter, r *http.Request) {
		switch r.Method {
		case http.MethodGet:
			writeJSON(w, http.StatusOK, map[string]interface{}{"providers": providers})
		case http.MethodPost:
			var req newProviderRequest
			if err := decodeJSON(r, &req); err != nil {
				writeJSON(w, http.StatusBadRequest, map[string]string{"error": err.Error()})
				return
			}
			provider := providerRecord{
				ID:             fmt.Sprintf("provider-%03d", nextProviderID),
				Name:           req.Name,
				PlatformID:     req.PlatformID,
				Kind:           req.Kind,
				Status:         defaultString(req.Status, "active"),
				Owner:          defaultString(req.Owner, "external"),
				PrimaryContact: req.PrimaryContact,
				Tags:           req.Tags,
				CurrentVersion: "unversioned",
				Versions:       []providerVersion{},
				Resources:      []providerResource{},
				Metadata:       []providerMetadata{},
				DataAssets:     []providerDataAsset{},
				CreatedAt:      defaultTimestamp(),
				UpdatedAt:      defaultTimestamp(),
			}
			nextProviderID++
			providers = append(providers, provider)
			writeJSON(w, http.StatusCreated, map[string]interface{}{"provider": provider})
		default:
			writeJSON(w, http.StatusMethodNotAllowed, map[string]string{"error": "method_not_allowed"})
		}
	})

	mux.HandleFunc("/api/v1/providers/resources", func(w http.ResponseWriter, r *http.Request) {
		switch r.Method {
		case http.MethodGet:
			writeJSON(w, http.StatusOK, map[string]interface{}{"resources": allResources(providers)})
		case http.MethodPost:
			var req newResourceRequest
			if err := decodeJSON(r, &req); err != nil {
				writeJSON(w, http.StatusBadRequest, map[string]string{"error": err.Error()})
				return
			}
			p, ok := findProvider(providers, req.ProviderID)
			if !ok {
				writeJSON(w, http.StatusNotFound, map[string]string{"error": "provider_not_found"})
				return
			}
			resource := providerResource{
				ID:             fmt.Sprintf("resource-%03d", nextResourceID),
				ProviderID:     req.ProviderID,
				ResourceType:   req.ResourceType,
				Name:           req.Name,
				Status:         defaultString(req.Status, "active"),
				Environment:    defaultString(req.Environment, "production"),
				Endpoint:       req.Endpoint,
				CredentialsRef: req.CredentialsRef,
				LastChecked:    defaultTimestamp(),
			}
			nextResourceID++
			p.Resources = append(p.Resources, resource)
			writeJSON(w, http.StatusCreated, map[string]interface{}{"resource": resource})
		default:
			writeJSON(w, http.StatusMethodNotAllowed, map[string]string{"error": "method_not_allowed"})
		}
	})

	mux.HandleFunc("/api/v1/providers/versions", func(w http.ResponseWriter, r *http.Request) {
		switch r.Method {
		case http.MethodGet:
			writeJSON(w, http.StatusOK, map[string]interface{}{"versions": allVersions(providers)})
		case http.MethodPost:
			var req newVersionRequest
			if err := decodeJSON(r, &req); err != nil {
				writeJSON(w, http.StatusBadRequest, map[string]string{"error": err.Error()})
				return
			}
			p, ok := findProvider(providers, req.ProviderID)
			if !ok {
				writeJSON(w, http.StatusNotFound, map[string]string{"error": "provider_not_found"})
				return
			}
			version := providerVersion{
				ID:            fmt.Sprintf("version-%03d", nextVersionID),
				ProviderID:    req.ProviderID,
				Version:       req.Version,
				Status:        defaultString(req.Status, "stable"),
				ReleasedAt:    defaultString(req.ReleasedAt, defaultTimestamp()),
				Notes:         req.Notes,
				Compatibility: req.Compatibility,
			}
			nextVersionID++
			p.CurrentVersion = req.Version
			p.Versions = append(p.Versions, version)
			writeJSON(w, http.StatusCreated, map[string]interface{}{"version": version})
		default:
			writeJSON(w, http.StatusMethodNotAllowed, map[string]string{"error": "method_not_allowed"})
		}
	})

	mux.HandleFunc("/api/v1/providers/metadata", func(w http.ResponseWriter, r *http.Request) {
		switch r.Method {
		case http.MethodGet:
			writeJSON(w, http.StatusOK, map[string]interface{}{"metadata": allMetadata(providers)})
		case http.MethodPost:
			var req newMetadataRequest
			if err := decodeJSON(r, &req); err != nil {
				writeJSON(w, http.StatusBadRequest, map[string]string{"error": err.Error()})
				return
			}
			p, ok := findProvider(providers, req.ProviderID)
			if !ok {
				writeJSON(w, http.StatusNotFound, map[string]string{"error": "provider_not_found"})
				return
			}
			entry := providerMetadata{
				ID:         fmt.Sprintf("meta-%03d", nextMetadataID),
				ProviderID: req.ProviderID,
				Key:        req.Key,
				Value:      req.Value,
				Scope:      defaultString(req.Scope, "general"),
				UpdatedAt:  defaultTimestamp(),
			}
			nextMetadataID++
			p.Metadata = append(p.Metadata, entry)
			writeJSON(w, http.StatusCreated, map[string]interface{}{"metadata": entry})
		default:
			writeJSON(w, http.StatusMethodNotAllowed, map[string]string{"error": "method_not_allowed"})
		}
	})

	mux.HandleFunc("/api/v1/providers/data", func(w http.ResponseWriter, r *http.Request) {
		switch r.Method {
		case http.MethodGet:
			writeJSON(w, http.StatusOK, map[string]interface{}{"data_assets": allDataAssets(providers)})
		case http.MethodPost:
			var req newDataAssetRequest
			if err := decodeJSON(r, &req); err != nil {
				writeJSON(w, http.StatusBadRequest, map[string]string{"error": err.Error()})
				return
			}
			p, ok := findProvider(providers, req.ProviderID)
			if !ok {
				writeJSON(w, http.StatusNotFound, map[string]string{"error": "provider_not_found"})
				return
			}
			asset := providerDataAsset{
				ID:          fmt.Sprintf("data-%03d", nextDataID),
				ProviderID:  req.ProviderID,
				Dataset:     req.Dataset,
				Status:      defaultString(req.Status, "active"),
				RecordCount: req.RecordCount,
				Storage:     req.Storage,
				LastSync:    defaultString(req.LastSync, defaultTimestamp()),
			}
			nextDataID++
			p.DataAssets = append(p.DataAssets, asset)
			writeJSON(w, http.StatusCreated, map[string]interface{}{"data_asset": asset})
		default:
			writeJSON(w, http.StatusMethodNotAllowed, map[string]string{"error": "method_not_allowed"})
		}
	})

	mux.HandleFunc("/api/v1/providers/affiliates", func(w http.ResponseWriter, r *http.Request) {
		switch r.Method {
		case http.MethodGet:
			writeJSON(w, http.StatusOK, map[string]interface{}{"affiliates": affiliates})
		case http.MethodPost:
			var req newAffiliateRequest
			if err := decodeJSON(r, &req); err != nil {
				writeJSON(w, http.StatusBadRequest, map[string]string{"error": err.Error()})
				return
			}
			affiliate := affiliateRecord{
				ID:        fmt.Sprintf("affiliate-%03d", nextAffiliateID),
				Name:      req.Name,
				Kind:      req.Kind,
				Status:    defaultString(req.Status, "active"),
				Website:   req.Website,
				Contact:   req.Contact,
				Tags:      req.Tags,
				Metadata:  defaultMap(req.Metadata),
				CreatedAt: defaultTimestamp(),
				UpdatedAt: defaultTimestamp(),
			}
			nextAffiliateID++
			affiliates = append(affiliates, affiliate)
			writeJSON(w, http.StatusCreated, map[string]interface{}{"affiliate": affiliate})
		default:
			writeJSON(w, http.StatusMethodNotAllowed, map[string]string{"error": "method_not_allowed"})
		}
	})

	mux.HandleFunc("/api/v1/providers/affiliate-links", func(w http.ResponseWriter, r *http.Request) {
		switch r.Method {
		case http.MethodGet:
			writeJSON(w, http.StatusOK, map[string]interface{}{"affiliate_links": affiliateLinks})
		case http.MethodPost:
			var req newAffiliateLinkRequest
			if err := decodeJSON(r, &req); err != nil {
				writeJSON(w, http.StatusBadRequest, map[string]string{"error": err.Error()})
				return
			}
			link := affiliateLink{
				ID:          fmt.Sprintf("aff-link-%03d", nextAffiliateLinkID),
				ProviderID:  req.ProviderID,
				AffiliateID: req.AffiliateID,
				Status:      defaultString(req.Status, "active"),
				Channel:     defaultString(req.Channel, "referral"),
				TrackingURL: req.TrackingURL,
				ContractRef: req.ContractRef,
				StartedAt:   defaultTimestamp(),
				UpdatedAt:   defaultTimestamp(),
			}
			nextAffiliateLinkID++
			affiliateLinks = append(affiliateLinks, link)
			writeJSON(w, http.StatusCreated, map[string]interface{}{"affiliate_link": link})
		default:
			writeJSON(w, http.StatusMethodNotAllowed, map[string]string{"error": "method_not_allowed"})
		}
	})

	addr := resolveAddr("9016", "KOGI_PROVIDERS_PORT")
	rt.State("running")
	rt.Status("ok", "listening="+addr)
	log.Printf("providers-service listening on %s", addr)
	log.Fatal(http.ListenAndServe(addr, ops.WithHTTPDebug(rt, mux)))
}

func snapshot(
	platforms []providerPlatform,
	providers []providerRecord,
	affiliates []affiliateRecord,
	affiliateLinks []affiliateLink,
) providerSnapshot {
	resources := allResources(providers)
	versions := allVersions(providers)
	metadata := allMetadata(providers)
	dataAssets := allDataAssets(providers)
	activeProviders := 0
	activePlatforms := 0
	activeAffiliates := 0
	activeAffiliateLinks := 0
	for _, p := range providers {
		if p.Status == "active" {
			activeProviders++
		}
	}
	for _, p := range platforms {
		if p.Status == "active" {
			activePlatforms++
		}
	}
	for _, a := range affiliates {
		if a.Status == "active" {
			activeAffiliates++
		}
	}
	for _, l := range affiliateLinks {
		if l.Status == "active" {
			activeAffiliateLinks++
		}
	}
	return providerSnapshot{
		View: "providers.registry",
		Totals: providerTotals{
			Platforms:       len(platforms),
			Providers:       len(providers),
			Resources:       len(resources),
			Versions:        len(versions),
			MetadataEntries: len(metadata),
			DataAssets:      len(dataAssets),
			Affiliates:      len(affiliates),
			AffiliateLinks:  len(affiliateLinks),
			ActiveProviders: activeProviders,
			ActivePlatforms: activePlatforms,
			ActiveAffiliates: activeAffiliates,
			ActiveAffiliateLinks: activeAffiliateLinks,
		},
		Platforms: platforms,
		Providers: providers,
		Resources: resources,
		Versions:  versions,
		Metadata:  metadata,
		Data:      dataAssets,
		Affiliates: affiliates,
		AffiliateLinks: affiliateLinks,
	}
}

func findProvider(providers []providerRecord, id string) (*providerRecord, bool) {
	for i := range providers {
		if providers[i].ID == id {
			return &providers[i], true
		}
	}
	return nil, false
}

func allResources(providers []providerRecord) []providerResource {
	var out []providerResource
	for _, p := range providers {
		out = append(out, p.Resources...)
	}
	return out
}

func allVersions(providers []providerRecord) []providerVersion {
	var out []providerVersion
	for _, p := range providers {
		out = append(out, p.Versions...)
	}
	return out
}

func allMetadata(providers []providerRecord) []providerMetadata {
	var out []providerMetadata
	for _, p := range providers {
		out = append(out, p.Metadata...)
	}
	return out
}

func allDataAssets(providers []providerRecord) []providerDataAsset {
	var out []providerDataAsset
	for _, p := range providers {
		out = append(out, p.DataAssets...)
	}
	return out
}

func writeJSON(w http.ResponseWriter, status int, payload interface{}) {
	w.Header().Set("Content-Type", "application/json")
	w.WriteHeader(status)
	_ = json.NewEncoder(w).Encode(payload)
}

func decodeJSON(r *http.Request, dst interface{}) error {
	defer r.Body.Close()
	return json.NewDecoder(r.Body).Decode(dst)
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

func defaultTimestamp() string {
	return time.Now().UTC().Format(time.RFC3339)
}

func defaultString(value, fallback string) string {
	if value == "" {
		return fallback
	}
	return value
}

func defaultMap(value map[string]string) map[string]string {
	if value == nil {
		return map[string]string{}
	}
	return value
}
