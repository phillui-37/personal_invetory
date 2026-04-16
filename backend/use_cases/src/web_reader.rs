use domain::{NewResource, NewWebReaderMeta, ResourceType};
use url::Url;

use crate::ValidationError;

pub struct NewWebReaderInput {
    pub title: String,
    pub notes: Option<String>,
    pub url: String,
    pub site_name: Option<String>,
    pub last_checked_chapter: Option<String>,
}

pub fn validate_new_web_reader(
    input: &NewWebReaderInput,
) -> Result<(NewResource, NewWebReaderMeta), ValidationError> {
    let mut error = ValidationError::new();
    validate_title(&input.title, &mut error);
    validate_url(&input.url, &mut error);

    if !error.is_empty() {
        return Err(error);
    }

    Ok((
        NewResource {
            title: input.title.trim().to_string(),
            notes: input.notes.clone(),
            resource_type: ResourceType::WebReader,
        },
        NewWebReaderMeta {
            url: input.url.trim().to_string(),
            site_name: input.site_name.clone(),
            last_checked_chapter: input.last_checked_chapter.clone(),
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

fn validate_url(value: &str, error: &mut ValidationError) {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        error.add_field_error("url", "url must not be empty");
        return;
    }

    if Url::parse(trimmed).is_err() {
        error.add_field_error("url", "url must be a valid URL");
    }
}

#[cfg(test)]
mod tests {
    use domain::ResourceType;

    use super::{validate_new_web_reader, NewWebReaderInput};

    fn valid_input() -> NewWebReaderInput {
        NewWebReaderInput {
            title: "Reader".to_string(),
            notes: Some("notes".to_string()),
            url: "https://example.com/chapter/1".to_string(),
            site_name: Some("Example".to_string()),
            last_checked_chapter: Some("10".to_string()),
        }
    }

    #[test]
    fn validate_new_web_reader_accepts_valid_input() {
        let input = valid_input();

        let (resource, meta) = validate_new_web_reader(&input).expect("should be valid");
        assert_eq!(resource.title, "Reader");
        assert_eq!(resource.resource_type, ResourceType::WebReader);
        assert_eq!(meta.url, "https://example.com/chapter/1");
    }

    #[test]
    fn validate_new_web_reader_rejects_empty_title() {
        let mut input = valid_input();
        input.title = " ".to_string();

        let error = validate_new_web_reader(&input).expect_err("should fail");
        assert_eq!(error.field_errors["title"][0], "title must not be empty");
    }

    #[test]
    fn validate_new_web_reader_rejects_title_over_500_chars() {
        let mut input = valid_input();
        input.title = "a".repeat(501);

        let error = validate_new_web_reader(&input).expect_err("should fail");
        assert_eq!(
            error.field_errors["title"][0],
            "title must be at most 500 chars"
        );
    }

    #[test]
    fn validate_new_web_reader_accepts_500_char_title() {
        let mut input = valid_input();
        input.title = "a".repeat(500);

        assert!(validate_new_web_reader(&input).is_ok());
    }

    #[test]
    fn validate_new_web_reader_rejects_empty_url() {
        let mut input = valid_input();
        input.url = " ".to_string();

        let error = validate_new_web_reader(&input).expect_err("should fail");
        assert_eq!(error.field_errors["url"][0], "url must not be empty");
    }

    #[test]
    fn validate_new_web_reader_rejects_invalid_url() {
        let mut input = valid_input();
        input.url = "invalid-url".to_string();

        let error = validate_new_web_reader(&input).expect_err("should fail");
        assert_eq!(error.field_errors["url"][0], "url must be a valid URL");
    }
}
