use std::collections::HashMap;

use domain::ResourceType;
use serde::Deserialize;

pub fn plugins_ready() -> bool {
    domain::domain_ready()
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ExtractedMeta {
    pub title: Option<String>,
    pub author: Option<String>,
    pub extra: HashMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct CheckResult {
    pub has_new: bool,
    pub latest_chapter: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PluginError {
    IoError(String),
    ParseError(String),
    UnsupportedInput,
}

pub trait MetadataExtractor: Send + Sync {
    fn resource_type(&self) -> ResourceType;
    fn extract(&self, input: &str) -> Result<ExtractedMeta, PluginError>;
}

pub trait WebChecker: Send + Sync {
    fn check(&self, url: &str) -> Result<CheckResult, PluginError>;
}

#[cfg(feature = "stub-plugins")]
#[derive(Debug, Default, Clone, Copy)]
pub struct NoOpMetadataExtractor;

#[cfg(feature = "stub-plugins")]
impl MetadataExtractor for NoOpMetadataExtractor {
    fn resource_type(&self) -> ResourceType {
        ResourceType::Ebook
    }

    fn extract(&self, _input: &str) -> Result<ExtractedMeta, PluginError> {
        Ok(ExtractedMeta::default())
    }
}

#[cfg(feature = "stub-plugins")]
#[derive(Debug, Default, Clone, Copy)]
pub struct NoOpWebChecker;

#[cfg(feature = "stub-plugins")]
impl WebChecker for NoOpWebChecker {
    fn check(&self, _url: &str) -> Result<CheckResult, PluginError> {
        Ok(CheckResult::default())
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct PluginsConfig {
    pub use_noop_metadata_extractor: bool,
    pub use_noop_web_checker: bool,
    pub web_checker: WebCheckerConfig,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct SiteConfig {
    pub url_pattern: String,
    pub css_selector: Option<String>,
    pub xpath: Option<String>,
    pub text_regex: Option<String>,
    pub check_interval_secs: Option<u64>,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Default)]
pub struct WebCheckerConfig {
    pub default_interval_secs: u64,
    pub sites: Vec<SiteConfig>,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Default)]
pub struct PluginsToml {
    pub web_checker: Option<WebCheckerConfig>,
}

#[derive(Default)]
pub struct PluginRegistry {
    pub metadata_extractors: Vec<Box<dyn MetadataExtractor>>,
    pub web_checkers: Vec<Box<dyn WebChecker>>,
}

impl PluginRegistry {
    pub fn from_config(config: &PluginsConfig) -> Self {
        #[cfg(feature = "stub-plugins")]
        let mut registry = Self::default();
        #[cfg(not(feature = "stub-plugins"))]
        let registry = Self::default();

        #[cfg(feature = "stub-plugins")]
        {
            if config.use_noop_metadata_extractor {
                registry
                    .metadata_extractors
                    .push(Box::new(NoOpMetadataExtractor));
            }
            if config.use_noop_web_checker {
                registry.web_checkers.push(Box::new(NoOpWebChecker));
            }
        }

        #[cfg(not(feature = "stub-plugins"))]
        let _ = config;

        registry
    }
}

/// Returns the first SiteConfig whose url_pattern (treated as a regex) matches the given URL.
/// Returns None if no config matches.
pub fn match_site_config<'a>(url: &str, configs: &'a [SiteConfig]) -> Option<&'a SiteConfig> {
    for config in configs {
        if let Ok(regex) = regex::Regex::new(&config.url_pattern) {
            if regex.is_match(url) {
                return Some(config);
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registry_from_config_without_flags_stays_empty() {
        let registry = PluginRegistry::from_config(&PluginsConfig::default());

        assert_eq!(registry.metadata_extractors.len(), 0);
        assert_eq!(registry.web_checkers.len(), 0);
    }

    #[cfg(feature = "stub-plugins")]
    #[test]
    fn no_op_metadata_extractor_returns_empty_meta() {
        let extractor = NoOpMetadataExtractor;
        let extracted = extractor
            .extract("anything")
            .expect("expected no-op success");

        assert_eq!(extractor.resource_type(), domain::ResourceType::Ebook);
        assert_eq!(
            extracted,
            ExtractedMeta {
                title: None,
                author: None,
                extra: std::collections::HashMap::new(),
            }
        );
    }

    #[cfg(feature = "stub-plugins")]
    #[test]
    fn no_op_web_checker_returns_no_new_chapter() {
        let checker = NoOpWebChecker;
        let result = checker
            .check("https://example.com")
            .expect("expected no-op success");

        assert_eq!(
            result,
            CheckResult {
                has_new: false,
                latest_chapter: None,
            }
        );
    }

    #[cfg(feature = "stub-plugins")]
    #[test]
    fn registry_from_config_builds_stub_entries() {
        let config = PluginsConfig {
            use_noop_metadata_extractor: true,
            use_noop_web_checker: true,
            web_checker: WebCheckerConfig::default(),
        };

        let registry = PluginRegistry::from_config(&config);

        assert_eq!(registry.metadata_extractors.len(), 1);
        assert_eq!(registry.web_checkers.len(), 1);
    }

    // TOML parsing tests
    #[test]
    fn toml_parses_full_web_checker_config() {
        let toml_str = r#"
[web_checker]
default_interval_secs = 3600
[[web_checker.sites]]
url_pattern = "https://example\\.com/.*"
css_selector = ".chapter"
text_regex = "Chapter (\\d+)"
check_interval_secs = 1800
"#;
        let parsed: PluginsToml = toml::from_str(toml_str).expect("parse failed");

        assert!(parsed.web_checker.is_some());
        let web_checker = parsed.web_checker.unwrap();
        assert_eq!(web_checker.default_interval_secs, 3600);
        assert_eq!(web_checker.sites.len(), 1);
        let site = &web_checker.sites[0];
        assert_eq!(site.url_pattern, "https://example\\.com/.*");
        assert_eq!(site.css_selector, Some(".chapter".to_string()));
        assert_eq!(site.text_regex, Some("Chapter (\\d+)".to_string()));
        assert_eq!(site.check_interval_secs, Some(1800));
    }

    #[test]
    fn toml_parses_empty_config_to_defaults() {
        let toml_str = "";
        let parsed: PluginsToml = toml::from_str(toml_str).expect("parse failed");

        assert!(parsed.web_checker.is_none());
    }

    #[test]
    fn toml_parses_site_with_partial_fields() {
        let toml_str = r#"
[web_checker]
default_interval_secs = 600
[[web_checker.sites]]
url_pattern = "https://test\\.com"
"#;
        let parsed: PluginsToml = toml::from_str(toml_str).expect("parse failed");

        assert!(parsed.web_checker.is_some());
        let web_checker = parsed.web_checker.unwrap();
        assert_eq!(web_checker.sites.len(), 1);
        let site = &web_checker.sites[0];
        assert_eq!(site.url_pattern, "https://test\\.com");
        assert_eq!(site.css_selector, None);
        assert_eq!(site.xpath, None);
        assert_eq!(site.text_regex, None);
        assert_eq!(site.check_interval_secs, None);
    }

    // match_site_config tests
    #[test]
    fn match_site_config_returns_first_matching_site() {
        let configs = vec![
            SiteConfig {
                url_pattern: "https://a\\.com/.*".to_string(),
                css_selector: Some("sel_a".to_string()),
                xpath: None,
                text_regex: None,
                check_interval_secs: None,
            },
            SiteConfig {
                url_pattern: "https://b\\.com/.*".to_string(),
                css_selector: Some("sel_b".to_string()),
                xpath: None,
                text_regex: None,
                check_interval_secs: None,
            },
        ];

        let result = match_site_config("https://a.com/chapter1", &configs);
        assert!(result.is_some());
        assert_eq!(result.unwrap().css_selector, Some("sel_a".to_string()));
    }

    #[test]
    fn match_site_config_returns_none_when_no_match() {
        let configs = vec![
            SiteConfig {
                url_pattern: "https://a\\.com/.*".to_string(),
                css_selector: None,
                xpath: None,
                text_regex: None,
                check_interval_secs: None,
            },
            SiteConfig {
                url_pattern: "https://b\\.com/.*".to_string(),
                css_selector: None,
                xpath: None,
                text_regex: None,
                check_interval_secs: None,
            },
        ];

        let result = match_site_config("https://c.com/", &configs);
        assert!(result.is_none());
    }

    #[test]
    fn match_site_config_is_case_sensitive_for_url_pattern() {
        let configs = vec![SiteConfig {
            url_pattern: "https://example\\.com/.*".to_string(),
            css_selector: None,
            xpath: None,
            text_regex: None,
            check_interval_secs: None,
        }];

        let result = match_site_config("https://EXAMPLE.COM/", &configs);
        assert!(result.is_none());
    }
}
