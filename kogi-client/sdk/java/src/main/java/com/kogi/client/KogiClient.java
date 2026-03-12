package com.kogi.client;

import java.io.IOException;
import java.net.URI;
import java.net.URLEncoder;
import java.net.http.HttpClient;
import java.net.http.HttpRequest;
import java.net.http.HttpResponse;
import java.nio.charset.StandardCharsets;

public final class KogiClient {
    private final String baseUrl;
    private final HttpClient http;

    public KogiClient() {
        this("http://127.0.0.1:8080");
    }

    public KogiClient(String baseUrl) {
        this.baseUrl = trimRightSlash(baseUrl);
        this.http = HttpClient.newHttpClient();
    }

    public String health() throws IOException, InterruptedException {
        return get("/health");
    }

    public String systemSummary() throws IOException, InterruptedException {
        return get("/api/v1/system");
    }

    public String hostSummary() throws IOException, InterruptedException {
        return get("/api/v1/host");
    }

    public String hostComponents() throws IOException, InterruptedException {
        return get("/api/v1/host/components");
    }

    public String modules() throws IOException, InterruptedException {
        return get("/api/v1/modules");
    }

    public String engineSystem() throws IOException, InterruptedException {
        return get("/api/v1/engine/system");
    }

    public String engineRuntime() throws IOException, InterruptedException {
        return get("/api/v1/engine/runtime");
    }

    public String engineControl(String action) throws IOException, InterruptedException {
        String body = "{\"action\":\"" + escape(action) + "\"}";
        return postJson("/api/v1/engine/control", body);
    }

    public String engineIngest(String payload) throws IOException, InterruptedException {
        String body = payload.trim().startsWith("{")
            ? payload
            : "{\"payload\":\"" + escape(payload) + "\"}";
        return postJson("/api/v1/engine/ingest", body);
    }

    public String databaseRuntime() throws IOException, InterruptedException {
        return get("/api/v1/database/runtime");
    }

    public String databaseQuery(String sql) throws IOException, InterruptedException {
        String body = "{\"sql\":\"" + escape(sql) + "\"}";
        return postJson("/api/v1/database/query", body);
    }

    public String messages(int limit, String topic) throws IOException, InterruptedException {
        String path = "/api/v1/messages?limit=" + limit;
        if (topic != null && !topic.isEmpty()) {
            path = path + "&topic=" + urlEncode(topic);
        }
        return get(path);
    }

    public String sendMessage(String topic, String payload, String source, String target)
            throws IOException, InterruptedException {
        String jsonPayload = payload.trim().startsWith("{")
            ? payload
            : "\"" + escape(payload) + "\"";
        String body = "{"
            + "\"topic\":\"" + escape(topic) + "\","
            + "\"payload\":" + jsonPayload + ","
            + "\"source\":\"" + escape(source) + "\","
            + "\"target\":\"" + escape(target) + "\""
            + "}";
        return postJson("/api/v1/messages", body);
    }

    public String identities() throws IOException, InterruptedException {
        return get("/api/v1/ims/identities");
    }

    public String profiles() throws IOException, InterruptedException {
        return get("/api/v1/ims/profiles");
    }

    public String autonomyCapabilities() throws IOException, InterruptedException {
        return get("/api/v1/autonomy/capabilities");
    }

    public String moduleIsolation() throws IOException, InterruptedException {
        return get("/api/v1/kernel/modules/isolation");
    }

    public String officeOverview() throws IOException, InterruptedException {
        return get("/api/v1/office");
    }

    public String officeDashboard() throws IOException, InterruptedException {
        return get("/api/v1/office/dashboard");
    }

    public String officePortfolio() throws IOException, InterruptedException {
        return get("/api/v1/office/portfolio");
    }

    public String officeTimeline() throws IOException, InterruptedException {
        return get("/api/v1/office/timeline");
    }

    public String officeWorkspace() throws IOException, InterruptedException {
        return get("/api/v1/office/workspace");
    }

    public String officeAssistant() throws IOException, InterruptedException {
        return get("/api/v1/office/assistant");
    }

    public String officeAckNotification(String notificationId) throws IOException, InterruptedException {
        String body = "{\"notification_id\":\"" + escape(notificationId) + "\"}";
        return postJson("/api/v1/office/dashboard/notifications/ack", body);
    }

    public String officeCreatePortfolioItem(String itemType, String name, String status)
            throws IOException, InterruptedException {
        String body = "{"
            + "\"item_type\":\"" + escape(itemType) + "\","
            + "\"name\":\"" + escape(name) + "\","
            + "\"status\":\"" + escape(status) + "\""
            + "}";
        return postJson("/api/v1/office/portfolio/items", body);
    }

    public String officeCreateTimelineEvent(
            String calendarId, String title, String kind, String scheduledFor)
            throws IOException, InterruptedException {
        String body = "{"
            + "\"calendar_id\":\"" + escape(calendarId) + "\","
            + "\"title\":\"" + escape(title) + "\","
            + "\"kind\":\"" + escape(kind) + "\","
            + "\"scheduled_for\":\"" + escape(scheduledFor) + "\""
            + "}";
        return postJson("/api/v1/office/timeline/events", body);
    }

    public String officeCreateWorkspaceStory(String title, int points)
            throws IOException, InterruptedException {
        String body = "{"
            + "\"title\":\"" + escape(title) + "\","
            + "\"points\":" + points
            + "}";
        return postJson("/api/v1/office/workspace/stories", body);
    }

    public String officeSubscribeAssistant(String topic) throws IOException, InterruptedException {
        String body = "{\"topic\":\"" + escape(topic) + "\"}";
        return postJson("/api/v1/office/assistant/subscriptions", body);
    }

    public String unifiedScreens() throws IOException, InterruptedException {
        return get("/api/v1/screens/unified");
    }

    public String unifiedScreensFlat() throws IOException, InterruptedException {
        return get("/api/v1/screens/unified/flat");
    }

    private String get(String path) throws IOException, InterruptedException {
        HttpRequest request = HttpRequest.newBuilder()
            .uri(URI.create(baseUrl + path))
            .GET()
            .build();
        return send(request);
    }

    private String postJson(String path, String jsonBody) throws IOException, InterruptedException {
        HttpRequest request = HttpRequest.newBuilder()
            .uri(URI.create(baseUrl + path))
            .header("Content-Type", "application/json")
            .POST(HttpRequest.BodyPublishers.ofString(jsonBody))
            .build();
        return send(request);
    }

    private String send(HttpRequest request) throws IOException, InterruptedException {
        HttpResponse<String> response = http.send(request, HttpResponse.BodyHandlers.ofString());
        if (response.statusCode() < 200 || response.statusCode() >= 300) {
            throw new IOException("status=" + response.statusCode() + " body=" + response.body());
        }
        return response.body();
    }

    private static String escape(String value) {
        return value
            .replace("\\", "\\\\")
            .replace("\"", "\\\"")
            .replace("\n", "\\n")
            .replace("\r", "\\r")
            .replace("\t", "\\t");
    }

    private static String urlEncode(String value) {
        return URLEncoder.encode(value, StandardCharsets.UTF_8);
    }

    private static String trimRightSlash(String value) {
        int end = value.length();
        while (end > 0 && value.charAt(end - 1) == '/') {
            end--;
        }
        return value.substring(0, end);
    }
}
