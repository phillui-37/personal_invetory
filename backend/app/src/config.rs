use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppConfig {
    pub database_url: String,
    pub api_key: Option<String>,
    pub host: String,
    pub port: u16,
    pub plugins_config: String,
    pub chromium_path: Option<String>,
    pub fcm_service_account: Option<String>,
    pub scheduler_enabled: bool,
    pub device_id: String,
    pub device_name: Option<String>,
    pub otp_timeout_secs: Option<u64>,
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
        let chromium_path = map
            .get("CHROMIUM_PATH")
            .map(|v| v.trim().to_string())
            .filter(|v| !v.is_empty());
        let fcm_service_account = map
            .get("FCM_SERVICE_ACCOUNT_JSON")
            .map(|v| v.trim().to_string())
            .filter(|v| !v.is_empty());
        let scheduler_enabled = map
            .get("SCHEDULER_ENABLED")
            .map(|v| v.trim().to_lowercase())
            .map(|v| v != "false" && v != "0")
            .unwrap_or(true);
        let device_id = map
            .get("DEVICE_ID")
            .cloned()
            .ok_or(ConfigError::MissingRequiredVar("DEVICE_ID"))?;
        let device_name = map
            .get("DEVICE_NAME")
            .map(|v| v.trim().to_string())
            .filter(|v| !v.is_empty());

        let otp_timeout_secs = map
            .get("OTP_TIMEOUT_SECS")
            .and_then(|v| v.trim().parse::<u64>().ok());

        Ok(Self {
            database_url,
            api_key,
            host,
            port,
            plugins_config,
            chromium_path,
            fcm_service_account,
            scheduler_enabled,
            device_id,
            device_name,
            otp_timeout_secs,
        })
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::{AppConfig, ConfigError};

    fn base_map() -> BTreeMap<String, String> {
        BTreeMap::from([
            (
                String::from("DATABASE_URL"),
                String::from("sqlite://./inventory.db"),
            ),
            (String::from("DEVICE_ID"), String::from("test-device")),
        ])
    }

    #[test]
    fn config_requires_database_url() {
        let map = BTreeMap::new();
        let err = AppConfig::from_map(&map).expect_err("DATABASE_URL should be required");
        assert_eq!(err, ConfigError::MissingRequiredVar("DATABASE_URL"));
    }

    #[test]
    fn config_applies_defaults_for_optional_values() {
        let map = base_map();
        let config = AppConfig::from_map(&map).expect("config should load with defaults");
        assert_eq!(config.database_url, "sqlite://./inventory.db");
        assert_eq!(config.api_key, None);
        assert_eq!(config.host, "0.0.0.0");
        assert_eq!(config.port, 8080);
        assert_eq!(config.plugins_config, "plugins.toml");
        assert_eq!(config.chromium_path, None);
        assert_eq!(config.fcm_service_account, None);
        assert!(config.scheduler_enabled);
        assert_eq!(config.device_id, "test-device");
        assert_eq!(config.device_name, None);
        assert_eq!(config.otp_timeout_secs, None);
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
            (String::from("DEVICE_ID"), String::from("device-1")),
        ]);

        let config = AppConfig::from_map(&map).expect("config should load explicit values");
        assert_eq!(config.database_url, "postgres://localhost:5432/inventory");
        assert_eq!(config.api_key.as_deref(), Some("secret-key"));
        assert_eq!(config.host, "127.0.0.1");
        assert_eq!(config.port, 9090);
        assert_eq!(config.plugins_config, "custom-plugins.toml");
        assert_eq!(config.device_id, "device-1");
    }

    #[test]
    fn config_treats_blank_api_key_as_missing() {
        let mut map = base_map();
        map.insert(String::from("API_KEY"), String::from("   "));
        let config = AppConfig::from_map(&map).expect("config should load");
        assert_eq!(config.api_key, None);
    }

    #[test]
    fn config_rejects_invalid_port() {
        let mut map = base_map();
        map.insert(String::from("PORT"), String::from("not-a-port"));
        let err = AppConfig::from_map(&map).expect_err("invalid PORT should be rejected");
        assert_eq!(err, ConfigError::InvalidPort(String::from("not-a-port")));
    }

    #[test]
    fn config_reads_chromium_path_when_set() {
        let mut map = base_map();
        map.insert(
            String::from("CHROMIUM_PATH"),
            String::from("/usr/bin/chromium"),
        );
        let config = AppConfig::from_map(&map).expect("config should load");
        assert_eq!(config.chromium_path.as_deref(), Some("/usr/bin/chromium"));
    }

    #[test]
    fn config_treats_blank_chromium_path_as_none() {
        let mut map = base_map();
        map.insert(String::from("CHROMIUM_PATH"), String::from("  "));
        let config = AppConfig::from_map(&map).expect("config should load");
        assert_eq!(config.chromium_path, None);
    }

    #[test]
    fn config_reads_fcm_service_account_when_set() {
        let mut map = base_map();
        map.insert(
            String::from("FCM_SERVICE_ACCOUNT_JSON"),
            String::from(r#"{"type":"service_account"}"#),
        );
        let config = AppConfig::from_map(&map).expect("config should load");
        assert!(config.fcm_service_account.is_some());
    }

    #[test]
    fn config_treats_blank_fcm_service_account_as_none() {
        let mut map = base_map();
        map.insert(String::from("FCM_SERVICE_ACCOUNT_JSON"), String::from(" "));
        let config = AppConfig::from_map(&map).expect("config should load");
        assert_eq!(config.fcm_service_account, None);
    }

    #[test]
    fn scheduler_enabled_defaults_to_true() {
        let map = base_map();
        let config = AppConfig::from_map(&map).expect("config should load");
        assert!(config.scheduler_enabled);
    }

    #[test]
    fn scheduler_can_be_disabled_with_false() {
        let mut map = base_map();
        map.insert(String::from("SCHEDULER_ENABLED"), String::from("false"));
        let config = AppConfig::from_map(&map).expect("config should load");
        assert!(!config.scheduler_enabled);
    }

    #[test]
    fn scheduler_can_be_disabled_with_zero() {
        let mut map = base_map();
        map.insert(String::from("SCHEDULER_ENABLED"), String::from("0"));
        let config = AppConfig::from_map(&map).expect("config should load");
        assert!(!config.scheduler_enabled);
    }

    #[test]
    fn config_requires_device_id() {
        let map = BTreeMap::from([(
            String::from("DATABASE_URL"),
            String::from("sqlite://./inventory.db"),
        )]);
        let err = AppConfig::from_map(&map).expect_err("DEVICE_ID should be required");
        assert_eq!(err, ConfigError::MissingRequiredVar("DEVICE_ID"));
    }

    #[test]
    fn config_reads_device_id_and_optional_device_name() {
        let mut map = base_map();
        map.insert(String::from("DEVICE_ID"), String::from("desktop-home"));
        map.insert(String::from("DEVICE_NAME"), String::from("Home Desktop"));
        let config = AppConfig::from_map(&map).expect("config loads");
        assert_eq!(config.device_id, "desktop-home");
        assert_eq!(config.device_name.as_deref(), Some("Home Desktop"));
    }

    #[test]
    fn config_treats_blank_device_name_as_none() {
        let mut map = base_map();
        map.insert(String::from("DEVICE_ID"), String::from("desktop-home"));
        map.insert(String::from("DEVICE_NAME"), String::from("  "));
        let config = AppConfig::from_map(&map).expect("config loads");
        assert_eq!(config.device_name, None);
    }
}
