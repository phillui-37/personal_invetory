use domain::DomainError;
use serde::Deserialize;

/// A parsed HAR entry containing the essential information for API endpoint extraction.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HarEntry {
    pub request_url: String,
    pub request_method: String,
    pub response_status: u16,
    pub response_body: String,
    pub request_cookies: Vec<HarCookie>,
}

/// A HAR cookie with the fields we care about.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HarCookie {
    pub name: String,
    pub value: String,
    pub domain: Option<String>,
    pub path: Option<String>,
}

// Internal serde-deserializable structs for HAR JSON parsing
#[derive(Debug, Deserialize)]
struct HarFile {
    log: HarLog,
}

#[derive(Debug, Deserialize)]
struct HarLog {
    entries: Vec<HarRawEntry>,
}

#[derive(Debug, Deserialize)]
struct HarRawEntry {
    request: HarRequest,
    response: HarResponse,
}

#[derive(Debug, Deserialize)]
struct HarRequest {
    method: String,
    url: String,
    #[serde(default)]
    cookies: Vec<HarRawCookie>,
}

#[derive(Debug, Deserialize)]
struct HarResponse {
    status: u16,
    content: HarContent,
}

#[derive(Debug, Deserialize)]
struct HarContent {
    #[serde(default)]
    text: String,
}

#[derive(Debug, Deserialize)]
struct HarRawCookie {
    name: String,
    value: String,
    #[serde(default)]
    domain: Option<String>,
    #[serde(default)]
    path: Option<String>,
}

/// Parses a HAR JSON string and returns a vector of HarEntry objects.
pub fn parse_har(json: &str) -> Result<Vec<HarEntry>, DomainError> {
    let har_file: HarFile = serde_json::from_str(json)
        .map_err(|e| DomainError::ValidationError(format!("HAR parse error: {e}")))?;

    let entries = har_file
        .log
        .entries
        .into_iter()
        .map(|raw_entry| {
            let request_cookies = raw_entry
                .request
                .cookies
                .into_iter()
                .map(|raw_cookie| HarCookie {
                    name: raw_cookie.name,
                    value: raw_cookie.value,
                    domain: raw_cookie.domain,
                    path: raw_cookie.path,
                })
                .collect();

            HarEntry {
                request_url: raw_entry.request.url,
                request_method: raw_entry.request.method,
                response_status: raw_entry.response.status,
                response_body: raw_entry.response.content.text,
                request_cookies,
            }
        })
        .collect();

    Ok(entries)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_har_entries() {
        let har_json = r#"{
            "log": {
                "entries": [
                    {
                        "request": {
                            "method": "GET",
                            "url": "https://api.example.com/games",
                            "cookies": []
                        },
                        "response": {
                            "status": 200,
                            "content": {
                                "text": "{\"games\":[{\"id\":1,\"name\":\"Test Game\"}]}"
                            }
                        }
                    }
                ]
            }
        }"#;

        let entries = parse_har(har_json).unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].request_url, "https://api.example.com/games");
        assert_eq!(entries[0].request_method, "GET");
        assert_eq!(entries[0].response_status, 200);
        assert_eq!(entries[0].response_body, r#"{"games":[{"id":1,"name":"Test Game"}]}"#);
        assert_eq!(entries[0].request_cookies.len(), 0);
    }

    #[test]
    fn extracts_cookies_from_har() {
        let har_json = r#"{
            "log": {
                "entries": [
                    {
                        "request": {
                            "method": "POST",
                            "url": "https://api.example.com/auth",
                            "cookies": [
                                {
                                    "name": "session",
                                    "value": "abc123",
                                    "domain": ".example.com",
                                    "path": "/"
                                },
                                {
                                    "name": "csrf",
                                    "value": "xyz789"
                                }
                            ]
                        },
                        "response": {
                            "status": 201,
                            "content": {
                                "text": "{\"token\":\"bearer123\"}"
                            }
                        }
                    }
                ]
            }
        }"#;

        let entries = parse_har(har_json).unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].request_cookies.len(), 2);
        assert_eq!(entries[0].request_cookies[0].name, "session");
        assert_eq!(entries[0].request_cookies[0].value, "abc123");
        assert_eq!(entries[0].request_cookies[0].domain, Some(".example.com".to_string()));
        assert_eq!(entries[0].request_cookies[0].path, Some("/".to_string()));
        assert_eq!(entries[0].request_cookies[1].name, "csrf");
        assert_eq!(entries[0].request_cookies[1].value, "xyz789");
        assert_eq!(entries[0].request_cookies[1].domain, None);
        assert_eq!(entries[0].request_cookies[1].path, None);
    }

    #[test]
    fn handles_empty_har() {
        let har_json = r#"{
            "log": {
                "entries": []
            }
        }"#;

        let entries = parse_har(har_json).unwrap();
        assert_eq!(entries.len(), 0);
    }

    #[test]
    fn rejects_invalid_json() {
        let invalid_json = r#"{"not": "a har file"}"#;
        let result = parse_har(invalid_json);
        assert!(result.is_err());
        assert!(matches!(result, Err(DomainError::ValidationError(_))));
    }
}