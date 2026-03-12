package kogiclient

import (
	"bytes"
	"encoding/json"
	"fmt"
	"io"
	"net/http"
	"net/url"
	"time"
)

type Client struct {
	BaseURL string
	HTTP    *http.Client
}

func NewClient(baseURL string) *Client {
	if baseURL == "" {
		baseURL = "http://127.0.0.1:8080"
	}
	return &Client{
		BaseURL: stringsTrimRightSlash(baseURL),
		HTTP: &http.Client{
			Timeout: 15 * time.Second,
		},
	}
}

func (c *Client) Health() (json.RawMessage, error) {
	return c.getJSON("/health")
}

func (c *Client) SystemSummary() (json.RawMessage, error) {
	return c.getJSON("/api/v1/system")
}

func (c *Client) HostSummary() (json.RawMessage, error) {
	return c.getJSON("/api/v1/host")
}

func (c *Client) HostComponents() (json.RawMessage, error) {
	return c.getJSON("/api/v1/host/components")
}

func (c *Client) Modules() (json.RawMessage, error) {
	return c.getJSON("/api/v1/modules")
}

func (c *Client) EngineSystem() (json.RawMessage, error) {
	return c.getJSON("/api/v1/engine/system")
}

func (c *Client) EngineRuntime() (json.RawMessage, error) {
	return c.getJSON("/api/v1/engine/runtime")
}

func (c *Client) EngineControl(action string) (json.RawMessage, error) {
	body := map[string]string{"action": action}
	return c.postJSON("/api/v1/engine/control", body)
}

func (c *Client) EngineIngest(payload any) (json.RawMessage, error) {
	return c.postJSON("/api/v1/engine/ingest", payload)
}

func (c *Client) DatabaseRuntime() (json.RawMessage, error) {
	return c.getJSON("/api/v1/database/runtime")
}

func (c *Client) DatabaseQuery(sql string) (json.RawMessage, error) {
	body := map[string]string{"sql": sql}
	return c.postJSON("/api/v1/database/query", body)
}

func (c *Client) Messages(limit int, topic string) (json.RawMessage, error) {
	path := fmt.Sprintf("/api/v1/messages?limit=%d", limit)
	if topic != "" {
		path = path + "&topic=" + url.QueryEscape(topic)
	}
	return c.getJSON(path)
}

func (c *Client) SendMessage(topic string, payload any, source string, target string) (json.RawMessage, error) {
	body := map[string]any{
		"topic":   topic,
		"payload": payload,
		"source":  source,
		"target":  target,
	}
	return c.postJSON("/api/v1/messages", body)
}

func (c *Client) Identities() (json.RawMessage, error) {
	return c.getJSON("/api/v1/ims/identities")
}

func (c *Client) Profiles() (json.RawMessage, error) {
	return c.getJSON("/api/v1/ims/profiles")
}

func (c *Client) AutonomyCapabilities() (json.RawMessage, error) {
	return c.getJSON("/api/v1/autonomy/capabilities")
}

func (c *Client) ModuleIsolation() (json.RawMessage, error) {
	return c.getJSON("/api/v1/kernel/modules/isolation")
}

func (c *Client) OfficeOverview() (json.RawMessage, error) {
	return c.getJSON("/api/v1/office")
}

func (c *Client) OfficeDashboard() (json.RawMessage, error) {
	return c.getJSON("/api/v1/office/dashboard")
}

func (c *Client) OfficePortfolio() (json.RawMessage, error) {
	return c.getJSON("/api/v1/office/portfolio")
}

func (c *Client) OfficeTimeline() (json.RawMessage, error) {
	return c.getJSON("/api/v1/office/timeline")
}

func (c *Client) OfficeWorkspace() (json.RawMessage, error) {
	return c.getJSON("/api/v1/office/workspace")
}

func (c *Client) OfficeAssistant() (json.RawMessage, error) {
	return c.getJSON("/api/v1/office/assistant")
}

func (c *Client) OfficeAckNotification(notificationID string) (json.RawMessage, error) {
	body := map[string]string{"notification_id": notificationID}
	return c.postJSON("/api/v1/office/dashboard/notifications/ack", body)
}

func (c *Client) OfficeCreatePortfolioItem(itemType string, name string, status string) (json.RawMessage, error) {
	body := map[string]string{
		"item_type": itemType,
		"name":      name,
		"status":    status,
	}
	return c.postJSON("/api/v1/office/portfolio/items", body)
}

func (c *Client) OfficeCreateTimelineEvent(calendarID string, title string, kind string, scheduledFor string) (json.RawMessage, error) {
	body := map[string]string{
		"calendar_id":  calendarID,
		"title":        title,
		"kind":         kind,
		"scheduled_for": scheduledFor,
	}
	return c.postJSON("/api/v1/office/timeline/events", body)
}

func (c *Client) OfficeCreateWorkspaceStory(title string, points int) (json.RawMessage, error) {
	body := map[string]any{
		"title":  title,
		"points": points,
	}
	return c.postJSON("/api/v1/office/workspace/stories", body)
}

func (c *Client) OfficeSubscribeAssistant(topic string) (json.RawMessage, error) {
	body := map[string]string{"topic": topic}
	return c.postJSON("/api/v1/office/assistant/subscriptions", body)
}

func (c *Client) ProvidersSnapshot() (json.RawMessage, error) {
	return c.getJSON("/api/v1/providers")
}

func (c *Client) ProvidersPlatforms() (json.RawMessage, error) {
	return c.getJSON("/api/v1/providers/platforms")
}

func (c *Client) ProvidersList() (json.RawMessage, error) {
	return c.getJSON("/api/v1/providers/providers")
}

func (c *Client) ProvidersResources() (json.RawMessage, error) {
	return c.getJSON("/api/v1/providers/resources")
}

func (c *Client) ProvidersVersions() (json.RawMessage, error) {
	return c.getJSON("/api/v1/providers/versions")
}

func (c *Client) ProvidersMetadata() (json.RawMessage, error) {
	return c.getJSON("/api/v1/providers/metadata")
}

func (c *Client) ProvidersDataAssets() (json.RawMessage, error) {
	return c.getJSON("/api/v1/providers/data")
}

func (c *Client) ProvidersAffiliates() (json.RawMessage, error) {
	return c.getJSON("/api/v1/providers/affiliates")
}

func (c *Client) ProvidersAffiliateLinks() (json.RawMessage, error) {
	return c.getJSON("/api/v1/providers/affiliate-links")
}

func (c *Client) ProvidersRegisterPlatform(
	name, kind, category, status, homeURL, docsURL, supportContact string,
	tags []string,
) (json.RawMessage, error) {
	body := map[string]interface{}{
		"name":            name,
		"kind":            kind,
		"category":        category,
		"status":          status,
		"home_url":        homeURL,
		"docs_url":        docsURL,
		"support_contact": supportContact,
		"tags":            tags,
	}
	return c.postJSON("/api/v1/providers/platforms", body)
}

func (c *Client) ProvidersRegister(
	name, platformID, kind, status, owner, primaryContact string,
	tags []string,
) (json.RawMessage, error) {
	body := map[string]interface{}{
		"name":            name,
		"platform_id":     platformID,
		"kind":            kind,
		"status":          status,
		"owner":           owner,
		"primary_contact": primaryContact,
		"tags":            tags,
	}
	return c.postJSON("/api/v1/providers/providers", body)
}

func (c *Client) ProvidersAddResource(
	providerID, resourceType, name, status, environment, endpoint, credentialsRef string,
) (json.RawMessage, error) {
	body := map[string]interface{}{
		"provider_id":     providerID,
		"resource_type":   resourceType,
		"name":            name,
		"status":          status,
		"environment":     environment,
		"endpoint":        endpoint,
		"credentials_ref": credentialsRef,
	}
	return c.postJSON("/api/v1/providers/resources", body)
}

func (c *Client) ProvidersAddVersion(
	providerID, version, status, releasedAt, notes string,
	compatibility []string,
) (json.RawMessage, error) {
	body := map[string]interface{}{
		"provider_id":   providerID,
		"version":       version,
		"status":        status,
		"released_at":   releasedAt,
		"notes":         notes,
		"compatibility": compatibility,
	}
	return c.postJSON("/api/v1/providers/versions", body)
}

func (c *Client) ProvidersSetMetadata(providerID, key, value, scope string) (json.RawMessage, error) {
	body := map[string]interface{}{
		"provider_id": providerID,
		"key":         key,
		"value":       value,
		"scope":       scope,
	}
	return c.postJSON("/api/v1/providers/metadata", body)
}

func (c *Client) ProvidersAddDataAsset(
	providerID, dataset, status string,
	recordCount int,
	storage, lastSync string,
) (json.RawMessage, error) {
	body := map[string]interface{}{
		"provider_id":  providerID,
		"dataset":      dataset,
		"status":       status,
		"record_count": recordCount,
		"storage":      storage,
		"last_sync":    lastSync,
	}
	return c.postJSON("/api/v1/providers/data", body)
}

func (c *Client) ProvidersRegisterAffiliate(
	name, kind, status, website, contact string,
	tags []string,
) (json.RawMessage, error) {
	body := map[string]interface{}{
		"name":    name,
		"kind":    kind,
		"status":  status,
		"website": website,
		"contact": contact,
		"tags":    tags,
	}
	return c.postJSON("/api/v1/providers/affiliates", body)
}

func (c *Client) ProvidersAddAffiliateLink(
	providerID, affiliateID, status, channel, trackingURL, contractRef string,
) (json.RawMessage, error) {
	body := map[string]interface{}{
		"provider_id":  providerID,
		"affiliate_id": affiliateID,
		"status":       status,
		"channel":      channel,
		"tracking_url": trackingURL,
		"contract_ref": contractRef,
	}
	return c.postJSON("/api/v1/providers/affiliate-links", body)
}

func (c *Client) UnifiedScreens() (json.RawMessage, error) {
	return c.getJSON("/api/v1/screens/unified")
}

func (c *Client) UnifiedScreensFlat() (string, error) {
	return c.getText("/api/v1/screens/unified/flat")
}

func (c *Client) getJSON(path string) (json.RawMessage, error) {
	return c.doJSON(http.MethodGet, path, nil)
}

func (c *Client) postJSON(path string, payload any) (json.RawMessage, error) {
	return c.doJSON(http.MethodPost, path, payload)
}

func (c *Client) doJSON(method string, path string, payload any) (json.RawMessage, error) {
	var body io.Reader
	if payload != nil {
		encoded, err := json.Marshal(payload)
		if err != nil {
			return nil, err
		}
		body = bytes.NewReader(encoded)
	}
	req, err := http.NewRequest(method, c.BaseURL+path, body)
	if err != nil {
		return nil, err
	}
	req.Header.Set("Content-Type", "application/json")
	resp, err := c.HTTP.Do(req)
	if err != nil {
		return nil, err
	}
	defer resp.Body.Close()
	data, _ := io.ReadAll(resp.Body)
	if resp.StatusCode < 200 || resp.StatusCode >= 300 {
		return nil, fmt.Errorf("status=%d body=%s", resp.StatusCode, string(data))
	}
	return json.RawMessage(data), nil
}

func (c *Client) getText(path string) (string, error) {
	req, err := http.NewRequest(http.MethodGet, c.BaseURL+path, nil)
	if err != nil {
		return "", err
	}
	resp, err := c.HTTP.Do(req)
	if err != nil {
		return "", err
	}
	defer resp.Body.Close()
	data, _ := io.ReadAll(resp.Body)
	if resp.StatusCode < 200 || resp.StatusCode >= 300 {
		return "", fmt.Errorf("status=%d body=%s", resp.StatusCode, string(data))
	}
	return string(data), nil
}

func stringsTrimRightSlash(value string) string {
	for len(value) > 0 && value[len(value)-1] == '/' {
		value = value[:len(value)-1]
	}
	return value
}
