using System.Net.Http;
using System.Text;
using System.Text.Json;

namespace Kogi.Client;

public sealed class KogiClient : IDisposable
{
    private readonly HttpClient _http;
    private readonly string _baseUrl;

    public KogiClient(string baseUrl = "http://127.0.0.1:8080")
    {
        _baseUrl = baseUrl.TrimEnd('/');
        _http = new HttpClient { Timeout = TimeSpan.FromSeconds(15) };
    }

    public Task<string> HealthAsync() => GetAsync("/health");
    public Task<string> SystemSummaryAsync() => GetAsync("/api/v1/system");
    public Task<string> HostSummaryAsync() => GetAsync("/api/v1/host");
    public Task<string> HostComponentsAsync() => GetAsync("/api/v1/host/components");
    public Task<string> ModulesAsync() => GetAsync("/api/v1/modules");
    public Task<string> EngineSystemAsync() => GetAsync("/api/v1/engine/system");
    public Task<string> EngineRuntimeAsync() => GetAsync("/api/v1/engine/runtime");
    public Task<string> DatabaseRuntimeAsync() => GetAsync("/api/v1/database/runtime");
    public Task<string> IdentitiesAsync() => GetAsync("/api/v1/ims/identities");
    public Task<string> ProfilesAsync() => GetAsync("/api/v1/ims/profiles");
    public Task<string> AutonomyCapabilitiesAsync() => GetAsync("/api/v1/autonomy/capabilities");
    public Task<string> ModuleIsolationAsync() => GetAsync("/api/v1/kernel/modules/isolation");
    public Task<string> OfficeOverviewAsync() => GetAsync("/api/v1/office");
    public Task<string> OfficeDashboardAsync() => GetAsync("/api/v1/office/dashboard");
    public Task<string> OfficePortfolioAsync() => GetAsync("/api/v1/office/portfolio");
    public Task<string> OfficeTimelineAsync() => GetAsync("/api/v1/office/timeline");
    public Task<string> OfficeWorkspaceAsync() => GetAsync("/api/v1/office/workspace");
    public Task<string> OfficeAssistantAsync() => GetAsync("/api/v1/office/assistant");
    public Task<string> UnifiedScreensAsync() => GetAsync("/api/v1/screens/unified");
    public Task<string> UnifiedScreensFlatAsync() => GetAsync("/api/v1/screens/unified/flat");
    public Task<string> ProvidersSnapshotAsync() => GetAsync("/api/v1/providers");
    public Task<string> ProvidersPlatformsAsync() => GetAsync("/api/v1/providers/platforms");
    public Task<string> ProvidersListAsync() => GetAsync("/api/v1/providers/providers");
    public Task<string> ProvidersResourcesAsync() => GetAsync("/api/v1/providers/resources");
    public Task<string> ProvidersVersionsAsync() => GetAsync("/api/v1/providers/versions");
    public Task<string> ProvidersMetadataAsync() => GetAsync("/api/v1/providers/metadata");
    public Task<string> ProvidersDataAssetsAsync() => GetAsync("/api/v1/providers/data");
    public Task<string> ProvidersAffiliatesAsync() => GetAsync("/api/v1/providers/affiliates");
    public Task<string> ProvidersAffiliateLinksAsync() => GetAsync("/api/v1/providers/affiliate-links");

    public Task<string> EngineControlAsync(string action) =>
        PostJsonAsync("/api/v1/engine/control", new { action });

    public Task<string> EngineIngestJsonAsync(object payload) =>
        PostJsonAsync("/api/v1/engine/ingest", payload);

    public Task<string> EngineIngestRawAsync(string payload) =>
        PostRawJsonAsync("/api/v1/engine/ingest", payload);

    public Task<string> DatabaseQueryAsync(string sql) =>
        PostJsonAsync("/api/v1/database/query", new { sql });

    public Task<string> MessagesAsync(int limit = 100, string? topic = null)
    {
        var path = $"/api/v1/messages?limit={limit}";
        if (!string.IsNullOrWhiteSpace(topic))
        {
            path += "&topic=" + Uri.EscapeDataString(topic);
        }
        return GetAsync(path);
    }

    public Task<string> SendMessageAsync(string topic, object payload, string source = "client", string target = "")
    {
        return PostJsonAsync("/api/v1/messages", new { topic, payload, source, target });
    }

    public Task<string> OfficeAckNotificationAsync(string notificationId) =>
        PostJsonAsync("/api/v1/office/dashboard/notifications/ack", new { notification_id = notificationId });

    public Task<string> OfficeCreatePortfolioItemAsync(string itemType, string name, string status) =>
        PostJsonAsync("/api/v1/office/portfolio/items", new { item_type = itemType, name, status });

    public Task<string> OfficeCreateTimelineEventAsync(
        string calendarId, string title, string kind, string scheduledFor) =>
        PostJsonAsync("/api/v1/office/timeline/events",
            new { calendar_id = calendarId, title, kind, scheduled_for = scheduledFor });

    public Task<string> OfficeCreateWorkspaceStoryAsync(string title, int points) =>
        PostJsonAsync("/api/v1/office/workspace/stories", new { title, points });

    public Task<string> OfficeSubscribeAssistantAsync(string topic) =>
        PostJsonAsync("/api/v1/office/assistant/subscriptions", new { topic });

    public Task<string> ProvidersRegisterPlatformAsync(
        string name,
        string kind,
        string category,
        string status,
        string homeUrl = "",
        string docsUrl = "",
        string supportContact = "",
        string[]? tags = null
    ) => PostJsonAsync("/api/v1/providers/platforms", new
    {
        name,
        kind,
        category,
        status,
        home_url = homeUrl,
        docs_url = docsUrl,
        support_contact = supportContact,
        tags = tags ?? Array.Empty<string>(),
    });

    public Task<string> ProvidersRegisterAsync(
        string name,
        string platformId,
        string kind,
        string status,
        string owner = "",
        string primaryContact = "",
        string[]? tags = null
    ) => PostJsonAsync("/api/v1/providers/providers", new
    {
        name,
        platform_id = platformId,
        kind,
        status,
        owner,
        primary_contact = primaryContact,
        tags = tags ?? Array.Empty<string>(),
    });

    public Task<string> ProvidersAddResourceAsync(
        string providerId,
        string resourceType,
        string name,
        string status,
        string environment = "",
        string endpoint = "",
        string credentialsRef = ""
    ) => PostJsonAsync("/api/v1/providers/resources", new
    {
        provider_id = providerId,
        resource_type = resourceType,
        name,
        status,
        environment,
        endpoint,
        credentials_ref = credentialsRef,
    });

    public Task<string> ProvidersAddVersionAsync(
        string providerId,
        string version,
        string status,
        string releasedAt = "",
        string notes = "",
        string[]? compatibility = null
    ) => PostJsonAsync("/api/v1/providers/versions", new
    {
        provider_id = providerId,
        version,
        status,
        released_at = releasedAt,
        notes,
        compatibility = compatibility ?? Array.Empty<string>(),
    });

    public Task<string> ProvidersSetMetadataAsync(
        string providerId,
        string key,
        string value,
        string scope = ""
    ) => PostJsonAsync("/api/v1/providers/metadata", new
    {
        provider_id = providerId,
        key,
        value,
        scope,
    });

    public Task<string> ProvidersAddDataAssetAsync(
        string providerId,
        string dataset,
        string status,
        int recordCount = 0,
        string storage = "",
        string lastSync = ""
    ) => PostJsonAsync("/api/v1/providers/data", new
    {
        provider_id = providerId,
        dataset,
        status,
        record_count = recordCount,
        storage,
        last_sync = lastSync,
    });

    public Task<string> ProvidersRegisterAffiliateAsync(
        string name,
        string kind,
        string status,
        string website = "",
        string contact = "",
        string[]? tags = null
    ) => PostJsonAsync("/api/v1/providers/affiliates", new
    {
        name,
        kind,
        status,
        website,
        contact,
        tags = tags ?? Array.Empty<string>(),
    });

    public Task<string> ProvidersAddAffiliateLinkAsync(
        string providerId,
        string affiliateId,
        string status,
        string channel = "",
        string trackingUrl = "",
        string contractRef = ""
    ) => PostJsonAsync("/api/v1/providers/affiliate-links", new
    {
        provider_id = providerId,
        affiliate_id = affiliateId,
        status,
        channel,
        tracking_url = trackingUrl,
        contract_ref = contractRef,
    });

    private async Task<string> GetAsync(string path)
    {
        using var response = await _http.GetAsync(_baseUrl + path).ConfigureAwait(false);
        var body = await response.Content.ReadAsStringAsync().ConfigureAwait(false);
        if (!response.IsSuccessStatusCode)
        {
            throw new HttpRequestException($"status={(int)response.StatusCode} body={body}");
        }
        return body;
    }

    private async Task<string> PostJsonAsync(string path, object payload)
    {
        var body = JsonSerializer.Serialize(payload);
        return await PostRawJsonAsync(path, body).ConfigureAwait(false);
    }

    private async Task<string> PostRawJsonAsync(string path, string body)
    {
        using var content = new StringContent(body, Encoding.UTF8, "application/json");
        using var response = await _http.PostAsync(_baseUrl + path, content).ConfigureAwait(false);
        var responseBody = await response.Content.ReadAsStringAsync().ConfigureAwait(false);
        if (!response.IsSuccessStatusCode)
        {
            throw new HttpRequestException($"status={(int)response.StatusCode} body={responseBody}");
        }
        return responseBody;
    }

    public void Dispose()
    {
        _http.Dispose();
    }
}
