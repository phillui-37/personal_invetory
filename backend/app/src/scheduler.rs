use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use domain::{DomainError, ResourceRepository, ResourceType, WebReaderMetaRepository};
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

/// Abstraction over ChapterCheckService for scheduler injection.
#[async_trait]
pub trait CheckRunner: Send + Sync {
    async fn run_check(&self, resource_id: Uuid) -> Result<(), DomainError>;
}

/// Wrapper to adapt the type-erased ChapterCheckOps into a CheckRunner.
pub struct OpsCheckRunner(pub Arc<dyn services::ChapterCheckOps>);

#[async_trait]
impl CheckRunner for OpsCheckRunner {
    async fn run_check(&self, resource_id: Uuid) -> Result<(), DomainError> {
        self.0.check_resource(resource_id).await.map(|_| ())
    }
}

pub struct Scheduler {
    runner: Arc<dyn CheckRunner>,
    web_meta_repo: Arc<dyn WebReaderMetaRepository>,
    resource_repo: Arc<dyn ResourceRepository>,
    default_interval: Duration,
    shutdown: CancellationToken,
}

impl Scheduler {
    pub fn new(
        runner: Arc<dyn CheckRunner>,
        web_meta_repo: Arc<dyn WebReaderMetaRepository>,
        resource_repo: Arc<dyn ResourceRepository>,
        default_interval: Duration,
        shutdown: CancellationToken,
    ) -> Self {
        Self {
            runner,
            web_meta_repo,
            resource_repo,
            default_interval,
            shutdown,
        }
    }

    pub fn start(self) -> JoinHandle<()> {
        tokio::spawn(async move {
            self.run_loop().await;
        })
    }

    async fn run_loop(&self) {
        loop {
            let sleep_duration = self.tick(Utc::now()).await;

            tokio::select! {
                _ = tokio::time::sleep(sleep_duration) => {}
                _ = self.shutdown.cancelled() => {
                    break;
                }
            }
        }
    }

    /// Performs one scheduler tick. Returns how long to sleep before the next tick.
    /// Visible for testing.
    pub async fn tick(&self, now: DateTime<Utc>) -> Duration {
        let resources = match self.resource_repo.list().await {
            Ok(r) => r,
            Err(e) => {
                eprintln!("scheduler: failed to list resources: {e:?}");
                return Duration::from_secs(60);
            }
        };

        let web_readers: Vec<_> = resources
            .into_iter()
            .filter(|r| r.resource_type == ResourceType::WebReader)
            .collect();

        let mut min_interval = self.default_interval;

        for resource in &web_readers {
            let meta = match self.web_meta_repo.get(resource.id).await {
                Ok(m) => m,
                Err(_) => continue,
            };

            let interval = meta
                .check_interval_secs
                .map(Duration::from_secs)
                .unwrap_or(self.default_interval);

            if interval < min_interval {
                min_interval = interval;
            }

            if is_due(meta.last_checked_at, interval, now) {
                let runner = self.runner.clone();
                let id = resource.id;
                tokio::spawn(async move {
                    if let Err(e) = runner.run_check(id).await {
                        eprintln!("scheduler: check failed for {id}: {e:?}");
                    }
                });
            }
        }

        min_interval.max(Duration::from_secs(60))
    }
}

/// Returns true if a check is due given the last check time, interval, and current time.
/// Time-injectable for testing.
pub fn is_due(
    last_checked_at: Option<DateTime<Utc>>,
    interval: Duration,
    now: DateTime<Utc>,
) -> bool {
    match last_checked_at {
        None => true,
        Some(last) => {
            let elapsed = (now - last).to_std().unwrap_or(Duration::ZERO);
            elapsed >= interval
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration as CDuration;

    #[test]
    fn is_due_returns_true_when_never_checked() {
        let now = Utc::now();
        assert!(is_due(None, Duration::from_secs(3600), now));
    }

    #[test]
    fn is_due_returns_false_when_checked_recently() {
        let now = Utc::now();
        let last = now - CDuration::seconds(100);
        assert!(!is_due(Some(last), Duration::from_secs(3600), now));
    }

    #[test]
    fn is_due_returns_true_when_interval_elapsed() {
        let now = Utc::now();
        let last = now - CDuration::seconds(3601);
        assert!(is_due(Some(last), Duration::from_secs(3600), now));
    }

    #[test]
    fn is_due_returns_true_at_exact_interval_boundary() {
        let now = Utc::now();
        let last = now - CDuration::seconds(3600);
        assert!(is_due(Some(last), Duration::from_secs(3600), now));
    }
}
