use std::sync::Arc;

use chrono::Utc;

use domain::device::{Device, DeviceRepository};
use domain::DomainError;

#[derive(Debug, Clone)]
pub struct DeviceInfo {
    pub id: String,
    pub device_id: String,
    /// Always non-empty: DB name, or DEVICE_NAME env, or hostname (current device),
    /// or device_id (other devices).
    pub device_name: String,
    pub linked_at: chrono::DateTime<Utc>,
    pub delinked_at: Option<chrono::DateTime<Utc>>,
    pub location_count: u64,
    pub is_current: bool,
}

pub struct DeviceService {
    repo: Arc<dyn DeviceRepository>,
    current_device_id: String,
    /// Effective display-name fallback for the current device (DEVICE_NAME or hostname).
    fallback_hostname: String,
}

impl DeviceService {
    pub fn new(
        repo: Arc<dyn DeviceRepository>,
        current_device_id: String,
        fallback_hostname: String,
    ) -> Self {
        Self {
            repo,
            current_device_id,
            fallback_hostname,
        }
    }

    fn to_info(&self, device: Device) -> DeviceInfo {
        let is_current = device.device_id == self.current_device_id;
        let device_name = device.device_name.clone().unwrap_or_else(|| {
            if is_current {
                self.fallback_hostname.clone()
            } else {
                device.device_id.clone()
            }
        });
        DeviceInfo {
            id: device.id,
            device_id: device.device_id,
            device_name,
            linked_at: device.linked_at,
            delinked_at: device.delinked_at,
            location_count: device.location_count,
            is_current,
        }
    }

    pub async fn list(&self) -> Result<Vec<DeviceInfo>, DomainError> {
        let devices = self.repo.all_with_counts().await?;
        Ok(devices.into_iter().map(|d| self.to_info(d)).collect())
    }

    pub async fn current(&self) -> Result<DeviceInfo, DomainError> {
        self.repo
            .active_by_device_id(&self.current_device_id)
            .await?
            .map(|d| self.to_info(d))
            .ok_or_else(|| DomainError::NotFound("current device not registered".into()))
    }

    pub async fn register(
        &self,
        device_id: &str,
        device_name: Option<&str>,
    ) -> Result<DeviceInfo, DomainError> {
        let device_id = device_id.trim();
        if device_id.is_empty() {
            return Err(DomainError::ValidationError(
                "device_id must not be empty".into(),
            ));
        }
        let device = self.repo.register(device_id, device_name).await?;
        Ok(self.to_info(device))
    }

    pub async fn delink(&self, device_id: &str) -> Result<(), DomainError> {
        if device_id == self.current_device_id {
            return Err(DomainError::ValidationError(
                "cannot delink the current device".into(),
            ));
        }
        self.repo.delink(device_id, Utc::now()).await
    }
}
