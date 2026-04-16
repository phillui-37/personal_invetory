use std::collections::HashMap;

use domain::ResourceType;

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

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct PluginsConfig {
    pub use_noop_metadata_extractor: bool,
    pub use_noop_web_checker: bool,
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
        };

        let registry = PluginRegistry::from_config(&config);

        assert_eq!(registry.metadata_extractors.len(), 1);
        assert_eq!(registry.web_checkers.len(), 1);
    }
}
