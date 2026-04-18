use domain::{GameMeta, NewGameMeta, NewResource, ResourceType, UpdateResource};

use crate::ValidationError;

pub struct NewGameInput {
    pub title: String,
    pub notes: Option<String>,
    pub platform: Option<String>,
    pub store: Option<String>,
    pub developer: Option<String>,
    pub publisher: Option<String>,
    pub manual_notes: Option<String>,
}

pub struct UpdateGameInput {
    pub title: Option<String>,
    pub notes: Option<String>,
    pub platform: Option<String>,
    pub store: Option<String>,
    pub developer: Option<String>,
    pub publisher: Option<String>,
    pub manual_notes: Option<String>,
}

pub fn validate_new_game(
    input: &NewGameInput,
) -> Result<(NewResource, NewGameMeta), ValidationError> {
    let mut error = ValidationError::new();
    validate_title(&input.title, &mut error);

    if !error.is_empty() {
        return Err(error);
    }

    Ok((
        NewResource {
            title: input.title.trim().to_string(),
            notes: input.notes.clone(),
            resource_type: ResourceType::Game,
        },
        NewGameMeta {
            platform: input.platform.clone(),
            store: input.store.clone(),
            developer: input.developer.clone(),
            publisher: input.publisher.clone(),
            manual_notes: input.manual_notes.clone(),
        },
    ))
}

pub fn validate_update_game(
    existing: &domain::Resource,
    existing_meta: &GameMeta,
    input: &UpdateGameInput,
) -> Result<(UpdateResource, NewGameMeta), ValidationError> {
    let mut error = ValidationError::new();

    if let Some(title) = &input.title {
        validate_title(title, &mut error);
    } else {
        validate_title(&existing.title, &mut error);
    }

    if !error.is_empty() {
        return Err(error);
    }

    Ok((
        UpdateResource {
            title: input.title.as_ref().map(|title| title.trim().to_string()),
            notes: input.notes.clone(),
        },
        NewGameMeta {
            platform: input
                .platform
                .clone()
                .or_else(|| existing_meta.platform.clone()),
            store: input
                .store
                .clone()
                .or_else(|| existing_meta.store.clone()),
            developer: input
                .developer
                .clone()
                .or_else(|| existing_meta.developer.clone()),
            publisher: input
                .publisher
                .clone()
                .or_else(|| existing_meta.publisher.clone()),
            manual_notes: input
                .manual_notes
                .clone()
                .or_else(|| existing_meta.manual_notes.clone()),
        },
    ))
}

fn validate_title(title: &str, error: &mut ValidationError) {
    let trimmed = title.trim();
    if trimmed.is_empty() {
        error.add_field_error("title", "title must not be empty");
        return;
    }

    if trimmed.chars().count() > 500 {
        error.add_field_error("title", "title must be at most 500 chars");
    }
}

#[cfg(test)]
mod tests {
    use chrono::Utc;
    use domain::{self, Resource, ResourceType};
    use uuid::Uuid;

    use super::{validate_new_game, validate_update_game, NewGameInput, UpdateGameInput};

    fn valid_new_input() -> NewGameInput {
        NewGameInput {
            title: "My Game".to_string(),
            notes: Some("notes".to_string()),
            platform: Some("PC".to_string()),
            store: Some("Steam".to_string()),
            developer: Some("Dev Studio".to_string()),
            publisher: Some("Publisher Inc".to_string()),
            manual_notes: Some("manual notes".to_string()),
        }
    }

    fn existing_resource() -> Resource {
        let now = Utc::now();
        Resource {
            id: Uuid::new_v4(),
            title: "Existing".to_string(),
            notes: Some("existing note".to_string()),
            resource_type: ResourceType::Game,
            created_at: now,
            updated_at: now,
        }
    }

    fn existing_meta() -> domain::GameMeta {
        domain::GameMeta {
            resource_id: Uuid::new_v4(),
            platform: Some("PS5".to_string()),
            store: Some("PSN".to_string()),
            developer: Some("Existing Dev".to_string()),
            publisher: Some("Existing Pub".to_string()),
            manual_notes: Some("existing manual notes".to_string()),
        }
    }

    #[test]
    fn validate_new_game_accepts_valid_input() {
        let input = valid_new_input();
        let (resource, meta) = validate_new_game(&input).expect("should be valid");

        assert_eq!(resource.title, "My Game");
        assert_eq!(resource.resource_type, ResourceType::Game);
        assert_eq!(meta.platform, Some("PC".to_string()));
        assert_eq!(meta.store, Some("Steam".to_string()));
    }

    #[test]
    fn validate_new_game_rejects_empty_title() {
        let mut input = valid_new_input();
        input.title = "   ".to_string();

        let error = validate_new_game(&input).expect_err("should fail");
        assert_eq!(error.field_errors["title"][0], "title must not be empty");
    }

    #[test]
    fn validate_new_game_rejects_title_over_500_chars() {
        let mut input = valid_new_input();
        input.title = "a".repeat(501);

        let error = validate_new_game(&input).expect_err("should fail");
        assert_eq!(
            error.field_errors["title"][0],
            "title must be at most 500 chars"
        );
    }

    #[test]
    fn validate_new_game_accepts_500_char_title() {
        let mut input = valid_new_input();
        input.title = "a".repeat(500);

        assert!(validate_new_game(&input).is_ok());
    }

    #[test]
    fn validate_update_game_accepts_valid_input() {
        let existing = existing_resource();
        let meta = existing_meta();
        let input = UpdateGameInput {
            title: Some("Updated".to_string()),
            notes: Some("updated note".to_string()),
            platform: Some("Switch".to_string()),
            store: Some("eShop".to_string()),
            developer: Some("New Dev".to_string()),
            publisher: Some("New Pub".to_string()),
            manual_notes: Some("new manual notes".to_string()),
        };

        let (resource, meta_out) =
            validate_update_game(&existing, &meta, &input).expect("should be valid");
        assert_eq!(resource.title, Some("Updated".to_string()));
        assert_eq!(meta_out.platform, Some("Switch".to_string()));
    }

    #[test]
    fn validate_update_game_rejects_empty_title_when_provided() {
        let existing = existing_resource();
        let meta = existing_meta();
        let input = UpdateGameInput {
            title: Some(" ".to_string()),
            notes: None,
            platform: None,
            store: None,
            developer: None,
            publisher: None,
            manual_notes: None,
        };

        let error = validate_update_game(&existing, &meta, &input).expect_err("should fail");
        assert_eq!(error.field_errors["title"][0], "title must not be empty");
    }

    #[test]
    fn validate_update_game_rejects_title_over_500_chars_when_provided() {
        let existing = existing_resource();
        let meta = existing_meta();
        let input = UpdateGameInput {
            title: Some("a".repeat(501)),
            notes: None,
            platform: None,
            store: None,
            developer: None,
            publisher: None,
            manual_notes: None,
        };

        let error = validate_update_game(&existing, &meta, &input).expect_err("should fail");
        assert_eq!(
            error.field_errors["title"][0],
            "title must be at most 500 chars"
        );
    }

    #[test]
    fn validate_update_game_allows_no_title_change() {
        let existing = existing_resource();
        let meta = existing_meta();
        let input = UpdateGameInput {
            title: None,
            notes: None,
            platform: Some("PC".to_string()),
            store: None,
            developer: None,
            publisher: None,
            manual_notes: None,
        };

        let (resource, meta_out) =
            validate_update_game(&existing, &meta, &input).expect("should be valid");
        assert_eq!(resource.title, None);
        assert_eq!(meta_out.platform, Some("PC".to_string()));
    }

    #[test]
    fn validate_update_game_preserves_existing_meta_when_fields_omitted() {
        let existing = existing_resource();
        let meta = existing_meta();
        let input = UpdateGameInput {
            title: Some("New Title".to_string()),
            notes: None,
            platform: None,
            store: None,
            developer: None,
            publisher: None,
            manual_notes: None,
        };

        let (_, meta_out) =
            validate_update_game(&existing, &meta, &input).expect("should be valid");
        assert_eq!(meta_out.platform, Some("PS5".to_string()));
        assert_eq!(meta_out.store, Some("PSN".to_string()));
        assert_eq!(meta_out.developer, Some("Existing Dev".to_string()));
        assert_eq!(meta_out.publisher, Some("Existing Pub".to_string()));
        assert_eq!(meta_out.manual_notes, Some("existing manual notes".to_string()));
    }
}
