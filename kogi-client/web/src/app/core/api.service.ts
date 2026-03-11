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

  unifiedScreens(): Observable<unknown> {
    return this.http.get(`${this.baseUrl}/api/v1/screens/unified`);
  }
}
