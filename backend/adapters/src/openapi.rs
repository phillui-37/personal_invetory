use domain::{
    ChapterCheck, EbookMeta, Notification, Resource, ResourceLocation, ResourceType, StorageType,
    WebReaderMeta,
};
use utoipa::openapi::security::{HttpAuthScheme, HttpBuilder, SecurityScheme};
use utoipa::{Modify, OpenApi};

use crate::{
    batch_import::{
        BatchImportEbookEntry, BatchImportFailure, BatchImportLocationInput, BatchImportResponse,
        BatchImportSuccess,
    },
    ebook::{AddEbookRequest, AddLocationRequest as EbookAddLocationRequest, EbookDetailResponse, UpdateEbookRequest},
    state::NotificationEvent,
    web_reader::{
        AddLocationRequest as WebReaderAddLocationRequest, UpsertWebReaderRequest,
        UpdateWebReaderRequest, WebReaderDetailResponse,
    },
};

struct SecurityAddon;

impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "bearer_auth",
                SecurityScheme::Http(
                    HttpBuilder::new()
                        .scheme(HttpAuthScheme::Bearer)
                        .bearer_format("JWT")
                        .build(),
                ),
            );
        }
    }
}

#[derive(OpenApi)]
#[openapi(
    modifiers(&SecurityAddon),
    paths(
        crate::ebook::list_ebooks,
        crate::ebook::search_ebooks,
        crate::ebook::add_ebook,
        crate::ebook::ebook_detail,
        crate::ebook::update_ebook,
        crate::ebook::delete_ebook,
        crate::ebook::add_ebook_location,
        crate::ebook::remove_ebook_location,
        crate::web_reader::list_web_readers,
        crate::web_reader::search_web_readers,
        crate::web_reader::add_web_reader,
        crate::web_reader::web_reader_detail,
        crate::web_reader::update_web_reader,
        crate::web_reader::delete_web_reader,
        crate::web_reader::add_web_reader_location,
        crate::web_reader::remove_web_reader_location,
        crate::chapter_check::trigger_chapter_check,
        crate::chapter_check::list_chapter_checks,
        crate::notifications::list_notifications,
        crate::notifications::mark_notification_read,
        crate::notifications::notifications_stream,
        crate::batch_import::batch_import_ebooks,
    ),
    components(
        schemas(
            Resource,
            ResourceType,
            ResourceLocation,
            StorageType,
            EbookMeta,
            WebReaderMeta,
            ChapterCheck,
            Notification,
            NotificationEvent,
            EbookDetailResponse,
            AddEbookRequest,
            UpdateEbookRequest,
            EbookAddLocationRequest,
            WebReaderDetailResponse,
            UpsertWebReaderRequest,
            UpdateWebReaderRequest,
            WebReaderAddLocationRequest,
            BatchImportEbookEntry,
            BatchImportLocationInput,
            BatchImportSuccess,
            BatchImportFailure,
            BatchImportResponse,
        )
    ),
    info(
        title = "Personal Inventory API",
        version = "1.0.0",
        description = "API for managing personal inventory resources (ebooks, web readers)"
    )
)]
pub struct ApiDoc;

pub fn generate_openapi_json() -> String {
    ApiDoc::openapi()
        .to_pretty_json()
        .unwrap_or_else(|_| "{}".to_string())
}
