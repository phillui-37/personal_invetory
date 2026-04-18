use async_trait::async_trait;
use crate::DomainError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiscoveredItem {
    pub external_id: String,
    pub title: String,
    pub platform: String,
    pub metadata: std::collections::HashMap<String, String>,
}

#[async_trait]
pub trait EcosystemConnector: Send + Sync {
    fn platform_name(&self) -> &str;

    async fn discover_items(&self, credentials: &[u8]) -> Result<Vec<DiscoveredItem>, DomainError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn discovered_item_holds_metadata() {
        let mut metadata = std::collections::HashMap::new();
        metadata.insert("playtime".to_string(), "100".to_string());

        let item = DiscoveredItem {
            external_id: "440".to_string(),
            title: "Team Fortress 2".to_string(),
            platform: "steam".to_string(),
            metadata,
        };

        assert_eq!(item.external_id, "440");
        assert_eq!(item.title, "Team Fortress 2");
        assert_eq!(item.platform, "steam");
        assert_eq!(item.metadata.get("playtime"), Some(&"100".to_string()));
    }

    #[test]
    fn discovered_item_equality() {
        let a = DiscoveredItem {
            external_id: "1".to_string(),
            title: "Game".to_string(),
            platform: "steam".to_string(),
            metadata: Default::default(),
        };
        let b = a.clone();
        assert_eq!(a, b);
    }
}
