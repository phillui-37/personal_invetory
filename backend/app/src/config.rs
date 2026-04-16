use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppConfig {
    pub database_url: String,
    pub api_key: Option<String>,
    pub host: String,
    pub port: u16,
    pub plugins_config: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConfigError {
    MissingRequiredVar(&'static str),
    InvalidPort(String),
}

impl AppConfig {
    pub fn from_env() -> Result<Self, ConfigError> {
        let env_map: BTreeMap<String, String> = std::env::vars().collect();
        Self::from_map(&env_map)
    }

    fn from_map(map: &BTreeMap<String, String>) -> Result<Self, ConfigError> {
        let database_url = map
            .get("DATABASE_URL")
            .cloned()
            .ok_or(ConfigError::MissingRequiredVar("DATABASE_URL"))?;
        let api_key = map
            .get("API_KEY")
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty());
        let host = map
            .get("HOST")
            .cloned()
            .unwrap_or_else(|| String::from("0.0.0.0"));
        let port = map
            .get("PORT")
            .map(|value| {
                value
                    .parse::<u16>()
                    .map_err(|_| ConfigError::InvalidPort(value.clone()))
            })
            .transpose()?
            .unwrap_or(8080);
        let plugins_config = map
            .get("PLUGINS_CONFIG")
            .cloned()
            .unwrap_or_else(|| String::from("plugins.toml"));

        Ok(Self {
            database_url,
            api_key,
            host,
            port,
            plugins_config,
        })
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::{AppConfig, ConfigError};

    #[test]
    fn config_requires_database_url() {
        let map = BTreeMap::new();
        let err = AppConfig::from_map(&map).expect_err("DATABASE_URL should be required");
        assert_eq!(err, ConfigError::MissingRequiredVar("DATABASE_URL"));
    }

    #[test]
    fn config_applies_defaults_for_optional_values() {
        let map = BTreeMap::from([(
            String::from("DATABASE_URL"),
            String::from("sqlite://./inventory.db"),
        )]);

        let config = AppConfig::from_map(&map).expect("config should load with defaults");
        assert_eq!(config.database_url, "sqlite://./inventory.db");
        assert_eq!(config.api_key, None);
        assert_eq!(config.host, "0.0.0.0");
        assert_eq!(config.port, 8080);
        assert_eq!(config.plugins_config, "plugins.toml");
    }

    #[test]
    fn config_reads_explicit_values() {
        let map = BTreeMap::from([
            (
                String::from("DATABASE_URL"),
                String::from("postgres://localhost:5432/inventory"),
            ),
            (String::from("API_KEY"), String::from("secret-key")),
            (String::from("HOST"), String::from("127.0.0.1")),
            (String::from("PORT"), String::from("9090")),
            (
                String::from("PLUGINS_CONFIG"),
                String::from("custom-plugins.toml"),
            ),
        ]);

        let config = AppConfig::from_map(&map).expect("config should load explicit values");
        assert_eq!(config.database_url, "postgres://localhost:5432/inventory");
        assert_eq!(config.api_key.as_deref(), Some("secret-key"));
        assert_eq!(config.host, "127.0.0.1");
        assert_eq!(config.port, 9090);
        assert_eq!(config.plugins_config, "custom-plugins.toml");
    }

    #[test]
    fn config_treats_blank_api_key_as_missing() {
        let map = BTreeMap::from([
            (
                String::from("DATABASE_URL"),
                String::from("sqlite://./inventory.db"),
            ),
            (String::from("API_KEY"), String::from("   ")),
        ]);

        let config = AppConfig::from_map(&map).expect("config should load");
        assert_eq!(config.api_key, None);
    }

    #[test]
    fn config_rejects_invalid_port() {
        let map = BTreeMap::from([
            (
                String::from("DATABASE_URL"),
                String::from("sqlite://./inventory.db"),
            ),
            (String::from("PORT"), String::from("not-a-port")),
        ]);

        let err = AppConfig::from_map(&map).expect_err("invalid PORT should be rejected");
        assert_eq!(err, ConfigError::InvalidPort(String::from("not-a-port")));
    }
}
