use domain::{ImageMeta, NewImageMeta, NewResource, ResourceType, UpdateResource};

use crate::ValidationError;

pub struct NewImageInput {
    pub title: String,
    pub notes: Option<String>,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub file_format: Option<String>,
    pub file_size_bytes: Option<u64>,
}

pub struct UpdateImageInput {
    pub title: Option<String>,
    pub notes: Option<String>,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub file_format: Option<String>,
    pub file_size_bytes: Option<u64>,
}

impl Clone for UpdateImageInput {
    fn clone(&self) -> Self {
        Self {
            title: self.title.clone(),
            notes: self.notes.clone(),
            width: self.width,
            height: self.height,
            file_format: self.file_format.clone(),
            file_size_bytes: self.file_size_bytes,
        }
    }
}

pub fn validate_new_image(
    input: &NewImageInput,
) -> Result<(NewResource, NewImageMeta), ValidationError> {
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
            resource_type: ResourceType::Image,
        },
        NewImageMeta {
            width: input.width,
            height: input.height,
            file_format: normalize_file_format(input.file_format.as_deref()),
            file_size_bytes: input.file_size_bytes,
        },
    ))
}

pub fn validate_update_image(
    existing: &domain::Resource,
    existing_meta: &ImageMeta,
    input: &UpdateImageInput,
) -> Result<(UpdateResource, NewImageMeta), ValidationError> {
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
        NewImageMeta {
            width: input.width.or(existing_meta.width),
            height: input.height.or(existing_meta.height),
            file_format: normalize_file_format(input.file_format.as_deref())
                .or_else(|| existing_meta.file_format.clone()),
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
                "file_format must be one of: png, jpg, jpeg, gif, bmp, webp, svg, tiff",
            );
        }
    }
}

fn normalize_file_format(file_format: Option<&str>) -> Option<String> {
    file_format.map(|value| value.trim().to_ascii_lowercase())
}

fn is_allowed_file_format(value: &str) -> bool {
    matches!(value, "png" | "jpg" | "jpeg" | "gif" | "bmp" | "webp" | "svg" | "tiff")
}

#[cfg(test)]
mod tests {
    use chrono::Utc;
    use domain::{self, Resource, ResourceType};
    use uuid::Uuid;

    use super::{validate_new_image, validate_update_image, NewImageInput, UpdateImageInput};

    fn valid_new_input() -> NewImageInput {
        NewImageInput {
            title: "My Image".to_string(),
            notes: Some("notes".to_string()),
            width: Some(1920),
            height: Some(1080),
            file_format: Some("png".to_string()),
            file_size_bytes: Some(2048),
        }
    }

    fn existing_resource() -> Resource {
        let now = Utc::now();
        Resource {
            id: Uuid::new_v4(),
            title: "Existing".to_string(),
            notes: Some("existing note".to_string()),
            resource_type: ResourceType::Image,
            created_at: now,
            updated_at: now,
        }
    }

    fn existing_meta() -> domain::ImageMeta {
        domain::ImageMeta {
            resource_id: Uuid::new_v4(),
            width: Some(800),
            height: Some(600),
            file_format: Some("jpg".to_string()),
            file_size_bytes: Some(1024),
        }
    }

    #[test]
    fn validate_new_image_accepts_valid_input() {
        let input = valid_new_input();
        let (resource, meta) = validate_new_image(&input).expect("should be valid");

        assert_eq!(resource.title, "My Image");
        assert_eq!(resource.resource_type, ResourceType::Image);
        assert_eq!(meta.file_format, Some("png".to_string()));
        assert_eq!(meta.width, Some(1920));
        assert_eq!(meta.height, Some(1080));
    }

    #[test]
    fn validate_new_image_rejects_empty_title() {
        let mut input = valid_new_input();
        input.title = "   ".to_string();

        let error = validate_new_image(&input).expect_err("should fail");
        assert_eq!(error.field_errors["title"][0], "title must not be empty");
    }

    #[test]
    fn validate_new_image_rejects_title_over_500_chars() {
        let mut input = valid_new_input();
        input.title = "a".repeat(501);

        let error = validate_new_image(&input).expect_err("should fail");
        assert_eq!(
            error.field_errors["title"][0],
            "title must be at most 500 chars"
        );
    }

    #[test]
    fn validate_new_image_accepts_500_char_title() {
        let mut input = valid_new_input();
        input.title = "a".repeat(500);

        assert!(validate_new_image(&input).is_ok());
    }

    #[test]
    fn validate_new_image_rejects_invalid_file_format() {
        let mut input = valid_new_input();
        input.file_format = Some("docx".to_string());

        let error = validate_new_image(&input).expect_err("should fail");
        assert_eq!(
            error.field_errors["file_format"][0],
            "file_format must be one of: png, jpg, jpeg, gif, bmp, webp, svg, tiff"
        );
    }

    #[test]
    fn validate_new_image_rejects_empty_file_format_when_set() {
        let mut input = valid_new_input();
        input.file_format = Some("   ".to_string());

        let error = validate_new_image(&input).expect_err("should fail");
        assert_eq!(
            error.field_errors["file_format"][0],
            "file_format must be one of: png, jpg, jpeg, gif, bmp, webp, svg, tiff"
        );
    }

    #[test]
    fn validate_update_image_accepts_valid_input() {
        let existing = existing_resource();
        let meta = existing_meta();
        let input = UpdateImageInput {
            title: Some("Updated".to_string()),
            notes: Some("updated note".to_string()),
            width: Some(3840),
            height: Some(2160),
            file_format: Some("webp".to_string()),
            file_size_bytes: Some(4096),
        };

        let (resource, meta_out) =
            validate_update_image(&existing, &meta, &input).expect("should be valid");
        assert_eq!(resource.title, Some("Updated".to_string()));
        assert_eq!(meta_out.file_format, Some("webp".to_string()));
    }

    #[test]
    fn validate_update_image_rejects_empty_title_when_provided() {
        let existing = existing_resource();
        let meta = existing_meta();
        let input = UpdateImageInput {
            title: Some(" ".to_string()),
            notes: None,
            width: None,
            height: None,
            file_format: None,
            file_size_bytes: None,
        };

        let error = validate_update_image(&existing, &meta, &input).expect_err("should fail");
        assert_eq!(error.field_errors["title"][0], "title must not be empty");
    }

    #[test]
    fn validate_update_image_rejects_title_over_500_chars_when_provided() {
        let existing = existing_resource();
        let meta = existing_meta();
        let input = UpdateImageInput {
            title: Some("a".repeat(501)),
            notes: None,
            width: None,
            height: None,
            file_format: None,
            file_size_bytes: None,
        };

        let error = validate_update_image(&existing, &meta, &input).expect_err("should fail");
        assert_eq!(
            error.field_errors["title"][0],
            "title must be at most 500 chars"
        );
    }

    #[test]
    fn validate_update_image_rejects_invalid_file_format() {
        let existing = existing_resource();
        let meta = existing_meta();
        let input = UpdateImageInput {
            title: None,
            notes: None,
            width: None,
            height: None,
            file_format: Some("exe".to_string()),
            file_size_bytes: None,
        };

        let error = validate_update_image(&existing, &meta, &input).expect_err("should fail");
        assert_eq!(
            error.field_errors["file_format"][0],
            "file_format must be one of: png, jpg, jpeg, gif, bmp, webp, svg, tiff"
        );
    }

    #[test]
    fn validate_update_image_allows_no_title_change() {
        let existing = existing_resource();
        let meta = existing_meta();
        let input = UpdateImageInput {
            title: None,
            notes: None,
            width: None,
            height: None,
            file_format: Some("png".to_string()),
            file_size_bytes: None,
        };

        let (resource, meta_out) =
            validate_update_image(&existing, &meta, &input).expect("should be valid");
        assert_eq!(resource.title, None);
        assert_eq!(meta_out.file_format, Some("png".to_string()));
    }

    #[test]
    fn validate_update_image_preserves_existing_meta_when_fields_omitted() {
        let existing = existing_resource();
        let meta = existing_meta();
        let input = UpdateImageInput {
            title: Some("New Title".to_string()),
            notes: None,
            width: None,
            height: None,
            file_format: None,
            file_size_bytes: None,
        };

        let (_, meta_out) =
            validate_update_image(&existing, &meta, &input).expect("should be valid");
        assert_eq!(meta_out.width, Some(800));
        assert_eq!(meta_out.height, Some(600));
        assert_eq!(meta_out.file_format, Some("jpg".to_string()));
        assert_eq!(meta_out.file_size_bytes, Some(1024));
    }
}
