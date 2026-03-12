import { Injectable } from '@angular/core';
import { HttpClient } from '@angular/common/http';
import { Observable } from 'rxjs';

@Injectable()
export class ApiService {
  private readonly baseUrl = 'http://127.0.0.1:8080';

  constructor(private readonly http: HttpClient) {}

  systemSummary(): Observable<unknown> {
    return this.http.get(`${this.baseUrl}/api/v1/system`);
  }

  hostSummary(): Observable<unknown> {
    return this.http.get(`${this.baseUrl}/api/v1/host`);
  }

  hostComponents(): Observable<unknown> {
    return this.http.get(`${this.baseUrl}/api/v1/host/components`);
  }

  modulesList(): Observable<unknown> {
    return this.http.get(`${this.baseUrl}/api/v1/modules`);
  }

  engineOverview(): Observable<unknown> {
    return this.http.get(`${this.baseUrl}/api/v1/engine/system`);
  }

  engineRuntime(): Observable<unknown> {
    return this.http.get(`${this.baseUrl}/api/v1/engine/runtime`);
  }

  engineControl(action: string): Observable<unknown> {
    return this.http.post(`${this.baseUrl}/api/v1/engine/control`, {
      action,
    });
  }

  engineIngest(payload: unknown): Observable<unknown> {
    return this.http.post(`${this.baseUrl}/api/v1/engine/ingest`, payload ?? {});
  }

  databaseRuntime(): Observable<unknown> {
    return this.http.get(`${this.baseUrl}/api/v1/database/runtime`);
  }

  databaseQuery(sql: string): Observable<unknown> {
    return this.http.post(`${this.baseUrl}/api/v1/database/query`, {
      sql,
    });
  }

  autonomyCapabilities(): Observable<unknown> {
    return this.http.get(`${this.baseUrl}/api/v1/autonomy/capabilities`);
  }

  identities(): Observable<unknown> {
    return this.http.get(`${this.baseUrl}/api/v1/ims/identities`);
  }

  profiles(): Observable<unknown> {
    return this.http.get(`${this.baseUrl}/api/v1/ims/profiles`);
  }

  moduleIsolation(): Observable<unknown> {
    return this.http.get(`${this.baseUrl}/api/v1/kernel/modules/isolation`);
  }

  officeOverview(): Observable<unknown> {
    return this.http.get(`${this.baseUrl}/api/v1/office`);
  }

  officeDashboard(): Observable<unknown> {
    return this.http.get(`${this.baseUrl}/api/v1/office/dashboard`);
  }

  officePortfolio(): Observable<unknown> {
    return this.http.get(`${this.baseUrl}/api/v1/office/portfolio`);
  }

  officeTimeline(): Observable<unknown> {
    return this.http.get(`${this.baseUrl}/api/v1/office/timeline`);
  }

  officeWorkspace(): Observable<unknown> {
    return this.http.get(`${this.baseUrl}/api/v1/office/workspace`);
  }

  officeAssistant(): Observable<unknown> {
    return this.http.get(`${this.baseUrl}/api/v1/office/assistant`);
  }

  providersSnapshot(): Observable<unknown> {
    return this.http.get(`${this.baseUrl}/api/v1/providers`);
  }

  providersPlatforms(): Observable<unknown> {
    return this.http.get(`${this.baseUrl}/api/v1/providers/platforms`);
  }

  providersList(): Observable<unknown> {
    return this.http.get(`${this.baseUrl}/api/v1/providers/providers`);
  }

  providersResources(): Observable<unknown> {
    return this.http.get(`${this.baseUrl}/api/v1/providers/resources`);
  }

  providersVersions(): Observable<unknown> {
    return this.http.get(`${this.baseUrl}/api/v1/providers/versions`);
  }

  providersMetadata(): Observable<unknown> {
    return this.http.get(`${this.baseUrl}/api/v1/providers/metadata`);
  }

  providersDataAssets(): Observable<unknown> {
    return this.http.get(`${this.baseUrl}/api/v1/providers/data`);
  }

  providersAffiliates(): Observable<unknown> {
    return this.http.get(`${this.baseUrl}/api/v1/providers/affiliates`);
  }

  providersAffiliateLinks(): Observable<unknown> {
    return this.http.get(`${this.baseUrl}/api/v1/providers/affiliate-links`);
  }

  officeAckNotification(notificationId: string): Observable<unknown> {
    return this.http.post(`${this.baseUrl}/api/v1/office/dashboard/notifications/ack`, {
      notification_id: notificationId,
    });
  }

  officeCreatePortfolioItem(itemType: string, name: string, status: string): Observable<unknown> {
    return this.http.post(`${this.baseUrl}/api/v1/office/portfolio/items`, {
      item_type: itemType,
      name,
      status,
    });
  }

  officeCreateTimelineEvent(
    calendarId: string,
    title: string,
    kind: string,
    scheduledFor: string,
  ): Observable<unknown> {
    return this.http.post(`${this.baseUrl}/api/v1/office/timeline/events`, {
      calendar_id: calendarId,
      title,
      kind,
      scheduled_for: scheduledFor,
    });
  }

  officeCreateWorkspaceStory(title: string, points: number): Observable<unknown> {
    return this.http.post(`${this.baseUrl}/api/v1/office/workspace/stories`, {
      title,
      points,
    });
  }

  officeSubscribeAssistant(topic: string): Observable<unknown> {
    return this.http.post(`${this.baseUrl}/api/v1/office/assistant/subscriptions`, {
      topic,
    });
  }

  providersCreatePlatform(
    name: string,
    kind: string,
    category: string,
    status: string,
    homeUrl?: string,
    docsUrl?: string,
    supportContact?: string,
    tags: string[] = [],
  ): Observable<unknown> {
    return this.http.post(`${this.baseUrl}/api/v1/providers/platforms`, {
      name,
      kind,
      category,
      status,
      home_url: homeUrl ?? '',
      docs_url: docsUrl ?? '',
      support_contact: supportContact ?? '',
      tags,
    });
  }

  providersCreateProvider(
    name: string,
    platformId: string,
    kind: string,
    status: string,
    owner?: string,
    primaryContact?: string,
    tags: string[] = [],
  ): Observable<unknown> {
    return this.http.post(`${this.baseUrl}/api/v1/providers/providers`, {
      name,
      platform_id: platformId,
      kind,
      status,
      owner: owner ?? '',
      primary_contact: primaryContact ?? '',
      tags,
    });
  }

  providersAddResource(
    providerId: string,
    resourceType: string,
    name: string,
    status: string,
    environment?: string,
    endpoint?: string,
    credentialsRef?: string,
  ): Observable<unknown> {
    return this.http.post(`${this.baseUrl}/api/v1/providers/resources`, {
      provider_id: providerId,
      resource_type: resourceType,
      name,
      status,
      environment: environment ?? '',
      endpoint: endpoint ?? '',
      credentials_ref: credentialsRef ?? '',
    });
  }

  providersAddVersion(
    providerId: string,
    version: string,
    status: string,
    releasedAt?: string,
    notes?: string,
    compatibility: string[] = [],
  ): Observable<unknown> {
    return this.http.post(`${this.baseUrl}/api/v1/providers/versions`, {
      provider_id: providerId,
      version,
      status,
      released_at: releasedAt ?? '',
      notes: notes ?? '',
      compatibility,
    });
  }

  providersSetMetadata(
    providerId: string,
    key: string,
    value: string,
    scope?: string,
  ): Observable<unknown> {
    return this.http.post(`${this.baseUrl}/api/v1/providers/metadata`, {
      provider_id: providerId,
      key,
      value,
      scope: scope ?? '',
    });
  }

  providersAddDataAsset(
    providerId: string,
    dataset: string,
    status: string,
    recordCount?: number,
    storage?: string,
    lastSync?: string,
  ): Observable<unknown> {
    return this.http.post(`${this.baseUrl}/api/v1/providers/data`, {
      provider_id: providerId,
      dataset,
      status,
      record_count: recordCount ?? 0,
      storage: storage ?? '',
      last_sync: lastSync ?? '',
    });
  }

  providersRegisterAffiliate(
    name: string,
    kind: string,
    status: string,
    website?: string,
    contact?: string,
    tags: string[] = [],
  ): Observable<unknown> {
    return this.http.post(`${this.baseUrl}/api/v1/providers/affiliates`, {
      name,
      kind,
      status,
      website: website ?? '',
      contact: contact ?? '',
      tags,
    });
  }

  providersAddAffiliateLink(
    providerId: string,
    affiliateId: string,
    status: string,
    channel?: string,
    trackingUrl?: string,
    contractRef?: string,
  ): Observable<unknown> {
    return this.http.post(`${this.baseUrl}/api/v1/providers/affiliate-links`, {
      provider_id: providerId,
      affiliate_id: affiliateId,
      status,
      channel: channel ?? '',
      tracking_url: trackingUrl ?? '',
      contract_ref: contractRef ?? '',
    });
  }

  unifiedScreens(): Observable<unknown> {
    return this.http.get(`${this.baseUrl}/api/v1/screens/unified`);
  }
}
