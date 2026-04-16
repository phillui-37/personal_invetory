use domain::{NewResourceLocation, StorageType};

use crate::ValidationError;

pub struct NewLocationInput {
    pub device_id: String,
    pub path_or_url: String,
    pub storage_type: String,
}

pub fn validate_new_location(
    input: &NewLocationInput,
) -> Result<NewResourceLocation, ValidationError> {
    let mut error = ValidationError::new();
    let Some(storage_type) = parse_storage_type(&input.storage_type) else {
        error.add_field_error(
            "storage_type",
            "storage_type must be one of: localfs, nas, platform, portable",
        );
        return Err(error);
    };

    Ok(NewResourceLocation {
        device_id: input.device_id.clone(),
        path_or_url: input.path_or_url.clone(),
        storage_type,
    })
}

fn parse_storage_type(value: &str) -> Option<StorageType> {
    let normalized = value.trim().to_ascii_lowercase().replace(['_', '-'], "");
    match normalized.as_str() {
        "localfs" => Some(StorageType::LocalFs),
        "nas" => Some(StorageType::Nas),
        "platform" => Some(StorageType::Platform),
        "portable" => Some(StorageType::Portable),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use domain::StorageType;

    use super::{validate_new_location, NewLocationInput};

    fn valid_input() -> NewLocationInput {
        NewLocationInput {
            device_id: "device-001".to_string(),
            path_or_url: "/books/book.epub".to_string(),
            storage_type: "localfs".to_string(),
        }
    }

    #[test]
    fn validate_new_location_accepts_known_storage_type() {
        let input = valid_input();

        let location = validate_new_location(&input).expect("should be valid");
        assert_eq!(location.storage_type, StorageType::LocalFs);
    }

    #[test]
    fn validate_new_location_accepts_storage_type_variant_format() {
        let mut input = valid_input();
        input.storage_type = "Local_FS".to_string();

        let location = validate_new_location(&input).expect("should be valid");
        assert_eq!(location.storage_type, StorageType::LocalFs);
    }

    #[test]
    fn validate_new_location_rejects_unknown_storage_type() {
        let mut input = valid_input();
        input.storage_type = "cloud".to_string();

        let error = validate_new_location(&input).expect_err("should fail");
        assert_eq!(
            error.field_errors["storage_type"][0],
            "storage_type must be one of: localfs, nas, platform, portable"
        );
    }
}
