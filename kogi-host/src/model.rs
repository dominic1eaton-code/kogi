use crate::executive::HostError;
use crate::host::HostSystem;

#[derive(Clone, Debug)]
pub struct HostMessage {
    pub topic: String,
    pub payload: String,
    pub source: String,
    pub target: String,
    pub received_at_ms: u64,
}

#[derive(Clone, Debug)]
pub struct HostMessageResult {
    pub topic: String,
    pub status: String,
    pub handled: bool,
    pub response: String,
    pub processed_at_ms: u64,
    pub error: Option<String>,
}

pub struct HostModel {
    system: HostSystem,
    message_count: u64,
    last_message: Option<HostMessage>,
}

impl HostModel {
    pub fn new() -> Result<Self, HostError> {
        Ok(Self {
            system: HostSystem::new(),
            message_count: 0,
            last_message: None,
        })
    }

    pub fn system(&self) -> &HostSystem {
        &self.system
    }

    pub fn system_mut(&mut self) -> &mut HostSystem {
        &mut self.system
    }

    pub fn init(&mut self) -> Result<(), HostError> {
        Ok(())
    }

    pub fn configure(&mut self) -> Result<(), HostError> {
        self.system.bootstrap()
    }

    pub fn run(&mut self) -> Result<(), HostError> {
        self.system.tick();
        Ok(())
    }

    pub fn shutdown(&mut self) -> Result<(), HostError> {
        Ok(())
    }

    pub fn message_count(&self) -> u64 {
        self.message_count
    }

    pub fn last_message(&self) -> Option<&HostMessage> {
        self.last_message.as_ref()
    }

    pub fn process_message(&mut self, message: HostMessage) -> HostMessageResult {
        let topic = message.topic.clone();
        self.message_count = self.message_count.saturating_add(1);
        self.last_message = Some(message.clone());

        let (handled, outcome) = match topic.as_str() {
            "engine.control.requested" | "engine.control" => (
                true,
                {
                let action = extract_string_field(&message.payload, "action")
                    .or_else(|| non_json_payload(&message.payload))
                    .unwrap_or_else(|| "start".to_string());
                self.system.engine_control(&action)
                }
            ),
            "engine.ingest" => (
                true,
                {
                let payload = if message.payload.trim().is_empty() {
                    "{}".to_string()
                } else {
                    message.payload.clone()
                };
                self.system.engine_ingest(&payload)
                }
            ),
            "database.query.executed" | "database.query" => (
                true,
                {
                let sql = extract_string_field(&message.payload, "sql")
                    .or_else(|| non_json_payload(&message.payload))
                    .unwrap_or_else(|| "select 1".to_string());
                self.system.database_query(&sql)
                }
            ),
            "host.status" => (true, Ok(self.system.summary_line())),
            _ => (false, Ok(format!(
                "{{\"status\":\"ignored\",\"topic\":\"{}\"}}",
                escape_json(&topic)
            ))),
        };

        match outcome {
            Ok(response) => HostMessageResult {
                topic,
                status: "ok".to_string(),
                handled,
                response,
                processed_at_ms: now_ms(),
                error: None,
            },
            Err(err) => HostMessageResult {
                topic,
                status: "error".to_string(),
                handled,
                response: "{}".to_string(),
                processed_at_ms: now_ms(),
                error: Some(err.to_string()),
            },
        }
    }
}

fn now_ms() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

fn non_json_payload(payload: &str) -> Option<String> {
    let trimmed = payload.trim();
    if trimmed.is_empty() || trimmed.starts_with('{') {
        None
    } else {
        Some(trimmed.to_string())
    }
}

fn extract_string_field(payload: &str, field: &str) -> Option<String> {
    let needle = format!("\"{}\"", field);
    let start = payload.find(&needle)?;
    let rest = &payload[start + needle.len()..];
    let colon = rest.find(':')?;
    let mut value = rest[colon + 1..].trim_start();
    if !value.starts_with('"') {
        return None;
    }
    value = &value[1..];
    let end = value.find('"')?;
    Some(value[..end].to_string())
}

fn escape_json(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}
