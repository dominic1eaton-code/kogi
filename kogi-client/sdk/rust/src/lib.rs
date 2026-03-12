use reqwest::blocking::Client;
use reqwest::Method;
use serde::Serialize;
use serde_json::Value;

#[derive(Debug)]
pub enum KogiError {
    Http(reqwest::Error),
    Status(u16, String),
    Json(serde_json::Error),
}

impl From<reqwest::Error> for KogiError {
    fn from(err: reqwest::Error) -> Self {
        KogiError::Http(err)
    }
}

impl From<serde_json::Error> for KogiError {
    fn from(err: serde_json::Error) -> Self {
        KogiError::Json(err)
    }
}

#[derive(Clone, Debug)]
pub struct KogiClient {
    base_url: String,
    client: Client,
}

impl KogiClient {
    pub fn new(base_url: &str) -> Self {
        Self {
            base_url: base_url.trim_end_matches('/').to_string(),
            client: Client::new(),
        }
    }

    pub fn default_local() -> Self {
        Self::new("http://127.0.0.1:8080")
    }

    fn request_json<T: Serialize>(&self, method: Method, path: &str, body: Option<&T>) -> Result<Value, KogiError> {
        let url = format!("{}{}", self.base_url, path);
        let mut req = self.client.request(method, &url);
        if let Some(payload) = body {
            req = req.json(payload);
        }
        let resp = req.send()?;
        let status = resp.status();
        let text = resp.text()?;
        if !status.is_success() {
            return Err(KogiError::Status(status.as_u16(), text));
        }
        Ok(serde_json::from_str(&text)?)
    }

    fn request_text<T: Serialize>(&self, method: Method, path: &str, body: Option<&T>) -> Result<String, KogiError> {
        let url = format!("{}{}", self.base_url, path);
        let mut req = self.client.request(method, &url);
        if let Some(payload) = body {
            req = req.json(payload);
        }
        let resp = req.send()?;
        let status = resp.status();
        let text = resp.text()?;
        if !status.is_success() {
            return Err(KogiError::Status(status.as_u16(), text));
        }
        Ok(text)
    }

    fn post_raw_json(&self, path: &str, body: &str) -> Result<Value, KogiError> {
        let url = format!("{}{}", self.base_url, path);
        let resp = self
            .client
            .request(Method::POST, &url)
            .header("Content-Type", "application/json")
            .body(body.to_string())
            .send()?;
        let status = resp.status();
        let text = resp.text()?;
        if !status.is_success() {
            return Err(KogiError::Status(status.as_u16(), text));
        }
        Ok(serde_json::from_str(&text)?)
    }

    pub fn health(&self) -> Result<Value, KogiError> {
        self.request_json(Method::GET, "/health", Option::<&()>::None)
    }

    pub fn system_summary(&self) -> Result<Value, KogiError> {
        self.request_json(Method::GET, "/api/v1/system", Option::<&()>::None)
    }

    pub fn host_summary(&self) -> Result<Value, KogiError> {
        self.request_json(Method::GET, "/api/v1/host", Option::<&()>::None)
    }

    pub fn host_components(&self) -> Result<Value, KogiError> {
        self.request_json(Method::GET, "/api/v1/host/components", Option::<&()>::None)
    }

    pub fn modules(&self) -> Result<Value, KogiError> {
        self.request_json(Method::GET, "/api/v1/modules", Option::<&()>::None)
    }

    pub fn engine_system(&self) -> Result<Value, KogiError> {
        self.request_json(Method::GET, "/api/v1/engine/system", Option::<&()>::None)
    }

    pub fn engine_runtime(&self) -> Result<Value, KogiError> {
        self.request_json(Method::GET, "/api/v1/engine/runtime", Option::<&()>::None)
    }

    pub fn engine_control(&self, action: &str) -> Result<Value, KogiError> {
        #[derive(Serialize)]
        struct Body<'a> { action: &'a str }
        self.request_json(Method::POST, "/api/v1/engine/control", Some(&Body { action }))
    }

    pub fn engine_ingest_value(&self, payload: &Value) -> Result<Value, KogiError> {
        self.request_json(Method::POST, "/api/v1/engine/ingest", Some(payload))
    }

    pub fn engine_ingest_raw(&self, payload: &str) -> Result<Value, KogiError> {
        self.post_raw_json("/api/v1/engine/ingest", payload)
    }

    pub fn database_runtime(&self) -> Result<Value, KogiError> {
        self.request_json(Method::GET, "/api/v1/database/runtime", Option::<&()>::None)
    }

    pub fn database_query(&self, sql: &str) -> Result<Value, KogiError> {
        #[derive(Serialize)]
        struct Body<'a> { sql: &'a str }
        self.request_json(Method::POST, "/api/v1/database/query", Some(&Body { sql }))
    }

    pub fn messages(&self, limit: usize, topic: Option<&str>) -> Result<Value, KogiError> {
        let mut path = format!("/api/v1/messages?limit={}", limit);
        if let Some(topic) = topic {
            path.push_str("&topic=");
            path.push_str(&urlencoding::encode(topic));
        }
        self.request_json(Method::GET, &path, Option::<&()>::None)
    }

    pub fn send_message(
        &self,
        topic: &str,
        payload: Value,
        source: &str,
        target: &str,
    ) -> Result<Value, KogiError> {
        #[derive(Serialize)]
        struct Body<'a> {
            topic: &'a str,
            payload: Value,
            source: &'a str,
            target: &'a str,
        }
        let body = Body {
            topic,
            payload,
            source,
            target,
        };
        self.request_json(Method::POST, "/api/v1/messages", Some(&body))
    }

    pub fn identities(&self) -> Result<Value, KogiError> {
        self.request_json(Method::GET, "/api/v1/ims/identities", Option::<&()>::None)
    }

    pub fn profiles(&self) -> Result<Value, KogiError> {
        self.request_json(Method::GET, "/api/v1/ims/profiles", Option::<&()>::None)
    }

    pub fn autonomy_capabilities(&self) -> Result<Value, KogiError> {
        self.request_json(Method::GET, "/api/v1/autonomy/capabilities", Option::<&()>::None)
    }

    pub fn module_isolation(&self) -> Result<Value, KogiError> {
        self.request_json(Method::GET, "/api/v1/kernel/modules/isolation", Option::<&()>::None)
    }

    pub fn office_overview(&self) -> Result<Value, KogiError> {
        self.request_json(Method::GET, "/api/v1/office", Option::<&()>::None)
    }

    pub fn office_dashboard(&self) -> Result<Value, KogiError> {
        self.request_json(Method::GET, "/api/v1/office/dashboard", Option::<&()>::None)
    }

    pub fn office_portfolio(&self) -> Result<Value, KogiError> {
        self.request_json(Method::GET, "/api/v1/office/portfolio", Option::<&()>::None)
    }

    pub fn office_timeline(&self) -> Result<Value, KogiError> {
        self.request_json(Method::GET, "/api/v1/office/timeline", Option::<&()>::None)
    }

    pub fn office_workspace(&self) -> Result<Value, KogiError> {
        self.request_json(Method::GET, "/api/v1/office/workspace", Option::<&()>::None)
    }

    pub fn office_assistant(&self) -> Result<Value, KogiError> {
        self.request_json(Method::GET, "/api/v1/office/assistant", Option::<&()>::None)
    }

    pub fn office_ack_notification(&self, notification_id: &str) -> Result<Value, KogiError> {
        #[derive(Serialize)]
        struct Body<'a> { notification_id: &'a str }
        self.request_json(
            Method::POST,
            "/api/v1/office/dashboard/notifications/ack",
            Some(&Body { notification_id }),
        )
    }

    pub fn office_create_portfolio_item(
        &self,
        item_type: &str,
        name: &str,
        status: &str,
    ) -> Result<Value, KogiError> {
        #[derive(Serialize)]
        struct Body<'a> {
            item_type: &'a str,
            name: &'a str,
            status: &'a str,
        }
        self.request_json(
            Method::POST,
            "/api/v1/office/portfolio/items",
            Some(&Body { item_type, name, status }),
        )
    }

    pub fn office_create_timeline_event(
        &self,
        calendar_id: &str,
        title: &str,
        kind: &str,
        scheduled_for: &str,
    ) -> Result<Value, KogiError> {
        #[derive(Serialize)]
        struct Body<'a> {
            calendar_id: &'a str,
            title: &'a str,
            kind: &'a str,
            scheduled_for: &'a str,
        }
        self.request_json(
            Method::POST,
            "/api/v1/office/timeline/events",
            Some(&Body {
                calendar_id,
                title,
                kind,
                scheduled_for,
            }),
        )
    }

    pub fn office_create_workspace_story(
        &self,
        title: &str,
        points: i32,
    ) -> Result<Value, KogiError> {
        #[derive(Serialize)]
        struct Body<'a> {
            title: &'a str,
            points: i32,
        }
        self.request_json(
            Method::POST,
            "/api/v1/office/workspace/stories",
            Some(&Body { title, points }),
        )
    }

    pub fn office_subscribe_assistant(&self, topic: &str) -> Result<Value, KogiError> {
        #[derive(Serialize)]
        struct Body<'a> { topic: &'a str }
        self.request_json(
            Method::POST,
            "/api/v1/office/assistant/subscriptions",
            Some(&Body { topic }),
        )
    }

    pub fn providers_snapshot(&self) -> Result<Value, KogiError> {
        self.request_json(Method::GET, "/api/v1/providers", Option::<&()>::None)
    }

    pub fn providers_platforms(&self) -> Result<Value, KogiError> {
        self.request_json(Method::GET, "/api/v1/providers/platforms", Option::<&()>::None)
    }

    pub fn providers_list(&self) -> Result<Value, KogiError> {
        self.request_json(Method::GET, "/api/v1/providers/providers", Option::<&()>::None)
    }

    pub fn providers_resources(&self) -> Result<Value, KogiError> {
        self.request_json(Method::GET, "/api/v1/providers/resources", Option::<&()>::None)
    }

    pub fn providers_versions(&self) -> Result<Value, KogiError> {
        self.request_json(Method::GET, "/api/v1/providers/versions", Option::<&()>::None)
    }

    pub fn providers_metadata(&self) -> Result<Value, KogiError> {
        self.request_json(Method::GET, "/api/v1/providers/metadata", Option::<&()>::None)
    }

    pub fn providers_data_assets(&self) -> Result<Value, KogiError> {
        self.request_json(Method::GET, "/api/v1/providers/data", Option::<&()>::None)
    }

    pub fn providers_affiliates(&self) -> Result<Value, KogiError> {
        self.request_json(Method::GET, "/api/v1/providers/affiliates", Option::<&()>::None)
    }

    pub fn providers_affiliate_links(&self) -> Result<Value, KogiError> {
        self.request_json(Method::GET, "/api/v1/providers/affiliate-links", Option::<&()>::None)
    }

    pub fn providers_register_platform(
        &self,
        name: &str,
        kind: &str,
        category: &str,
        status: &str,
        home_url: &str,
        docs_url: &str,
        support_contact: &str,
        tags: Vec<String>,
    ) -> Result<Value, KogiError> {
        #[derive(Serialize)]
        struct Body {
            name: String,
            kind: String,
            category: String,
            status: String,
            home_url: String,
            docs_url: String,
            support_contact: String,
            tags: Vec<String>,
        }
        let body = Body {
            name: name.to_string(),
            kind: kind.to_string(),
            category: category.to_string(),
            status: status.to_string(),
            home_url: home_url.to_string(),
            docs_url: docs_url.to_string(),
            support_contact: support_contact.to_string(),
            tags,
        };
        self.request_json(Method::POST, "/api/v1/providers/platforms", Some(&body))
    }

    pub fn providers_register(
        &self,
        name: &str,
        platform_id: &str,
        kind: &str,
        status: &str,
        owner: &str,
        primary_contact: &str,
        tags: Vec<String>,
    ) -> Result<Value, KogiError> {
        #[derive(Serialize)]
        struct Body {
            name: String,
            platform_id: String,
            kind: String,
            status: String,
            owner: String,
            primary_contact: String,
            tags: Vec<String>,
        }
        let body = Body {
            name: name.to_string(),
            platform_id: platform_id.to_string(),
            kind: kind.to_string(),
            status: status.to_string(),
            owner: owner.to_string(),
            primary_contact: primary_contact.to_string(),
            tags,
        };
        self.request_json(Method::POST, "/api/v1/providers/providers", Some(&body))
    }

    pub fn providers_add_resource(
        &self,
        provider_id: &str,
        resource_type: &str,
        name: &str,
        status: &str,
        environment: &str,
        endpoint: &str,
        credentials_ref: &str,
    ) -> Result<Value, KogiError> {
        #[derive(Serialize)]
        struct Body {
            provider_id: String,
            resource_type: String,
            name: String,
            status: String,
            environment: String,
            endpoint: String,
            credentials_ref: String,
        }
        let body = Body {
            provider_id: provider_id.to_string(),
            resource_type: resource_type.to_string(),
            name: name.to_string(),
            status: status.to_string(),
            environment: environment.to_string(),
            endpoint: endpoint.to_string(),
            credentials_ref: credentials_ref.to_string(),
        };
        self.request_json(Method::POST, "/api/v1/providers/resources", Some(&body))
    }

    pub fn providers_add_version(
        &self,
        provider_id: &str,
        version: &str,
        status: &str,
        released_at: &str,
        notes: &str,
        compatibility: Vec<String>,
    ) -> Result<Value, KogiError> {
        #[derive(Serialize)]
        struct Body {
            provider_id: String,
            version: String,
            status: String,
            released_at: String,
            notes: String,
            compatibility: Vec<String>,
        }
        let body = Body {
            provider_id: provider_id.to_string(),
            version: version.to_string(),
            status: status.to_string(),
            released_at: released_at.to_string(),
            notes: notes.to_string(),
            compatibility,
        };
        self.request_json(Method::POST, "/api/v1/providers/versions", Some(&body))
    }

    pub fn providers_set_metadata(
        &self,
        provider_id: &str,
        key: &str,
        value: &str,
        scope: &str,
    ) -> Result<Value, KogiError> {
        #[derive(Serialize)]
        struct Body {
            provider_id: String,
            key: String,
            value: String,
            scope: String,
        }
        let body = Body {
            provider_id: provider_id.to_string(),
            key: key.to_string(),
            value: value.to_string(),
            scope: scope.to_string(),
        };
        self.request_json(Method::POST, "/api/v1/providers/metadata", Some(&body))
    }

    pub fn providers_add_data_asset(
        &self,
        provider_id: &str,
        dataset: &str,
        status: &str,
        record_count: u64,
        storage: &str,
        last_sync: &str,
    ) -> Result<Value, KogiError> {
        #[derive(Serialize)]
        struct Body {
            provider_id: String,
            dataset: String,
            status: String,
            record_count: u64,
            storage: String,
            last_sync: String,
        }
        let body = Body {
            provider_id: provider_id.to_string(),
            dataset: dataset.to_string(),
            status: status.to_string(),
            record_count,
            storage: storage.to_string(),
            last_sync: last_sync.to_string(),
        };
        self.request_json(Method::POST, "/api/v1/providers/data", Some(&body))
    }

    pub fn providers_register_affiliate(
        &self,
        name: &str,
        kind: &str,
        status: &str,
        website: &str,
        contact: &str,
        tags: Vec<String>,
    ) -> Result<Value, KogiError> {
        #[derive(Serialize)]
        struct Body {
            name: String,
            kind: String,
            status: String,
            website: String,
            contact: String,
            tags: Vec<String>,
        }
        let body = Body {
            name: name.to_string(),
            kind: kind.to_string(),
            status: status.to_string(),
            website: website.to_string(),
            contact: contact.to_string(),
            tags,
        };
        self.request_json(Method::POST, "/api/v1/providers/affiliates", Some(&body))
    }

    pub fn providers_add_affiliate_link(
        &self,
        provider_id: &str,
        affiliate_id: &str,
        status: &str,
        channel: &str,
        tracking_url: &str,
        contract_ref: &str,
    ) -> Result<Value, KogiError> {
        #[derive(Serialize)]
        struct Body {
            provider_id: String,
            affiliate_id: String,
            status: String,
            channel: String,
            tracking_url: String,
            contract_ref: String,
        }
        let body = Body {
            provider_id: provider_id.to_string(),
            affiliate_id: affiliate_id.to_string(),
            status: status.to_string(),
            channel: channel.to_string(),
            tracking_url: tracking_url.to_string(),
            contract_ref: contract_ref.to_string(),
        };
        self.request_json(Method::POST, "/api/v1/providers/affiliate-links", Some(&body))
    }

    pub fn unified_screens(&self) -> Result<Value, KogiError> {
        self.request_json(Method::GET, "/api/v1/screens/unified", Option::<&()>::None)
    }

    pub fn unified_screens_flat(&self) -> Result<String, KogiError> {
        self.request_text(Method::GET, "/api/v1/screens/unified/flat", Option::<&()>::None)
    }
}
