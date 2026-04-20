#[cfg(feature = "real-plugins")]
use domain::DomainError;
use serde::Deserialize;

#[cfg(any(feature = "real-plugins", test))]
const STEAM_API_BASE: &str = "https://api.steampowered.com";

#[derive(Debug, Clone, Deserialize)]
pub struct SteamOwnedGame {
    pub appid: u64,
    pub name: String,
    pub playtime_forever: Option<u64>,
}

#[derive(Debug, Deserialize)]
pub struct SteamCredentials {
    pub api_key: String,
    pub steam_id: String,
}

#[cfg(any(feature = "real-plugins", test))]
#[derive(Debug, Deserialize)]
struct OwnedGamesResponse {
    response: OwnedGamesInner,
}

#[cfg(any(feature = "real-plugins", test))]
#[derive(Debug, Deserialize)]
struct OwnedGamesInner {
    #[allow(dead_code)]
    game_count: Option<u64>,
    games: Option<Vec<SteamOwnedGame>>,
}

#[cfg(any(feature = "real-plugins", test))]
pub fn build_steam_api_url(api_key: &str, steam_id: &str) -> String {
    format!(
        "{STEAM_API_BASE}/IPlayerService/GetOwnedGames/v0001/?key={api_key}&steamid={steam_id}&include_appinfo=true&include_played_free_games=true&format=json"
    )
}

#[cfg(feature = "real-plugins")]
pub async fn fetch_owned_games(
    api_key: &str,
    steam_id: &str,
) -> Result<Vec<SteamOwnedGame>, DomainError> {
    let url = reqwest::Url::parse_with_params(
        &format!("{STEAM_API_BASE}/IPlayerService/GetOwnedGames/v0001/"),
        &[
            ("key", api_key),
            ("steamid", steam_id),
            ("include_appinfo", "true"),
            ("include_played_free_games", "true"),
            ("format", "json"),
        ],
    )
    .map_err(|e| DomainError::InternalError(format!("Invalid Steam URL: {e}")))?;

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .map_err(|e| DomainError::InternalError(format!("HTTP client error: {e}")))?;
    let resp = client
        .get(url)
        .send()
        .await
        .map_err(|_| DomainError::InternalError("Steam API request failed".to_string()))?;

    if !resp.status().is_success() {
        return Err(DomainError::InternalError(format!(
            "Steam API returned status {}",
            resp.status()
        )));
    }

    let body: OwnedGamesResponse = resp
        .json()
        .await
        .map_err(|e| DomainError::InternalError(format!("Failed to parse Steam response: {e}")))?;

    Ok(body.response.games.unwrap_or_default())
}

#[cfg(feature = "real-plugins")]
pub async fn fetch_owned_games_with_client(
    client: &crate::http_client::HttpConnectorClient,
    api_key: &str,
    steam_id: &str,
) -> Result<Vec<SteamOwnedGame>, DomainError> {
    let url = build_steam_api_url(api_key, steam_id);
    let resp: OwnedGamesResponse = client.get_json(&url).await?;
    Ok(resp.response.games.unwrap_or_default())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn steam_owned_game_deserializes() {
        let json = r#"{"appid": 440, "name": "Team Fortress 2", "playtime_forever": 1234}"#;
        let game: SteamOwnedGame = serde_json::from_str(json).expect("deserialize");
        assert_eq!(game.appid, 440);
        assert_eq!(game.name, "Team Fortress 2");
        assert_eq!(game.playtime_forever, Some(1234));
    }

    #[test]
    fn owned_games_response_deserializes() {
        let json = r#"{"response": {"game_count": 1, "games": [{"appid": 440, "name": "TF2", "playtime_forever": 100}]}}"#;
        let resp: OwnedGamesResponse = serde_json::from_str(json).expect("deserialize");
        assert_eq!(resp.response.games.unwrap().len(), 1);
    }

    #[test]
    fn empty_games_response_deserializes() {
        let json = r#"{"response": {"game_count": 0, "games": []}}"#;
        let resp: OwnedGamesResponse = serde_json::from_str(json).expect("deserialize");
        assert_eq!(resp.response.games.unwrap().len(), 0);
    }

    #[test]
    fn real_fixture_deserializes() {
        let json = include_str!("../../tests/fixtures/real/steam_owned_games.json");
        let resp: OwnedGamesResponse = serde_json::from_str(json).expect("deserialize fixture");
        let games = resp.response.games.unwrap();
        assert_eq!(games.len(), 3);
        assert_eq!(games[0].name, "Team Fortress 2");
        assert_eq!(games[1].name, "Dota 2");
        assert_eq!(games[2].name, "Counter-Strike 2");
    }

    #[test]
    fn steam_api_url_is_correct() {
        let url = build_steam_api_url("TESTKEY", "123456");
        assert!(url.contains("api.steampowered.com"));
        assert!(url.contains("key=TESTKEY"));
        assert!(url.contains("steamid=123456"));
        assert!(url.contains("include_appinfo=true"));
    }

    #[test]
    fn parse_steam_credentials() {
        let json = r#"{"api_key": "TESTKEY", "steam_id": "123456"}"#;
        let creds: SteamCredentials = serde_json::from_str(json).unwrap();
        assert_eq!(creds.api_key, "TESTKEY");
        assert_eq!(creds.steam_id, "123456");
    }
}
