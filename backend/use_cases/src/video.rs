use domain::{NewResource, NewVideoMeta, ResourceType, UpdateResource, VideoMeta};

use crate::ValidationError;

pub struct NewVideoInput {
    pub title: String,
    pub notes: Option<String>,
    pub duration_secs: Option<u64>,
    pub file_format: Option<String>,
    pub resolution: Option<String>,
    pub file_size_bytes: Option<u64>,
}

pub struct UpdateVideoInput {
    pub title: Option<String>,
    pub notes: Option<String>,
    pub duration_secs: Option<u64>,
    pub file_format: Option<String>,
    pub resolution: Option<String>,
    pub file_size_bytes: Option<u64>,
}

impl Clone for UpdateVideoInput {
    fn clone(&self) -> Self {
        Self {
            title: self.title.clone(),
            notes: self.notes.clone(),
            duration_secs: self.duration_secs,
            file_format: self.file_format.clone(),
            resolution: self.resolution.clone(),
            file_size_bytes: self.file_size_bytes,
        }
    }
}

pub fn validate_new_video(
    input: &NewVideoInput,
) -> Result<(NewResource, NewVideoMeta), ValidationError> {
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
            resource_type: ResourceType::Video,
        },
        NewVideoMeta {
            duration_secs: input.duration_secs,
            file_format: normalize_file_format(input.file_format.as_deref()),
            resolution: input.resolution.clone(),
            file_size_bytes: input.file_size_bytes,
        },
    ))
}

pub fn validate_update_video(
    existing: &domain::Resource,
    existing_meta: &VideoMeta,
    input: &UpdateVideoInput,
) -> Result<(UpdateResource, NewVideoMeta), ValidationError> {
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
        NewVideoMeta {
            duration_secs: input.duration_secs.or(existing_meta.duration_secs),
            file_format: normalize_file_format(input.file_format.as_deref())
                .or_else(|| existing_meta.file_format.clone()),
            resolution: input
                .resolution
                .clone()
                .or_else(|| existing_meta.resolution.clone()),
            file_size_bytes: input.file_size_bytes.or(existing_meta.file_size_bytes),
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
                "file_format must be one of: mp4, mkv, avi, webm, mov, wmv, flv",
            );
        }
    }
}

fn normalize_file_format(file_format: Option<&str>) -> Option<String> {
    file_format.map(|value| value.trim().to_ascii_lowercase())
}

fn is_allowed_file_format(value: &str) -> bool {
    matches!(value, "mp4" | "mkv" | "avi" | "webm" | "mov" | "wmv" | "flv")
}

#[cfg(test)]
mod tests {
    use chrono::Utc;
    use domain::{self, Resource, ResourceType};
    use uuid::Uuid;

    use super::{validate_new_video, validate_update_video, NewVideoInput, UpdateVideoInput};

    fn valid_new_input() -> NewVideoInput {
        NewVideoInput {
            title: "My Video".to_string(),
            notes: Some("notes".to_string()),
            duration_secs: Some(3600),
            file_format: Some("mp4".to_string()),
            resolution: Some("1920x1080".to_string()),
            file_size_bytes: Some(1_000_000),
        }
    }

    fn existing_resource() -> Resource {
        let now = Utc::now();
        Resource {
            id: Uuid::new_v4(),
            title: "Existing".to_string(),
            notes: Some("existing note".to_string()),
            resource_type: ResourceType::Video,
            created_at: now,
            updated_at: now,
        }
    }

    fn existing_meta() -> domain::VideoMeta {
        domain::VideoMeta {
            resource_id: Uuid::new_v4(),
            duration_secs: Some(7200),
            file_format: Some("mkv".to_string()),
            resolution: Some("1280x720".to_string()),
            file_size_bytes: Some(500_000),
        }
    }

    #[test]
    fn validate_new_video_accepts_valid_input() {
        let input = valid_new_input();
        let (resource, meta) = validate_new_video(&input).expect("should be valid");

        assert_eq!(resource.title, "My Video");
        assert_eq!(resource.resource_type, ResourceType::Video);
        assert_eq!(meta.file_format, Some("mp4".to_string()));
        assert_eq!(meta.duration_secs, Some(3600));
    }

    #[test]
    fn validate_new_video_rejects_empty_title() {
        let mut input = valid_new_input();
        input.title = "   ".to_string();

        let error = validate_new_video(&input).expect_err("should fail");
        assert_eq!(error.field_errors["title"][0], "title must not be empty");
    }

    #[test]
    fn validate_new_video_rejects_title_over_500_chars() {
        let mut input = valid_new_input();
        input.title = "a".repeat(501);

        let error = validate_new_video(&input).expect_err("should fail");
        assert_eq!(
            error.field_errors["title"][0],
            "title must be at most 500 chars"
        );
    }

    #[test]
    fn validate_new_video_accepts_500_char_title() {
        let mut input = valid_new_input();
        input.title = "a".repeat(500);

        assert!(validate_new_video(&input).is_ok());
    }

    #[test]
    fn validate_new_video_rejects_invalid_file_format() {
        let mut input = valid_new_input();
        input.file_format = Some("docx".to_string());

        let error = validate_new_video(&input).expect_err("should fail");
        assert_eq!(
            error.field_errors["file_format"][0],
            "file_format must be one of: mp4, mkv, avi, webm, mov, wmv, flv"
        );
    }

    #[test]
    fn validate_new_video_rejects_empty_file_format_when_set() {
        let mut input = valid_new_input();
        input.file_format = Some("   ".to_string());

        let error = validate_new_video(&input).expect_err("should fail");
        assert_eq!(
            error.field_errors["file_format"][0],
            "file_format must be one of: mp4, mkv, avi, webm, mov, wmv, flv"
        );
    }

    #[test]
    fn validate_update_video_accepts_valid_input() {
        let existing = existing_resource();
        let meta = existing_meta();
        let input = UpdateVideoInput {
            title: Some("Updated".to_string()),
            notes: Some("updated note".to_string()),
            duration_secs: Some(5400),
            file_format: Some("webm".to_string()),
            resolution: Some("3840x2160".to_string()),
            file_size_bytes: Some(2_000_000),
        };

        let (resource, meta_out) =
            validate_update_video(&existing, &meta, &input).expect("should be valid");
        assert_eq!(resource.title, Some("Updated".to_string()));
        assert_eq!(meta_out.file_format, Some("webm".to_string()));
    }

    #[test]
    fn validate_update_video_rejects_empty_title_when_provided() {
        let existing = existing_resource();
        let meta = existing_meta();
        let input = UpdateVideoInput {
            title: Some(" ".to_string()),
            notes: None,
            duration_secs: None,
            file_format: None,
            resolution: None,
            file_size_bytes: None,
        };

        let error = validate_update_video(&existing, &meta, &input).expect_err("should fail");
        assert_eq!(error.field_errors["title"][0], "title must not be empty");
    }

    #[test]
    fn validate_update_video_rejects_title_over_500_chars_when_provided() {
        let existing = existing_resource();
        let meta = existing_meta();
        let input = UpdateVideoInput {
            title: Some("a".repeat(501)),
            notes: None,
            duration_secs: None,
            file_format: None,
            resolution: None,
            file_size_bytes: None,
        };

        let error = validate_update_video(&existing, &meta, &input).expect_err("should fail");
        assert_eq!(
            error.field_errors["title"][0],
            "title must be at most 500 chars"
        );
    }

    #[test]
    fn validate_update_video_rejects_invalid_file_format() {
        let existing = existing_resource();
        let meta = existing_meta();
        let input = UpdateVideoInput {
            title: None,
            notes: None,
            duration_secs: None,
            file_format: Some("txt".to_string()),
            resolution: None,
            file_size_bytes: None,
        };

        let error = validate_update_video(&existing, &meta, &input).expect_err("should fail");
        assert_eq!(
            error.field_errors["file_format"][0],
            "file_format must be one of: mp4, mkv, avi, webm, mov, wmv, flv"
        );
    }

    #[test]
    fn validate_update_video_allows_no_title_change() {
        let existing = existing_resource();
        let meta = existing_meta();
        let input = UpdateVideoInput {
            title: None,
            notes: None,
            duration_secs: None,
            file_format: Some("mp4".to_string()),
            resolution: None,
            file_size_bytes: None,
        };

        let (resource, meta_out) =
            validate_update_video(&existing, &meta, &input).expect("should be valid");
        assert_eq!(resource.title, None);
        assert_eq!(meta_out.file_format, Some("mp4".to_string()));
    }

    #[test]
    fn validate_update_video_preserves_existing_meta_when_fields_omitted() {
        let existing = existing_resource();
        let meta = existing_meta();
        let input = UpdateVideoInput {
            title: Some("New Title".to_string()),
            notes: None,
            duration_secs: None,
            file_format: None,
            resolution: None,
            file_size_bytes: None,
        };

        let (_, meta_out) =
            validate_update_video(&existing, &meta, &input).expect("should be valid");
        assert_eq!(meta_out.duration_secs, Some(7200));
        assert_eq!(meta_out.file_format, Some("mkv".to_string()));
        assert_eq!(meta_out.resolution, Some("1280x720".to_string()));
        assert_eq!(meta_out.file_size_bytes, Some(500_000));
    }
}
