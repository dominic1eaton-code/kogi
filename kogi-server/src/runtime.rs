use std::env;

const MAX_PAYLOAD_LEN: usize = 256;

#[derive(Clone, Debug)]
pub struct Runtime {
    service: String,
    debug: bool,
    silent: bool,
}

impl Runtime {
    pub fn new(service: &str, debug: bool, silent: bool) -> Self {
        Self {
            service: service.to_string(),
            debug,
            silent,
        }
    }

    pub fn debug_enabled(&self) -> bool {
        self.debug && !self.silent
    }

    pub fn info(&self, msg: &str) {
        if self.silent {
            return;
        }
        println!("[{}] {}", self.service, msg);
    }

    pub fn error(&self, msg: &str) {
        if self.silent {
            return;
        }
        eprintln!("[{}] {}", self.service, msg);
    }

    pub fn debug(&self, msg: &str) {
        if !self.debug_enabled() {
            return;
        }
        println!("[{}] {}", self.service, msg);
    }

    pub fn state(&self, state: &str) {
        self.debug(&format!("state={state}"));
    }

    pub fn status(&self, status: &str, note: &str) {
        if note.is_empty() {
            self.debug(&format!("status={status}"));
        } else {
            self.debug(&format!("status={status} note={note}"));
        }
    }

    pub fn message(&self, direction: &str, topic: &str, source: &str, target: &str, payload: &str) {
        self.debug(&format!(
            "message {direction} topic={} source={} target={} payload={}",
            empty_if_unknown(topic),
            empty_if_unknown(source),
            empty_if_unknown(target),
            truncate(payload)
        ));
    }
}

pub fn env_bool(key: &str) -> bool {
    match env::var(key) {
        Ok(value) => {
            let v = value.trim().to_lowercase();
            matches!(v.as_str(), "1" | "true" | "yes" | "on")
        }
        Err(_) => false,
    }
}

fn truncate(value: &str) -> String {
    if value.len() <= MAX_PAYLOAD_LEN {
        return value.to_string();
    }
    format!("{}...", &value[..MAX_PAYLOAD_LEN])
}

fn empty_if_unknown(value: &str) -> &str {
    if value.is_empty() {
        "-"
    } else {
        value
    }
}
