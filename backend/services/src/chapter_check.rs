use std::sync::Arc;

use domain::{
    ChapterCheck, ChapterCheckRepository, DomainError, NewWebReaderMeta, Notification,
    NotificationBroadcaster, NotificationRepository, PushNotifier, WebReaderMetaRepository,
};
use plugins::{PluginError, WebChecker};
use uuid::Uuid;

pub struct ChapterCheckService<WC, CCR, NR, WMR>
where
    WC: WebChecker + ?Sized,
    CCR: ChapterCheckRepository + ?Sized,
    NR: NotificationRepository + ?Sized,
    WMR: WebReaderMetaRepository + ?Sized,
{
    checker: Arc<WC>,
    check_repo: Arc<CCR>,
    notification_repo: Arc<NR>,
    web_meta_repo: Arc<WMR>,
    push_notifier: Option<Arc<dyn PushNotifier>>,
    broadcaster: Option<Arc<dyn NotificationBroadcaster>>,
}

impl<WC, CCR, NR, WMR> ChapterCheckService<WC, CCR, NR, WMR>
where
    WC: WebChecker + 'static + ?Sized,
    CCR: ChapterCheckRepository + ?Sized,
    NR: NotificationRepository + ?Sized,
    WMR: WebReaderMetaRepository + ?Sized,
{
    pub fn new(
        checker: Arc<WC>,
        check_repo: Arc<CCR>,
        notification_repo: Arc<NR>,
        web_meta_repo: Arc<WMR>,
        push_notifier: Option<Arc<dyn PushNotifier>>,
    ) -> Self {
        Self {
            checker,
            check_repo,
            notification_repo,
            web_meta_repo,
            push_notifier,
            broadcaster: None,
        }
    }

    pub fn with_broadcaster(mut self, broadcaster: Arc<dyn NotificationBroadcaster>) -> Self {
        self.broadcaster = Some(broadcaster);
        self
    }

    pub async fn check_resource(&self, resource_id: Uuid) -> Result<ChapterCheck, DomainError> {
        let meta = self.web_meta_repo.get(resource_id).await?;
        let checker = self.checker.clone();
        let url = meta.url.clone();
        let known_chapter_owned = meta.last_checked_chapter.clone();

        let (has_new, latest_chapter, error_message) =
            match tokio::task::spawn_blocking(move || {
                checker.check(&url, known_chapter_owned.as_deref())
            })
            .await
            .map_err(|error| DomainError::InternalError(format!("chapter check task failed: {error}")))?
            {
                Ok(result) => (result.has_new, result.latest_chapter, None),
                Err(PluginError::UnsupportedInput) => {
                    return Err(DomainError::ValidationError(format!(
                        "no site config matches URL: {}",
                        meta.url
                    )));
                }
                Err(err) => {
                    let msg = match err {
                        PluginError::IoError(s) => s,
                        PluginError::ParseError(s) => s,
                        PluginError::UnsupportedInput => unreachable!(),
                    };
                    (false, meta.last_checked_chapter.clone(), Some(msg))
                }
            };

        let chapter_check = self
            .check_repo
            .create(
                resource_id,
                has_new,
                latest_chapter.clone(),
                error_message.clone(),
            )
            .await?;

        if has_new {
            let chapter_label = latest_chapter.as_deref().unwrap_or("unknown");
            let notification = self
                .notification_repo
                .create(resource_id, format!("New chapter: {chapter_label}"))
                .await?;

            if let Some(notifier) = &self.push_notifier {
                if let Err(e) = notifier
                    .send(resource_id, "New Chapter Available", &notification.message)
                    .await
                {
                    eprintln!("push notification failed for {resource_id}: {e:?}");
                }
            }

            if let Some(broadcaster) = &self.broadcaster {
                broadcaster.broadcast(&notification);
            }
        }

        let now = chrono::Utc::now();
        self.web_meta_repo
            .upsert(
                resource_id,
                NewWebReaderMeta {
                    url: meta.url,
                    site_name: meta.site_name,
                    last_checked_chapter: latest_chapter,
                    check_interval_secs: meta.check_interval_secs,
                    last_checked_at: Some(now),
                    progress_css_selector: meta.progress_css_selector,
                },
            )
            .await?;

        Ok(chapter_check)
    }

    pub async fn list_check_history(
        &self,
        resource_id: Uuid,
    ) -> Result<Vec<ChapterCheck>, DomainError> {
        self.check_repo.list(resource_id).await
    }

    pub async fn list_notifications(
        &self,
        unread_only: bool,
    ) -> Result<Vec<Notification>, DomainError> {
        self.notification_repo.list(unread_only).await
    }

    pub async fn mark_notification_read(&self, id: Uuid) -> Result<(), DomainError> {
        self.notification_repo.mark_read(id).await
    }
}

/// Type-erased interface for adapters / scheduler.
#[async_trait::async_trait]
pub trait ChapterCheckOps: Send + Sync {
    async fn check_resource(&self, resource_id: Uuid) -> Result<ChapterCheck, DomainError>;
    async fn list_check_history(&self, resource_id: Uuid) -> Result<Vec<ChapterCheck>, DomainError>;
    async fn list_notifications(&self, unread_only: bool) -> Result<Vec<Notification>, DomainError>;
    async fn mark_notification_read(&self, id: Uuid) -> Result<(), DomainError>;
}

#[async_trait::async_trait]
impl<WC, CCR, NR, WMR> ChapterCheckOps for ChapterCheckService<WC, CCR, NR, WMR>
where
    WC: WebChecker + 'static + ?Sized,
    CCR: ChapterCheckRepository + 'static + ?Sized,
    NR: NotificationRepository + 'static + ?Sized,
    WMR: WebReaderMetaRepository + 'static + ?Sized,
{
    async fn check_resource(&self, resource_id: Uuid) -> Result<ChapterCheck, DomainError> {
        self.check_resource(resource_id).await
    }

    async fn list_check_history(&self, resource_id: Uuid) -> Result<Vec<ChapterCheck>, DomainError> {
        self.list_check_history(resource_id).await
    }

    async fn list_notifications(&self, unread_only: bool) -> Result<Vec<Notification>, DomainError> {
        self.list_notifications(unread_only).await
    }

    async fn mark_notification_read(&self, id: Uuid) -> Result<(), DomainError> {
        self.mark_notification_read(id).await
    }
}
