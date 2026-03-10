#[derive(Clone, Debug)]
pub struct ResourceLimits {
    pub memory_limit_mb: u64,
    pub max_processes: u32,
    pub max_files: u32,
    pub max_resources: u32,
}

#[derive(Clone, Debug)]
pub struct ModuleRuntime {
    pub id: String,
    pub name: String,
    pub kind: String,
    pub version: String,
    pub entrypoint: String,
    pub status: String,
    pub language: String,
    pub network_manager: String,
    pub capabilities: Vec<String>,
    pub integrations: Vec<String>,
    pub limits: ResourceLimits,
    pub active: bool,
}

impl ModuleRuntime {
    pub fn from_manifest(input: &str) -> Result<Self, String> {
        let mut id = None;
        let mut name = None;
        let mut kind = None;
        let mut version = None;
        let mut entrypoint = None;
        let mut status = None;
        let mut language = None;
        let mut network_manager = None;
        let mut capabilities = None;
        let mut integrations = None;
        let mut memory_limit_mb = None;
        let mut max_processes = None;
        let mut max_files = None;
        let mut max_resources = None;

        for raw_line in input.lines() {
            let line = raw_line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            if let Some((key, value)) = line.split_once(':') {
                let key = key.trim();
                let value = value.trim().trim_matches('"').to_string();
                match key {
                    "id" => id = Some(value),
                    "name" => name = Some(value),
                    "kind" => kind = Some(value),
                    "version" => version = Some(value),
                    "entrypoint" => entrypoint = Some(value),
                    "status" => status = Some(value),
                    "language" => language = Some(value),
                    "network_manager" => network_manager = Some(value),
                    "capabilities" => capabilities = Some(value),
                    "integrations" => integrations = Some(value),
                    "memory_limit_mb" => memory_limit_mb = parse_u64(&value),
                    "max_processes" => max_processes = parse_u32(&value),
                    "max_files" => max_files = parse_u32(&value),
                    "max_resources" => max_resources = parse_u32(&value),
                    _ => {}
                }
            }
        }

        Ok(Self {
            id: id.ok_or_else(|| "missing id".to_string())?,
            name: name.ok_or_else(|| "missing name".to_string())?,
            kind: kind.ok_or_else(|| "missing kind".to_string())?,
            version: version.unwrap_or_else(|| "0.1.0".to_string()),
            entrypoint: entrypoint.unwrap_or_else(|| "service".to_string()),
            status: status.unwrap_or_else(|| "active".to_string()),
            language: language.unwrap_or_else(|| "unknown".to_string()),
            network_manager: network_manager.unwrap_or_else(|| "kogi-go-network".to_string()),
            capabilities: parse_csv(capabilities.as_deref().unwrap_or_default()),
            integrations: parse_csv(integrations.as_deref().unwrap_or_default()),
            limits: ResourceLimits {
                memory_limit_mb: memory_limit_mb.unwrap_or(512),
                max_processes: max_processes.unwrap_or(128),
                max_files: max_files.unwrap_or(5000),
                max_resources: max_resources.unwrap_or(10000),
            },
            active: false,
        })
    }
}

fn parse_u32(input: &str) -> Option<u32> {
    input.parse::<u32>().ok()
}

fn parse_u64(input: &str) -> Option<u64> {
    input.parse::<u64>().ok()
}

fn parse_csv(input: &str) -> Vec<String> {
    input
        .split(',')
        .map(str::trim)
        .filter(|x| !x.is_empty())
        .map(|x| x.to_string())
        .collect()
}
