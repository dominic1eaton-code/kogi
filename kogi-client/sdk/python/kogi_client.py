import json
from typing import Any, Dict, Optional, Union
from urllib import request, parse


class KogiClient:
    def __init__(self, base_url: str = "http://127.0.0.1:8080", timeout: int = 15) -> None:
        self.base_url = base_url.rstrip("/")
        self.timeout = timeout

    def _request_json(self, method: str, path: str, body: Optional[Dict[str, Any]] = None) -> Any:
        url = f"{self.base_url}{path}"
        data = None
        headers = {"Content-Type": "application/json"}
        if body is not None:
            data = json.dumps(body).encode("utf-8")
        req = request.Request(url, data=data, headers=headers, method=method)
        with request.urlopen(req, timeout=self.timeout) as resp:
            text = resp.read().decode("utf-8")
            if resp.status < 200 or resp.status >= 300:
                raise RuntimeError(f"status={resp.status} body={text}")
            return json.loads(text)

    def _request_text(self, method: str, path: str) -> str:
        url = f"{self.base_url}{path}"
        req = request.Request(url, method=method)
        with request.urlopen(req, timeout=self.timeout) as resp:
            text = resp.read().decode("utf-8")
            if resp.status < 200 or resp.status >= 300:
                raise RuntimeError(f"status={resp.status} body={text}")
            return text

    def health(self) -> Any:
        return self._request_json("GET", "/health")

    def system_summary(self) -> Any:
        return self._request_json("GET", "/api/v1/system")

    def host_summary(self) -> Any:
        return self._request_json("GET", "/api/v1/host")

    def host_components(self) -> Any:
        return self._request_json("GET", "/api/v1/host/components")

    def modules(self) -> Any:
        return self._request_json("GET", "/api/v1/modules")

    def engine_system(self) -> Any:
        return self._request_json("GET", "/api/v1/engine/system")

    def engine_runtime(self) -> Any:
        return self._request_json("GET", "/api/v1/engine/runtime")

    def engine_control(self, action: str) -> Any:
        return self._request_json("POST", "/api/v1/engine/control", {"action": action})

    def engine_ingest(self, payload: Union[Dict[str, Any], str]) -> Any:
        if isinstance(payload, str):
            body = json.loads(payload) if payload.strip().startswith("{") else {"payload": payload}
        else:
            body = payload
        return self._request_json("POST", "/api/v1/engine/ingest", body)

    def database_runtime(self) -> Any:
        return self._request_json("GET", "/api/v1/database/runtime")

    def database_query(self, sql: str) -> Any:
        return self._request_json("POST", "/api/v1/database/query", {"sql": sql})

    def messages(self, limit: int = 100, topic: Optional[str] = None) -> Any:
        params = {"limit": str(limit)}
        if topic:
            params["topic"] = topic
        path = "/api/v1/messages?" + parse.urlencode(params)
        return self._request_json("GET", path)

    def send_message(self, topic: str, payload: Any, source: str = "client", target: str = "") -> Any:
        return self._request_json(
            "POST",
            "/api/v1/messages",
            {"topic": topic, "payload": payload, "source": source, "target": target},
        )

    def identities(self) -> Any:
        return self._request_json("GET", "/api/v1/ims/identities")

    def profiles(self) -> Any:
        return self._request_json("GET", "/api/v1/ims/profiles")

    def autonomy_capabilities(self) -> Any:
        return self._request_json("GET", "/api/v1/autonomy/capabilities")

    def module_isolation(self) -> Any:
        return self._request_json("GET", "/api/v1/kernel/modules/isolation")

    def office_overview(self) -> Any:
        return self._request_json("GET", "/api/v1/office")

    def office_dashboard(self) -> Any:
        return self._request_json("GET", "/api/v1/office/dashboard")

    def office_portfolio(self) -> Any:
        return self._request_json("GET", "/api/v1/office/portfolio")

    def office_timeline(self) -> Any:
        return self._request_json("GET", "/api/v1/office/timeline")

    def office_workspace(self) -> Any:
        return self._request_json("GET", "/api/v1/office/workspace")

    def office_assistant(self) -> Any:
        return self._request_json("GET", "/api/v1/office/assistant")

    def providers_snapshot(self) -> Any:
        return self._request_json("GET", "/api/v1/providers")

    def providers_platforms(self) -> Any:
        return self._request_json("GET", "/api/v1/providers/platforms")

    def providers_list(self) -> Any:
        return self._request_json("GET", "/api/v1/providers/providers")

    def providers_resources(self) -> Any:
        return self._request_json("GET", "/api/v1/providers/resources")

    def providers_versions(self) -> Any:
        return self._request_json("GET", "/api/v1/providers/versions")

    def providers_metadata(self) -> Any:
        return self._request_json("GET", "/api/v1/providers/metadata")

    def providers_data_assets(self) -> Any:
        return self._request_json("GET", "/api/v1/providers/data")

    def providers_affiliates(self) -> Any:
        return self._request_json("GET", "/api/v1/providers/affiliates")

    def providers_affiliate_links(self) -> Any:
        return self._request_json("GET", "/api/v1/providers/affiliate-links")

    def office_ack_notification(self, notification_id: str) -> Any:
        return self._request_json(
            "POST",
            "/api/v1/office/dashboard/notifications/ack",
            {"notification_id": notification_id},
        )

    def office_create_portfolio_item(self, item_type: str, name: str, status: str) -> Any:
        return self._request_json(
            "POST",
            "/api/v1/office/portfolio/items",
            {"item_type": item_type, "name": name, "status": status},
        )

    def office_create_timeline_event(
        self, calendar_id: str, title: str, kind: str, scheduled_for: str
    ) -> Any:
        return self._request_json(
            "POST",
            "/api/v1/office/timeline/events",
            {
                "calendar_id": calendar_id,
                "title": title,
                "kind": kind,
                "scheduled_for": scheduled_for,
            },
        )

    def office_create_workspace_story(self, title: str, points: int) -> Any:
        return self._request_json(
            "POST",
            "/api/v1/office/workspace/stories",
            {"title": title, "points": points},
        )

    def office_subscribe_assistant(self, topic: str) -> Any:
        return self._request_json(
            "POST",
            "/api/v1/office/assistant/subscriptions",
            {"topic": topic},
        )

    def providers_register_platform(
        self,
        name: str,
        kind: str,
        category: str,
        status: str,
        home_url: str = "",
        docs_url: str = "",
        support_contact: str = "",
        tags: Optional[list[str]] = None,
    ) -> Any:
        return self._request_json(
            "POST",
            "/api/v1/providers/platforms",
            {
                "name": name,
                "kind": kind,
                "category": category,
                "status": status,
                "home_url": home_url,
                "docs_url": docs_url,
                "support_contact": support_contact,
                "tags": tags or [],
            },
        )

    def providers_register(
        self,
        name: str,
        platform_id: str,
        kind: str,
        status: str,
        owner: str = "",
        primary_contact: str = "",
        tags: Optional[list[str]] = None,
    ) -> Any:
        return self._request_json(
            "POST",
            "/api/v1/providers/providers",
            {
                "name": name,
                "platform_id": platform_id,
                "kind": kind,
                "status": status,
                "owner": owner,
                "primary_contact": primary_contact,
                "tags": tags or [],
            },
        )

    def providers_add_resource(
        self,
        provider_id: str,
        resource_type: str,
        name: str,
        status: str,
        environment: str = "",
        endpoint: str = "",
        credentials_ref: str = "",
    ) -> Any:
        return self._request_json(
            "POST",
            "/api/v1/providers/resources",
            {
                "provider_id": provider_id,
                "resource_type": resource_type,
                "name": name,
                "status": status,
                "environment": environment,
                "endpoint": endpoint,
                "credentials_ref": credentials_ref,
            },
        )

    def providers_add_version(
        self,
        provider_id: str,
        version: str,
        status: str,
        released_at: str = "",
        notes: str = "",
        compatibility: Optional[list[str]] = None,
    ) -> Any:
        return self._request_json(
            "POST",
            "/api/v1/providers/versions",
            {
                "provider_id": provider_id,
                "version": version,
                "status": status,
                "released_at": released_at,
                "notes": notes,
                "compatibility": compatibility or [],
            },
        )

    def providers_set_metadata(
        self, provider_id: str, key: str, value: str, scope: str = ""
    ) -> Any:
        return self._request_json(
            "POST",
            "/api/v1/providers/metadata",
            {"provider_id": provider_id, "key": key, "value": value, "scope": scope},
        )

    def providers_add_data_asset(
        self,
        provider_id: str,
        dataset: str,
        status: str,
        record_count: int = 0,
        storage: str = "",
        last_sync: str = "",
    ) -> Any:
        return self._request_json(
            "POST",
            "/api/v1/providers/data",
            {
                "provider_id": provider_id,
                "dataset": dataset,
                "status": status,
                "record_count": record_count,
                "storage": storage,
                "last_sync": last_sync,
            },
        )

    def providers_register_affiliate(
        self,
        name: str,
        kind: str,
        status: str,
        website: str = "",
        contact: str = "",
        tags: Optional[list[str]] = None,
    ) -> Any:
        return self._request_json(
            "POST",
            "/api/v1/providers/affiliates",
            {
                "name": name,
                "kind": kind,
                "status": status,
                "website": website,
                "contact": contact,
                "tags": tags or [],
            },
        )

    def providers_add_affiliate_link(
        self,
        provider_id: str,
        affiliate_id: str,
        status: str,
        channel: str = "",
        tracking_url: str = "",
        contract_ref: str = "",
    ) -> Any:
        return self._request_json(
            "POST",
            "/api/v1/providers/affiliate-links",
            {
                "provider_id": provider_id,
                "affiliate_id": affiliate_id,
                "status": status,
                "channel": channel,
                "tracking_url": tracking_url,
                "contract_ref": contract_ref,
            },
        )

    def unified_screens(self) -> Any:
        return self._request_json("GET", "/api/v1/screens/unified")

    def unified_screens_flat(self) -> str:
        return self._request_text("GET", "/api/v1/screens/unified/flat")
