/// Integration tests for Steam ecosystem fixture-based loading and parsing.
/// Uses fixture data (steam_library.json) instead of real API calls.

#[cfg(test)]
mod steam_fixture_tests {
    use std::fs;
    use std::path::PathBuf;

    // Reuse Steam types from plugin
    #[derive(Debug, Clone, serde::Deserialize)]
    struct SteamOwnedGame {
        pub appid: u64,
        pub name: String,
        pub playtime_forever: Option<u64>,
        pub img_logo_url: Option<String>,
        pub capsule_image: Option<String>,
    }

    #[derive(Debug, serde::Deserialize)]
    struct OwnedGamesResponse {
        response: OwnedGamesInner,
    }

    #[derive(Debug, serde::Deserialize)]
    struct OwnedGamesInner {
        game_count: Option<u64>,
        games: Option<Vec<SteamOwnedGame>>,
    }

    /// Load Steam library fixture from JSON file.
    fn load_steam_fixture() -> Result<Vec<SteamOwnedGame>, Box<dyn std::error::Error>> {
        let fixture_path = PathBuf::from("tests/fixtures/steam_library.json");
        let content = fs::read_to_string(&fixture_path)?;
        let response: OwnedGamesResponse = serde_json::from_str(&content)?;
        Ok(response.response.games.unwrap_or_default())
    }

    #[test]
    fn test_load_steam_fixture_succeeds() {
        let result = load_steam_fixture();
        assert!(result.is_ok(), "Failed to load steam fixture: {:?}", result);
        let games = result.unwrap();
        assert!(!games.is_empty(), "Fixture should contain games");
    }

    #[test]
    fn test_fixture_has_expected_game_count() {
        let games = load_steam_fixture().expect("load fixture");
        assert_eq!(games.len(), 5, "Expected 5 games in fixture");
    }

    #[test]
    fn test_fixture_contains_dota2() {
        let games = load_steam_fixture().expect("load fixture");
        let dota = games.iter().find(|g| g.appid == 570);
        assert!(dota.is_some(), "Fixture should contain Dota 2");
        let dota = dota.unwrap();
        assert_eq!(dota.name, "Dota 2");
        assert_eq!(dota.playtime_forever, Some(2440));
    }

    #[test]
    fn test_fixture_game_titles_extracted() {
        let games = load_steam_fixture().expect("load fixture");
        let titles: Vec<String> = games.iter().map(|g| g.name.clone()).collect();
        assert!(titles.contains(&"Team Fortress 2".to_string()));
        assert!(titles.contains(&"Counter-Strike 2".to_string()));
    }

    #[test]
    fn test_fixture_playtime_extraction() {
        let games = load_steam_fixture().expect("load fixture");
        let game = games.iter().find(|g| g.appid == 730).unwrap();
        assert_eq!(game.playtime_forever, Some(5234), "CS2 should have 5234 minutes playtime");
    }

    #[test]
    fn test_fixture_game_with_zero_playtime() {
        let games = load_steam_fixture().expect("load fixture");
        let unplayed = games.iter().find(|g| g.appid == 221380).unwrap();
        assert_eq!(unplayed.name, "Dark Souls II");
        assert_eq!(unplayed.playtime_forever, Some(0), "Unplayed game should have 0 playtime");
    }

    #[test]
    fn test_fixture_missing_file_error() {
        // This tests error handling when fixture is missing
        let fixture_path = PathBuf::from("tests/fixtures/nonexistent.json");
        let result = fs::read_to_string(&fixture_path);
        assert!(result.is_err(), "Should fail when fixture file is missing");
    }

    #[test]
    fn test_fixture_malformed_json_error() {
        // Test parsing error handling with invalid JSON
        let invalid_json = r#"{ "broken": json }"#;
        let result: Result<OwnedGamesResponse, _> = serde_json::from_str(invalid_json);
        assert!(result.is_err(), "Should fail to parse invalid JSON");
    }

    #[test]
    fn test_fixture_all_games_have_appids() {
        let games = load_steam_fixture().expect("load fixture");
        for game in &games {
            assert!(game.appid > 0, "Each game must have valid appid: {:?}", game);
        }
    }

    #[test]
    fn test_fixture_all_games_have_titles() {
        let games = load_steam_fixture().expect("load fixture");
        for game in &games {
            assert!(!game.name.is_empty(), "Each game must have a title: {:?}", game);
        }
    }
}
