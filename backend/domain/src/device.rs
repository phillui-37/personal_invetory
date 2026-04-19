use async_trait::async_trait;
use chrono::{DateTime, Utc};

use crate::DomainError;

#[derive(Debug, Clone)]
pub struct Device {
    pub id: String,
    pub device_id: String,
    pub device_name: Option<String>,
    pub linked_at: DateTime<Utc>,
    pub delinked_at: Option<DateTime<Utc>>,
    pub location_count: u64,
}

#[async_trait]
pub trait DeviceRepository: Send + Sync {
    /// Returns one row per unique device_id (latest binding), with location count.
    async fn all_with_counts(&self) -> Result<Vec<Device>, DomainError>;

    /// Returns the currently active binding for a device_id, or None.
    async fn active_by_device_id(&self, device_id: &str) -> Result<Option<Device>, DomainError>;

    /// Delinks any existing active binding, then inserts a new binding.
    /// Returns the newly created Device with location_count = 0.
    async fn register(
        &self,
        device_id: &str,
        device_name: Option<&str>,
    ) -> Result<Device, DomainError>;

    /// Sets delinked_at on the active binding for device_id.
    /// Returns NotFound if device_id unknown, Conflict if already delinked.
    async fn delink(&self, device_id: &str, at: DateTime<Utc>) -> Result<(), DomainError>;
}
