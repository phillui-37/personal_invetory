/// FANZA ecosystem fixture loader for testing without real API calls.
/// Provides fixture data that mimics real FANZA API responses.

use domain::DomainError;
use serde::Deserialize;
use std::fs;
use std::path::Path;

/// A FANZA item as returned from fixture data.
#[derive(Debug, Clone, Deserialize)]
pub struct FanzaItem {
    pub product_id: String,
    pub title: String,
    pub category: String,
    pub content_type: String,
    pub purchase_date: String,
    pub thumbnail: Option<String>,
}

/// Load FANZA library fixture from a JSON file.
/// Returns a vector of FanzaItem objects parsed from fixture data.
///
/// # Errors
/// Returns DomainError if:
/// - File cannot be read
/// - JSON is malformed
pub fn load_fanza_fixture(path: impl AsRef<Path>) -> Result<Vec<FanzaItem>, DomainError> {
    let content = fs::read_to_string(path)
        .map_err(|e| DomainError::InternalError(format!("Failed to read FANZA fixture: {e}")))?;

    serde_json::from_str(&content)
        .map_err(|e| DomainError::InternalError(format!("Failed to parse FANZA fixture JSON: {e}")))
}

/// Load FANZA library fixture from the default test fixtures directory.
pub fn load_fanza_fixture_default() -> Result<Vec<FanzaItem>, DomainError> {
    let possible_paths = vec![
        "backend/plugins/tests/fixtures/fanza_library.json",
        "plugins/tests/fixtures/fanza_library.json",
        "tests/fixtures/fanza_library.json",
    ];

    for path in possible_paths {
        if Path::new(path).exists() {
            return load_fanza_fixture(path);
        }
    }

    Err(DomainError::InternalError(
        "FANZA fixture file not found in any expected location".to_string(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fanza_item_structure() {
        let json = r#"{
            "product_id": "26123456",
            "title": "Test Title",
            "category": "game",
            "content_type": "game",
            "purchase_date": "2024-01-15",
            "thumbnail": "http://example.com/thumb.jpg"
        }"#;
        let item: FanzaItem = serde_json::from_str(json).expect("deserialize");
        assert_eq!(item.product_id, "26123456");
        assert_eq!(item.title, "Test Title");
        assert_eq!(item.content_type, "game");
    }

    #[test]
    fn test_nonexistent_fixture_error() {
        let result = load_fanza_fixture("nonexistent/path/fixture.json");
        assert!(result.is_err(), "Should fail for missing fixture file");
    }

    #[test]
    fn test_malformed_json_error() {
        use std::io::Write;
        use tempfile::NamedTempFile;

        let mut temp = NamedTempFile::new().expect("create temp file");
        writeln!(temp, "{{invalid json}}").expect("write temp file");
        temp.flush().expect("flush temp file");

        let result = load_fanza_fixture(temp.path());
        assert!(result.is_err(), "Should fail for malformed JSON");
    }
}
