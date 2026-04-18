#[cfg(feature = "real-plugins")]
use domain::DomainError;
use serde::Deserialize;

#[cfg(feature = "real-plugins")]
const STEAM_API_BASE: &str = "https://api.steampowered.com";

#[derive(Debug, Clone, Deserialize)]
pub struct SteamOwnedGame {
    pub appid: u64,
    pub name: String,
    pub playtime_forever: Option<u64>,
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

#[cfg(feature = "real-plugins")]
pub async fn fetch_owned_games(
    api_key: &str,
    steam_id: &str,
) -> Result<Vec<SteamOwnedGame>, DomainError> {
    let url = format!(
        "{STEAM_API_BASE}/IPlayerService/GetOwnedGames/v0001/?key={api_key}&steamid={steam_id}&include_appinfo=true&include_played_free_games=true&format=json"
    );
    let client = reqwest::Client::new();
    let resp = client
        .get(&url)
        .send()
        .await
        .map_err(|e| DomainError::InternalError(format!("Steam API request failed: {e}")))?;

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
}
