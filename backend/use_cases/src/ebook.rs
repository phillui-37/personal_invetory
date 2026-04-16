use domain::{NewEbookMeta, NewResource, Resource, ResourceType, UpdateResource};

use crate::ValidationError;

pub struct NewEbookInput {
    pub title: String,
    pub notes: Option<String>,
    pub author: Option<String>,
    pub isbn: Option<String>,
    pub publisher: Option<String>,
    pub language: Option<String>,
    pub file_format: Option<String>,
}

pub struct UpdateEbookInput {
    pub title: Option<String>,
    pub notes: Option<String>,
    pub author: Option<String>,
    pub isbn: Option<String>,
    pub publisher: Option<String>,
    pub language: Option<String>,
    pub file_format: Option<String>,
}

pub fn validate_new_ebook(
    input: &NewEbookInput,
) -> Result<(NewResource, NewEbookMeta), ValidationError> {
    let mut error = ValidationError::new();
    validate_title(&input.title, &mut error);
    validate_file_format(input.file_format.as_deref(), &mut error);

    if !error.is_empty() {
        return Err(error);
    }

    Ok((
        NewResource {
            title: input.title.trim().to_string(),
            notes: input.notes.clone(),
            resource_type: ResourceType::Ebook,
        },
        NewEbookMeta {
            author: input.author.clone(),
            isbn: input.isbn.clone(),
            publisher: input.publisher.clone(),
            language: input.language.clone(),
            file_format: normalize_file_format(input.file_format.as_deref()),
        },
    ))
}

pub fn validate_update_ebook(
    existing: &Resource,
    input: &UpdateEbookInput,
) -> Result<(UpdateResource, NewEbookMeta), ValidationError> {
    let mut error = ValidationError::new();

    if let Some(title) = &input.title {
        validate_title(title, &mut error);
    } else {
        validate_title(&existing.title, &mut error);
    }

    validate_file_format(input.file_format.as_deref(), &mut error);

    if !error.is_empty() {
        return Err(error);
    }

    Ok((
        UpdateResource {
            title: input.title.as_ref().map(|title| title.trim().to_string()),
            notes: input.notes.clone(),
        },
        NewEbookMeta {
            author: input.author.clone(),
            isbn: input.isbn.clone(),
            publisher: input.publisher.clone(),
            language: input.language.clone(),
            file_format: normalize_file_format(input.file_format.as_deref()),
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

fn validate_file_format(file_format: Option<&str>, error: &mut ValidationError) {
    if let Some(format) = file_format {
        let normalized = format.trim().to_ascii_lowercase();
        if normalized.is_empty() || !is_allowed_file_format(&normalized) {
            error.add_field_error(
                "file_format",
                "file_format must be one of: pdf, epub, mobi, azw3",
            );
        }
    }
}

fn normalize_file_format(file_format: Option<&str>) -> Option<String> {
    file_format.map(|value| value.trim().to_ascii_lowercase())
}

fn is_allowed_file_format(value: &str) -> bool {
    matches!(value, "pdf" | "epub" | "mobi" | "azw3")
}

#[cfg(test)]
mod tests {
    use chrono::Utc;
    use domain::{Resource, ResourceType};
    use uuid::Uuid;

    use super::{validate_new_ebook, validate_update_ebook, NewEbookInput, UpdateEbookInput};

    fn valid_new_input() -> NewEbookInput {
        NewEbookInput {
            title: "My Ebook".to_string(),
            notes: Some("notes".to_string()),
            author: Some("Author".to_string()),
            isbn: Some("123".to_string()),
            publisher: Some("Pub".to_string()),
            language: Some("EN".to_string()),
            file_format: Some("epub".to_string()),
        }
    }

    fn existing_resource() -> Resource {
        let now = Utc::now();
        Resource {
            id: Uuid::new_v4(),
            title: "Existing".to_string(),
            notes: Some("existing note".to_string()),
            resource_type: ResourceType::Ebook,
            created_at: now,
            updated_at: now,
        }
    }

    #[test]
    fn validate_new_ebook_accepts_valid_input() {
        let input = valid_new_input();
        let (resource, meta) = validate_new_ebook(&input).expect("should be valid");

        assert_eq!(resource.title, "My Ebook");
        assert_eq!(resource.resource_type, ResourceType::Ebook);
        assert_eq!(meta.file_format, Some("epub".to_string()));
    }

    #[test]
    fn validate_new_ebook_rejects_empty_title() {
        let mut input = valid_new_input();
        input.title = "   ".to_string();

        let error = validate_new_ebook(&input).expect_err("should fail");
        assert_eq!(error.field_errors["title"][0], "title must not be empty");
    }

    #[test]
    fn validate_new_ebook_rejects_title_over_500_chars() {
        let mut input = valid_new_input();
        input.title = "a".repeat(501);

        let error = validate_new_ebook(&input).expect_err("should fail");
        assert_eq!(
            error.field_errors["title"][0],
            "title must be at most 500 chars"
        );
    }

    #[test]
    fn validate_new_ebook_accepts_500_char_title() {
        let mut input = valid_new_input();
        input.title = "a".repeat(500);

        assert!(validate_new_ebook(&input).is_ok());
    }

    #[test]
    fn validate_new_ebook_rejects_invalid_file_format() {
        let mut input = valid_new_input();
        input.file_format = Some("docx".to_string());

        let error = validate_new_ebook(&input).expect_err("should fail");
        assert_eq!(
            error.field_errors["file_format"][0],
            "file_format must be one of: pdf, epub, mobi, azw3"
        );
    }

    #[test]
    fn validate_new_ebook_rejects_empty_file_format_when_set() {
        let mut input = valid_new_input();
        input.file_format = Some("   ".to_string());

        let error = validate_new_ebook(&input).expect_err("should fail");
        assert_eq!(
            error.field_errors["file_format"][0],
            "file_format must be one of: pdf, epub, mobi, azw3"
        );
    }

    #[test]
    fn validate_update_ebook_accepts_valid_input() {
        let existing = existing_resource();
        let input = UpdateEbookInput {
            title: Some("Updated".to_string()),
            notes: Some("updated note".to_string()),
            author: Some("Author".to_string()),
            isbn: Some("123".to_string()),
            publisher: Some("Pub".to_string()),
            language: Some("EN".to_string()),
            file_format: Some("mobi".to_string()),
        };

        let (resource, meta) = validate_update_ebook(&existing, &input).expect("should be valid");
        assert_eq!(resource.title, Some("Updated".to_string()));
        assert_eq!(meta.file_format, Some("mobi".to_string()));
    }

    #[test]
    fn validate_update_ebook_rejects_empty_title_when_provided() {
        let existing = existing_resource();
        let input = UpdateEbookInput {
            title: Some(" ".to_string()),
            notes: None,
            author: None,
            isbn: None,
            publisher: None,
            language: None,
            file_format: None,
        };

        let error = validate_update_ebook(&existing, &input).expect_err("should fail");
        assert_eq!(error.field_errors["title"][0], "title must not be empty");
    }

    #[test]
    fn validate_update_ebook_rejects_title_over_500_chars_when_provided() {
        let existing = existing_resource();
        let input = UpdateEbookInput {
            title: Some("a".repeat(501)),
            notes: None,
            author: None,
            isbn: None,
            publisher: None,
            language: None,
            file_format: None,
        };

        let error = validate_update_ebook(&existing, &input).expect_err("should fail");
        assert_eq!(
            error.field_errors["title"][0],
            "title must be at most 500 chars"
        );
    }

    #[test]
    fn validate_update_ebook_rejects_invalid_file_format() {
        let existing = existing_resource();
        let input = UpdateEbookInput {
            title: None,
            notes: None,
            author: None,
            isbn: None,
            publisher: None,
            language: None,
            file_format: Some("txt".to_string()),
        };

        let error = validate_update_ebook(&existing, &input).expect_err("should fail");
        assert_eq!(
            error.field_errors["file_format"][0],
            "file_format must be one of: pdf, epub, mobi, azw3"
        );
    }

    #[test]
    fn validate_update_ebook_allows_no_title_change() {
        let existing = existing_resource();
        let input = UpdateEbookInput {
            title: None,
            notes: None,
            author: None,
            isbn: None,
            publisher: None,
            language: None,
            file_format: Some("pdf".to_string()),
        };

        let (resource, meta) = validate_update_ebook(&existing, &input).expect("should be valid");
        assert_eq!(resource.title, None);
        assert_eq!(meta.file_format, Some("pdf".to_string()));
    }
}
