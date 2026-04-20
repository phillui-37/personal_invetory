/// DLSite ecosystem fixture loader for testing without real API calls.
/// Provides fixture data that mimics real DLSite API responses.

use domain::DomainError;
use serde::Deserialize;
use std::fs;
use std::path::Path;

/// A DLSite work as returned from fixture data.
#[derive(Debug, Clone, Deserialize)]
pub struct DLSiteWork {
    pub workno: String,
    pub work_name: String,
    pub work_type: String,
    pub maker_name: Option<String>,
}

/// Load DLSite library fixture from a JSON file.
/// Returns a vector of DLSiteWork objects parsed from fixture data.
///
/// # Errors
/// Returns DomainError if:
/// - File cannot be read
/// - JSON is malformed
pub fn load_dlsite_fixture(path: impl AsRef<Path>) -> Result<Vec<DLSiteWork>, DomainError> {
    let content = fs::read_to_string(path)
        .map_err(|e| DomainError::InternalError(format!("Failed to read DLSite fixture: {e}")))?;

    serde_json::from_str(&content)
        .map_err(|e| DomainError::InternalError(format!("Failed to parse DLSite fixture JSON: {e}")))
}

/// Load DLSite library fixture from the default test fixtures directory.
pub fn load_dlsite_fixture_default() -> Result<Vec<DLSiteWork>, DomainError> {
    let possible_paths = vec![
        "backend/plugins/tests/fixtures/dlsite_purchases.json",
        "plugins/tests/fixtures/dlsite_purchases.json",
        "tests/fixtures/dlsite_purchases.json",
    ];

    for path in possible_paths {
        if Path::new(path).exists() {
            return load_dlsite_fixture(path);
        }
    }

    Err(DomainError::InternalError(
        "DLSite fixture file not found in any expected location".to_string(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dlsite_work_structure() {
        let json = r#"{
            "workno": "RJ001234",
            "work_name": "Test Work",
            "work_type": "GAM",
            "maker_name": "Test Maker"
        }"#;
        let work: DLSiteWork = serde_json::from_str(json).expect("deserialize");
        assert_eq!(work.workno, "RJ001234");
        assert_eq!(work.work_name, "Test Work");
        assert_eq!(work.work_type, "GAM");
    }

    #[test]
    fn test_nonexistent_fixture_error() {
        let result = load_dlsite_fixture("nonexistent/path/fixture.json");
        assert!(result.is_err(), "Should fail for missing fixture file");
    }

    #[test]
    fn test_malformed_json_error() {
        use std::io::Write;
        use tempfile::NamedTempFile;

        let mut temp = NamedTempFile::new().expect("create temp file");
        writeln!(temp, "{{invalid json}}").expect("write temp file");
        temp.flush().expect("flush temp file");

        let result = load_dlsite_fixture(temp.path());
        assert!(result.is_err(), "Should fail for malformed JSON");
    }
}
