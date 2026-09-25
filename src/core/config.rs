use std::{env, error::Error, time::Duration};

pub struct Config {
    pub server: ServerConfig,
    pub(crate) database: DatabaseConfig,
    pub(crate) storage: StorageConfig,
    pub(crate) openrouter: OpenRouterConfig,
    pub(crate) uploads: UploadsConfig,
    pub(crate) beans: BeansConfig,
}

pub struct ServerConfig {
    pub bind_address: String,
}

pub(crate) struct DatabaseConfig {
    pub(crate) url: String,
    pub(crate) pool_size: u32,
    pub(crate) busy_timeout: Duration,
}

pub(crate) struct StorageConfig {
    pub(crate) bucket: String,
    pub(crate) region: String,
    pub(crate) upload_ttl: Duration,
    pub(crate) read_ttl: Duration,
}

pub(crate) struct OpenRouterConfig {
    pub(crate) api_key: String,
    pub(crate) model: String,
    pub(crate) url: String,
    pub(crate) title: String,
    pub(crate) timeout: Duration,
    pub(crate) max_tokens: u32,
}

pub(crate) struct UploadsConfig {
    pub(crate) key_prefix: String,
    pub(crate) abandon_after: Duration,
    pub(crate) cleanup_interval: Duration,
}

pub(crate) struct BeansConfig {
    pub(crate) page_size: i64,
}

impl Config {
    pub fn from_env() -> Result<Self, Box<dyn Error>> {
        Ok(Self {
            server: ServerConfig {
                bind_address: "0.0.0.0:3000".to_owned(),
            },
            database: DatabaseConfig {
                url: required("DATABASE_URL")?,
                pool_size: 5,
                busy_timeout: Duration::from_secs(5),
            },
            storage: StorageConfig {
                bucket: required("S3_BUCKET")?,
                region: required("AWS_REGION")?,
                upload_ttl: Duration::from_secs(15 * 60),
                read_ttl: Duration::from_secs(60 * 60),
            },
            openrouter: OpenRouterConfig {
                api_key: required("OPENROUTER_API_KEY")?,
                model: required("OPENROUTER_MODEL")?,
                url: "https://openrouter.ai/api/v1/chat/completions".to_owned(),
                title: "Cophy".to_owned(),
                timeout: Duration::from_secs(60),
                max_tokens: 500,
            },
            uploads: UploadsConfig {
                key_prefix: "uploads".to_owned(),
                abandon_after: Duration::from_secs(24 * 60 * 60),
                cleanup_interval: Duration::from_secs(60 * 60),
            },
            beans: BeansConfig { page_size: 100 },
        })
    }
}

fn required(name: &str) -> Result<String, Box<dyn Error>> {
    let value = env::var(name).map_err(|_| format!("{name} is required"))?;
    if value.trim().is_empty() {
        return Err(format!("{name} is required").into());
    }
    Ok(value)
}
