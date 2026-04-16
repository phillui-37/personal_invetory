use std::path::{Path, PathBuf};
use std::{fs::OpenOptions, io::Write};

use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EnvWriteOutcome {
    Created,
    Appended,
    Failed(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ApiKeyBootstrapResult {
    Ready {
        api_key: String,
    },
    GeneratedAndMustExit {
        api_key: String,
        env_path: PathBuf,
        env_write: EnvWriteOutcome,
    },
}

pub fn bootstrap_api_key(api_key: Option<String>, env_path: &Path) -> ApiKeyBootstrapResult {
    if let Some(api_key) = api_key
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
    {
        return ApiKeyBootstrapResult::Ready { api_key };
    }

    let generated_key = Uuid::new_v4().to_string();
    let existed_before_write = env_path.exists();
    let env_write = match OpenOptions::new().create(true).append(true).open(env_path) {
        Ok(mut file) => {
            let line = format!("API_KEY={generated_key}\n");
            match file.write_all(line.as_bytes()) {
                Ok(()) => {
                    if existed_before_write {
                        EnvWriteOutcome::Appended
                    } else {
                        EnvWriteOutcome::Created
                    }
                }
                Err(error) => EnvWriteOutcome::Failed(error.to_string()),
            }
        }
        Err(error) => EnvWriteOutcome::Failed(error.to_string()),
    };

    ApiKeyBootstrapResult::GeneratedAndMustExit {
        api_key: generated_key,
        env_path: env_path.to_path_buf(),
        env_write,
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;

    use uuid::Uuid;

    use super::{bootstrap_api_key, ApiKeyBootstrapResult, EnvWriteOutcome};

    fn test_artifact_path(name: &str) -> PathBuf {
        let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("test-artifacts");
        fs::create_dir_all(&dir).expect("create test artifact directory");
        dir.join(format!("{name}-{}.env", Uuid::new_v4()))
    }

    #[test]
    fn bootstrap_returns_existing_api_key_without_generation() {
        let env_path = test_artifact_path("existing");

        let result = bootstrap_api_key(Some(String::from("known-key")), &env_path);
        assert_eq!(
            result,
            ApiKeyBootstrapResult::Ready {
                api_key: String::from("known-key")
            }
        );
        assert!(!env_path.exists());
    }

    #[test]
    fn bootstrap_generates_key_and_creates_env_file_when_missing() {
        let env_path = test_artifact_path("create");

        let result = bootstrap_api_key(None, &env_path);
        let ApiKeyBootstrapResult::GeneratedAndMustExit {
            api_key,
            env_path: result_path,
            env_write,
        } = result
        else {
            panic!("missing API key should trigger generated+must-exit branch");
        };

        assert_eq!(result_path, env_path);
        assert!(
            Uuid::parse_str(&api_key).is_ok(),
            "generated API key should be UUIDv4"
        );
        assert!(matches!(
            env_write,
            EnvWriteOutcome::Created | EnvWriteOutcome::Appended
        ));

        let content = fs::read_to_string(&env_path).expect("created .env should be readable");
        assert!(content.contains(&format!("API_KEY={api_key}")));

        fs::remove_file(&env_path).expect("cleanup test env file");
    }

    #[test]
    fn bootstrap_appends_key_when_env_file_exists() {
        let env_path = test_artifact_path("append");
        fs::write(&env_path, "DATABASE_URL=sqlite://./inventory.db\n").expect("seed env file");

        let result = bootstrap_api_key(None, &env_path);
        let ApiKeyBootstrapResult::GeneratedAndMustExit {
            api_key, env_write, ..
        } = result
        else {
            panic!("missing API key should trigger generated+must-exit branch");
        };

        assert!(matches!(env_write, EnvWriteOutcome::Appended));

        let content = fs::read_to_string(&env_path).expect("appended .env should be readable");
        assert!(content.contains("DATABASE_URL=sqlite://./inventory.db"));
        assert!(content.contains(&format!("API_KEY={api_key}")));

        fs::remove_file(&env_path).expect("cleanup test env file");
    }

    #[test]
    fn bootstrap_treats_blank_api_key_as_missing_and_generates() {
        let env_path = test_artifact_path("blank-api-key");

        let result = bootstrap_api_key(Some(String::from("   ")), &env_path);
        let ApiKeyBootstrapResult::GeneratedAndMustExit { api_key, .. } = result else {
            panic!("blank API key should be treated as missing");
        };

        assert!(
            Uuid::parse_str(&api_key).is_ok(),
            "generated API key should be UUIDv4"
        );

        fs::remove_file(&env_path).expect("cleanup test env file");
    }
}
