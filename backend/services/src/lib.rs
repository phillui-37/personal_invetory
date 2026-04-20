mod chapter_check;
mod dedup;
mod device;
mod ebook;
mod game;
mod image;
mod otp_service;
mod progress;
mod search;
mod search_aggregation;
mod sync_service;
mod tag;
mod video;
mod vault;
mod web_reader;

use std::collections::HashSet;

use domain::{DomainError, Resource, ResourceType};
use use_cases::ValidationError;
use uuid::Uuid;

pub use chapter_check::{ChapterCheckOps, ChapterCheckService};
pub use dedup::DedupService;
pub use device::{DeviceInfo, DeviceService};
pub use domain::SearchStrategyKind;
pub use ebook::{EbookDetail, EbookService};
pub use game::{GameDetail, GameService};
pub use image::{ImageDetail, ImageService};
pub use otp_service::OtpInteractionService;
pub use progress::ProgressService;
pub use search::SearchConfig;
pub use search_aggregation::{count_formats, sort_resources, SortField, SortOrder, FormatCount};
pub use use_cases::ebook::{NewEbookInput, UpdateEbookInput};
pub use use_cases::game::{NewGameInput, UpdateGameInput};
pub use use_cases::image::{NewImageInput, UpdateImageInput};
pub use use_cases::location::NewLocationInput;
pub use use_cases::video::{NewVideoInput, UpdateVideoInput};
pub use use_cases::web_reader::{NewWebReaderInput, UpdateWebReaderInput};
pub use sync_service::SyncService;
pub use tag::TagService;
pub use vault::VaultService;
pub use video::{VideoDetail, VideoService};
pub use web_reader::{WebReaderDetail, WebReaderService};

pub(crate) fn map_validation_error(error: ValidationError) -> DomainError {
    DomainError::ValidationError(format_validation_error(&error))
}

pub(crate) fn filter_resources_by_type(
    resources: Vec<Resource>,
    resource_type: ResourceType,
    allowed_ids: Option<&HashSet<Uuid>>,
) -> Vec<Resource> {
    resources
        .into_iter()
        .filter(|resource| {
            resource.resource_type == resource_type
                && allowed_ids
                    .map(|ids| ids.contains(&resource.id))
                    .unwrap_or(true)
        })
        .collect()
}

fn format_validation_error(error: &ValidationError) -> String {
    error
        .field_errors
        .iter()
        .map(|(field, messages)| format!("{field}: {}", messages.join(", ")))
        .collect::<Vec<_>>()
        .join("; ")
}

pub fn services_ready() -> bool {
    domain::domain_ready() && use_cases::use_cases_ready() && plugins::plugins_ready()
}

#[cfg(test)]
mod tests {
    use chrono::Utc;
    use std::collections::HashSet;

    use domain::{Resource, ResourceType};
    use uuid::Uuid;

    use crate::filter_resources_by_type;

    #[test]
    fn filter_resources_by_type_keeps_matching_type_without_id_filter() {
        let now = Utc::now();
        let ebook_id = Uuid::new_v4();
        let video_id = Uuid::new_v4();
        let resources = vec![
            Resource {
                id: ebook_id,
                title: "Book".into(),
                notes: None,
                resource_type: ResourceType::Ebook,
                created_at: now,
                updated_at: now,
            },
            Resource {
                id: video_id,
                title: "Video".into(),
                notes: None,
                resource_type: ResourceType::Video,
                created_at: now,
                updated_at: now,
            },
        ];

        let filtered = filter_resources_by_type(resources, ResourceType::Ebook, None);

        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].id, ebook_id);
    }

    #[test]
    fn filter_resources_by_type_applies_allowed_ids_with_type_filter() {
        let now = Utc::now();
        let tagged_ebook_id = Uuid::new_v4();
        let untagged_ebook_id = Uuid::new_v4();
        let tagged_video_id = Uuid::new_v4();
        let resources = vec![
            Resource {
                id: tagged_ebook_id,
                title: "Tagged Book".into(),
                notes: None,
                resource_type: ResourceType::Ebook,
                created_at: now,
                updated_at: now,
            },
            Resource {
                id: untagged_ebook_id,
                title: "Untagged Book".into(),
                notes: None,
                resource_type: ResourceType::Ebook,
                created_at: now,
                updated_at: now,
            },
            Resource {
                id: tagged_video_id,
                title: "Tagged Video".into(),
                notes: None,
                resource_type: ResourceType::Video,
                created_at: now,
                updated_at: now,
            },
        ];
        let allowed_ids = HashSet::from([tagged_ebook_id, tagged_video_id]);

        let filtered =
            filter_resources_by_type(resources, ResourceType::Ebook, Some(&allowed_ids));

        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].id, tagged_ebook_id);
    }
}
