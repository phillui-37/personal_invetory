/// Steam ecosystem fixture loader for testing without real API calls.
/// Provides fixture data that mimics real Steam API responses.

use domain::DomainError;
use serde::Deserialize;
use std::fs;
use std::path::Path;

/// A Steam game as returned from fixture data.
/// Mirrors the structure used in steam.rs for consistency.
#[derive(Debug, Clone, Deserialize)]
pub struct SteamGame {
    pub appid: u64,
    pub name: String,
    pub playtime_forever: Option<u64>,
    #[serde(default)]
    pub img_logo_url: Option<String>,
    #[serde(default)]
    pub capsule_image: Option<String>,
}

#[derive(Debug, Deserialize)]
struct SteamResponse {
    response: SteamResponseInner,
}

#[derive(Debug, Deserialize)]
struct SteamResponseInner {
    #[allow(dead_code)]
    game_count: Option<u64>,
    games: Option<Vec<SteamGame>>,
}

/// Load Steam library fixture from a JSON file.
/// Returns a vector of SteamGame objects parsed from fixture data.
///
/// # Errors
/// Returns DomainError if:
/// - File cannot be read
/// - JSON is malformed
/// - Required fields are missing
pub fn load_steam_fixture(path: impl AsRef<Path>) -> Result<Vec<SteamGame>, DomainError> {
    let content = fs::read_to_string(path)
        .map_err(|e| DomainError::InternalError(format!("Failed to read Steam fixture: {e}")))?;

    let response: SteamResponse = serde_json::from_str(&content)
        .map_err(|e| DomainError::InternalError(format!("Failed to parse Steam fixture JSON: {e}")))?;

    Ok(response.response.games.unwrap_or_default())
}

/// Load Steam library fixture from the default test fixtures directory.
/// This function determines the correct path based on the manifest directory.
pub fn load_steam_fixture_default() -> Result<Vec<SteamGame>, DomainError> {
    // Try multiple common paths where the fixture might be
    let possible_paths = vec![
        "backend/plugins/tests/fixtures/steam_library.json",
        "plugins/tests/fixtures/steam_library.json",
        "tests/fixtures/steam_library.json",
    ];

    for path in possible_paths {
        if Path::new(path).exists() {
            return load_steam_fixture(path);
        }
    }

    Err(DomainError::InternalError(
        "Steam fixture file not found in any expected location".to_string(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_steam_game_structure() {
        // Test that SteamGame can be deserialized correctly
        let json = r#"{
            "appid": 570,
            "name": "Dota 2",
            "playtime_forever": 100,
            "img_logo_url": "abc123",
            "capsule_image": "def456"
        }"#;
        let game: SteamGame = serde_json::from_str(json).expect("deserialize");
        assert_eq!(game.appid, 570);
        assert_eq!(game.name, "Dota 2");
        assert_eq!(game.playtime_forever, Some(100));
    }

    #[test]
    fn test_nonexistent_fixture_error() {
        let result = load_steam_fixture("nonexistent/path/fixture.json");
        assert!(result.is_err(), "Should fail for missing fixture file");
    }

    #[test]
    fn test_malformed_json_error() {
        // Create a temporary file with invalid JSON for testing
        use std::io::Write;
        use tempfile::NamedTempFile;

        let mut temp = NamedTempFile::new().expect("create temp file");
        writeln!(temp, "{{invalid json}}").expect("write temp file");
        temp.flush().expect("flush temp file");

        let result = load_steam_fixture(temp.path());
        assert!(result.is_err(), "Should fail for malformed JSON");
    }
}
