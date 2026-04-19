mod chapter_check;
mod dedup;
mod device;
mod ebook;
mod game;
mod image;
mod search;
mod sync_service;
mod video;
mod vault;
mod web_reader;

use domain::DomainError;
use use_cases::ValidationError;

pub use chapter_check::{ChapterCheckOps, ChapterCheckService};
pub use dedup::DedupService;
pub use device::{DeviceInfo, DeviceService};
pub use domain::SearchStrategyKind;
pub use ebook::{EbookDetail, EbookService};
pub use game::{GameDetail, GameService};
pub use image::{ImageDetail, ImageService};
pub use search::SearchConfig;
pub use use_cases::ebook::{NewEbookInput, UpdateEbookInput};
pub use use_cases::game::{NewGameInput, UpdateGameInput};
pub use use_cases::image::{NewImageInput, UpdateImageInput};
pub use use_cases::location::NewLocationInput;
pub use use_cases::video::{NewVideoInput, UpdateVideoInput};
pub use use_cases::web_reader::{NewWebReaderInput, UpdateWebReaderInput};
pub use sync_service::SyncService;
pub use vault::VaultService;
pub use video::{VideoDetail, VideoService};
pub use web_reader::{WebReaderDetail, WebReaderService};

pub(crate) fn map_validation_error(error: ValidationError) -> DomainError {
    DomainError::ValidationError(format_validation_error(&error))
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
