class KogiClient {
  constructor(baseUrl = "http://127.0.0.1:8080") {
    this.baseUrl = baseUrl.replace(/\/+$/, "");
  }

  async health() {
    return this.getJson("/health");
  }

  async systemSummary() {
    return this.getJson("/api/v1/system");
  }

  async hostSummary() {
    return this.getJson("/api/v1/host");
  }

  async hostComponents() {
    return this.getJson("/api/v1/host/components");
  }

  async modules() {
    return this.getJson("/api/v1/modules");
  }

  async engineSystem() {
    return this.getJson("/api/v1/engine/system");
  }

  async engineRuntime() {
    return this.getJson("/api/v1/engine/runtime");
  }

  async engineControl(action) {
    return this.postJson("/api/v1/engine/control", { action });
  }

  async engineIngest(payload) {
    if (typeof payload === "string") {
      const trimmed = payload.trim();
      const body = trimmed.startsWith("{") ? JSON.parse(trimmed) : { payload };
      return this.postJson("/api/v1/engine/ingest", body);
    }
    return this.postJson("/api/v1/engine/ingest", payload);
  }

  async databaseRuntime() {
    return this.getJson("/api/v1/database/runtime");
  }

  async databaseQuery(sql) {
    return this.postJson("/api/v1/database/query", { sql });
  }

  async messages(limit = 100, topic = "") {
    const params = new URLSearchParams({ limit: String(limit) });
    if (topic) params.set("topic", topic);
    return this.getJson(`/api/v1/messages?${params.toString()}`);
  }

  async sendMessage(topic, payload, source = "client", target = "") {
    return this.postJson("/api/v1/messages", { topic, payload, source, target });
  }

  async identities() {
    return this.getJson("/api/v1/ims/identities");
  }

  async profiles() {
    return this.getJson("/api/v1/ims/profiles");
  }

  async autonomyCapabilities() {
    return this.getJson("/api/v1/autonomy/capabilities");
  }

  async moduleIsolation() {
    return this.getJson("/api/v1/kernel/modules/isolation");
  }

  async officeOverview() {
    return this.getJson("/api/v1/office");
  }

  async officeDashboard() {
    return this.getJson("/api/v1/office/dashboard");
  }

  async officePortfolio() {
    return this.getJson("/api/v1/office/portfolio");
  }

  async officeTimeline() {
    return this.getJson("/api/v1/office/timeline");
  }

  async officeWorkspace() {
    return this.getJson("/api/v1/office/workspace");
  }

  async officeAssistant() {
    return this.getJson("/api/v1/office/assistant");
  }

  async providersSnapshot() {
    return this.getJson("/api/v1/providers");
  }

  async providersPlatforms() {
    return this.getJson("/api/v1/providers/platforms");
  }

  async providersList() {
    return this.getJson("/api/v1/providers/providers");
  }

  async providersResources() {
    return this.getJson("/api/v1/providers/resources");
  }

  async providersVersions() {
    return this.getJson("/api/v1/providers/versions");
  }

  async providersMetadata() {
    return this.getJson("/api/v1/providers/metadata");
  }

  async providersDataAssets() {
    return this.getJson("/api/v1/providers/data");
  }

  async providersAffiliates() {
    return this.getJson("/api/v1/providers/affiliates");
  }

  async providersAffiliateLinks() {
    return this.getJson("/api/v1/providers/affiliate-links");
  }

  async officeAckNotification(notificationId) {
    return this.postJson("/api/v1/office/dashboard/notifications/ack", {
      notification_id: notificationId,
    });
  }

  async officeCreatePortfolioItem(itemType, name, status) {
    return this.postJson("/api/v1/office/portfolio/items", {
      item_type: itemType,
      name,
      status,
    });
  }

  async officeCreateTimelineEvent(calendarId, title, kind, scheduledFor) {
    return this.postJson("/api/v1/office/timeline/events", {
      calendar_id: calendarId,
      title,
      kind,
      scheduled_for: scheduledFor,
    });
  }

  async officeCreateWorkspaceStory(title, points) {
    return this.postJson("/api/v1/office/workspace/stories", {
      title,
      points,
    });
  }

  async officeSubscribeAssistant(topic) {
    return this.postJson("/api/v1/office/assistant/subscriptions", { topic });
  }

  async providersRegisterPlatform(
    name,
    kind,
    category,
    status,
    homeUrl = "",
    docsUrl = "",
    supportContact = "",
    tags = [],
  ) {
    return this.postJson("/api/v1/providers/platforms", {
      name,
      kind,
      category,
      status,
      home_url: homeUrl,
      docs_url: docsUrl,
      support_contact: supportContact,
      tags,
    });
  }

  async providersRegister(
    name,
    platformId,
    kind,
    status,
    owner = "",
    primaryContact = "",
    tags = [],
  ) {
    return this.postJson("/api/v1/providers/providers", {
      name,
      platform_id: platformId,
      kind,
      status,
      owner,
      primary_contact: primaryContact,
      tags,
    });
  }

  async providersAddResource(
    providerId,
    resourceType,
    name,
    status,
    environment = "",
    endpoint = "",
    credentialsRef = "",
  ) {
    return this.postJson("/api/v1/providers/resources", {
      provider_id: providerId,
      resource_type: resourceType,
      name,
      status,
      environment,
      endpoint,
      credentials_ref: credentialsRef,
    });
  }

  async providersAddVersion(
    providerId,
    version,
    status,
    releasedAt = "",
    notes = "",
    compatibility = [],
  ) {
    return this.postJson("/api/v1/providers/versions", {
      provider_id: providerId,
      version,
      status,
      released_at: releasedAt,
      notes,
      compatibility,
    });
  }

  async providersSetMetadata(providerId, key, value, scope = "") {
    return this.postJson("/api/v1/providers/metadata", {
      provider_id: providerId,
      key,
      value,
      scope,
    });
  }

  async providersAddDataAsset(
    providerId,
    dataset,
    status,
    recordCount = 0,
    storage = "",
    lastSync = "",
  ) {
    return this.postJson("/api/v1/providers/data", {
      provider_id: providerId,
      dataset,
      status,
      record_count: recordCount,
      storage,
      last_sync: lastSync,
    });
  }

  async providersRegisterAffiliate(
    name,
    kind,
    status,
    website = "",
    contact = "",
    tags = [],
  ) {
    return this.postJson("/api/v1/providers/affiliates", {
      name,
      kind,
      status,
      website,
      contact,
      tags,
    });
  }

  async providersAddAffiliateLink(
    providerId,
    affiliateId,
    status,
    channel = "",
    trackingUrl = "",
    contractRef = "",
  ) {
    return this.postJson("/api/v1/providers/affiliate-links", {
      provider_id: providerId,
      affiliate_id: affiliateId,
      status,
      channel,
      tracking_url: trackingUrl,
      contract_ref: contractRef,
    });
  }

  async unifiedScreens() {
    return this.getJson("/api/v1/screens/unified");
  }

  async unifiedScreensFlat() {
    return this.getText("/api/v1/screens/unified/flat");
  }

  async getJson(path) {
    const response = await fetch(this.baseUrl + path, {
      headers: { "Content-Type": "application/json" },
    });
    const text = await response.text();
    if (!response.ok) {
      throw new Error(`status=${response.status} body=${text}`);
    }
    return JSON.parse(text);
  }

  async postJson(path, body) {
    const response = await fetch(this.baseUrl + path, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(body),
    });
    const text = await response.text();
    if (!response.ok) {
      throw new Error(`status=${response.status} body=${text}`);
    }
    return JSON.parse(text);
  }

  async getText(path) {
    const response = await fetch(this.baseUrl + path, {
      headers: { "Content-Type": "application/json" },
    });
    const text = await response.text();
    if (!response.ok) {
      throw new Error(`status=${response.status} body=${text}`);
    }
    return text;
  }
}

module.exports = { KogiClient };
