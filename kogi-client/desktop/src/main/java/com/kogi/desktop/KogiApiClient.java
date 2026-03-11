package com.kogi.desktop;

import java.io.IOException;
import java.net.URI;
import java.net.http.HttpClient;
import java.net.http.HttpRequest;
import java.net.http.HttpResponse;
import java.nio.charset.StandardCharsets;

public final class KogiApiClient {
    private final HttpClient http;
    private final String baseUrl;

    public KogiApiClient(String baseUrl) {
        this.http = HttpClient.newHttpClient();
        this.baseUrl = baseUrl;
    }

    public String health() throws IOException, InterruptedException {
        return get("/health");
    }

    public String modules() throws IOException, InterruptedException {
        return get("/api/v1/modules");
    }

    public String systemSummary() throws IOException, InterruptedException {
        return get("/api/v1/system");
    }

    public String identities() throws IOException, InterruptedException {
        return get("/api/v1/ims/identities");
    }

    public String profiles() throws IOException, InterruptedException {
        return get("/api/v1/ims/profiles");
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
        return post("/api/v1/office/dashboard/notifications/ack",
            "{\"notification_id\":\"" + escape(notificationId) + "\"}");
    }

    public String officeCreatePortfolioItem(String itemType, String name, String status) throws IOException, InterruptedException {
        return post("/api/v1/office/portfolio/items",
            "{\"item_type\":\"" + escape(itemType) + "\",\"name\":\"" + escape(name) + "\",\"status\":\"" + escape(status) + "\"}");
    }

    public String officeCreateTimelineEvent(
        String calendarId,
        String title,
        String kind,
        String scheduledFor
    ) throws IOException, InterruptedException {
        return post("/api/v1/office/timeline/events",
            "{\"calendar_id\":\"" + escape(calendarId) + "\",\"title\":\"" + escape(title) + "\",\"kind\":\"" + escape(kind) + "\",\"scheduled_for\":\"" + escape(scheduledFor) + "\"}");
    }

    public String officeCreateWorkspaceStory(String title, int points) throws IOException, InterruptedException {
        return post("/api/v1/office/workspace/stories",
            "{\"title\":\"" + escape(title) + "\",\"points\":" + points + "}");
    }

    public String officeSubscribeAssistant(String topic) throws IOException, InterruptedException {
        return post("/api/v1/office/assistant/subscriptions",
            "{\"topic\":\"" + escape(topic) + "\"}");
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
        return http.send(request, HttpResponse.BodyHandlers.ofString()).body();
    }

    private String post(String path, String body) throws IOException, InterruptedException {
        HttpRequest request = HttpRequest.newBuilder()
            .uri(URI.create(baseUrl + path))
            .header("Content-Type", "application/json")
            .POST(HttpRequest.BodyPublishers.ofString(body, StandardCharsets.UTF_8))
            .build();
        return http.send(request, HttpResponse.BodyHandlers.ofString()).body();
    }

    private static String escape(String value) {
        return value.replace("\\", "\\\\").replace("\"", "\\\"");
    }
}
