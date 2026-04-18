use domain::{NewResource, NewWebReaderMeta, ResourceType, UpdateResource, WebReaderMeta};
use url::Url;

use crate::ValidationError;

pub struct NewWebReaderInput {
    pub title: String,
    pub notes: Option<String>,
    pub url: String,
    pub site_name: Option<String>,
    pub last_checked_chapter: Option<String>,
    pub check_interval_secs: Option<u64>,
    pub progress_css_selector: Option<String>,
}

pub struct UpdateWebReaderInput {
    pub title: Option<String>,
    pub notes: Option<String>,
    pub url: Option<String>,
    pub site_name: Option<String>,
    pub last_checked_chapter: Option<String>,
    pub check_interval_secs: Option<u64>,
    pub progress_css_selector: Option<String>,
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
            check_interval_secs: input.check_interval_secs,
            last_checked_at: None,
            progress_css_selector: input.progress_css_selector.clone(),
        },
    ))
}

pub fn validate_update_web_reader(
    existing: &domain::Resource,
    existing_meta: &WebReaderMeta,
    input: &UpdateWebReaderInput,
) -> Result<(UpdateResource, NewWebReaderMeta), ValidationError> {
    let mut error = ValidationError::new();

    if let Some(title) = &input.title {
        validate_title(title, &mut error);
    } else {
        validate_title(&existing.title, &mut error);
    }

    if let Some(url) = &input.url {
        validate_url(url, &mut error);
    }

    if !error.is_empty() {
        return Err(error);
    }

    Ok((
        UpdateResource {
            title: input.title.as_ref().map(|t| t.trim().to_string()),
            notes: input.notes.clone(),
        },
        NewWebReaderMeta {
            url: input
                .url
                .as_ref()
                .map(|u| u.trim().to_string())
                .unwrap_or_else(|| existing_meta.url.clone()),
            site_name: input
                .site_name
                .clone()
                .or_else(|| existing_meta.site_name.clone()),
            last_checked_chapter: input
                .last_checked_chapter
                .clone()
                .or_else(|| existing_meta.last_checked_chapter.clone()),
            check_interval_secs: input
                .check_interval_secs
                .or(existing_meta.check_interval_secs),
            last_checked_at: existing_meta.last_checked_at,
            progress_css_selector: input
                .progress_css_selector
                .clone()
                .or_else(|| existing_meta.progress_css_selector.clone()),
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
    use chrono::Utc;
    use domain::{Resource, ResourceType, WebReaderMeta};
    use uuid::Uuid;

    use super::{validate_new_web_reader, validate_update_web_reader, NewWebReaderInput, UpdateWebReaderInput};

    fn valid_input() -> NewWebReaderInput {
        NewWebReaderInput {
            title: "Reader".to_string(),
            notes: Some("notes".to_string()),
            url: "https://example.com/chapter/1".to_string(),
            site_name: Some("Example".to_string()),
            last_checked_chapter: Some("10".to_string()),
            check_interval_secs: None,
            progress_css_selector: None,
        }
    }

    fn existing_resource() -> Resource {
        let now = Utc::now();
        Resource {
            id: Uuid::new_v4(),
            title: "Existing Reader".to_string(),
            notes: Some("existing note".to_string()),
            resource_type: ResourceType::WebReader,
            created_at: now,
            updated_at: now,
        }
    }

    fn existing_web_reader_meta() -> WebReaderMeta {
        WebReaderMeta {
            resource_id: Uuid::new_v4(),
            url: "https://existing.com/chapter/5".to_string(),
            site_name: Some("Existing Site".to_string()),
            last_checked_chapter: Some("5".to_string()),
            check_interval_secs: None,
            last_checked_at: None,
            progress_css_selector: None,
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

    #[test]
    fn validate_update_web_reader_accepts_partial_update() {
        let existing = existing_resource();
        let meta = existing_web_reader_meta();
        let input = UpdateWebReaderInput {
            title: Some("New Title".to_string()),
            notes: None,
            url: None,
            site_name: None,
            last_checked_chapter: None,
            check_interval_secs: None,
            progress_css_selector: None,
        };

        let (resource, meta_out) =
            validate_update_web_reader(&existing, &meta, &input).expect("should be valid");
        assert_eq!(resource.title, Some("New Title".to_string()));
        assert_eq!(meta_out.url, "https://existing.com/chapter/5");
        assert_eq!(meta_out.site_name, Some("Existing Site".to_string()));
        assert_eq!(meta_out.last_checked_chapter, Some("5".to_string()));
    }

    #[test]
    fn validate_update_web_reader_updates_url_when_provided() {
        let existing = existing_resource();
        let meta = existing_web_reader_meta();
        let input = UpdateWebReaderInput {
            title: None,
            notes: None,
            url: Some("https://new.com/chapter/10".to_string()),
            site_name: None,
            last_checked_chapter: Some("10".to_string()),
            check_interval_secs: None,
            progress_css_selector: None,
        };

        let (_, meta_out) =
            validate_update_web_reader(&existing, &meta, &input).expect("should be valid");
        assert_eq!(meta_out.url, "https://new.com/chapter/10");
        assert_eq!(meta_out.last_checked_chapter, Some("10".to_string()));
    }

    #[test]
    fn validate_update_web_reader_rejects_invalid_url_when_provided() {
        let existing = existing_resource();
        let meta = existing_web_reader_meta();
        let input = UpdateWebReaderInput {
            title: None,
            notes: None,
            url: Some("not-a-url".to_string()),
            site_name: None,
            last_checked_chapter: None,
            check_interval_secs: None,
            progress_css_selector: None,
        };

        let error =
            validate_update_web_reader(&existing, &meta, &input).expect_err("should fail");
        assert_eq!(error.field_errors["url"][0], "url must be a valid URL");
    }

    #[test]
    fn validate_update_web_reader_rejects_empty_title_when_provided() {
        let existing = existing_resource();
        let meta = existing_web_reader_meta();
        let input = UpdateWebReaderInput {
            title: Some("  ".to_string()),
            notes: None,
            url: None,
            site_name: None,
            last_checked_chapter: None,
            check_interval_secs: None,
            progress_css_selector: None,
        };

        let error =
            validate_update_web_reader(&existing, &meta, &input).expect_err("should fail");
        assert_eq!(error.field_errors["title"][0], "title must not be empty");
    }
}
