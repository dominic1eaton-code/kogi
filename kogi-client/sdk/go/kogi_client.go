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
