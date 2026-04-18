use std::sync::Arc;

use domain::dedup::{DedupWarning, DedupWarningRepository, NewDedupWarning};
use domain::{
    DomainError, LocationRepository, NewResourceLocation, Resource, ResourceRepository,
    UpdateResource,
};
use uuid::Uuid;

const SIMILARITY_THRESHOLD: f64 = 0.85;

pub struct DedupService {
    resource_repo: Arc<dyn ResourceRepository>,
    location_repo: Arc<dyn LocationRepository>,
    dedup_repo: Arc<dyn DedupWarningRepository>,
}

impl DedupService {
    pub fn new(
        resource_repo: Arc<dyn ResourceRepository>,
        location_repo: Arc<dyn LocationRepository>,
        dedup_repo: Arc<dyn DedupWarningRepository>,
    ) -> Self {
        Self {
            resource_repo,
            location_repo,
            dedup_repo,
        }
    }

    pub async fn scan_for_duplicates(&self) -> Result<Vec<DedupWarning>, DomainError> {
        let resources = self.resource_repo.list().await?;
        let mut warnings = Vec::new();
        for i in 0..resources.len() {
            for j in (i + 1)..resources.len() {
                let score = self.compute_similarity(&resources[i], &resources[j]);
                if score >= SIMILARITY_THRESHOLD {
                    let already_exists = self
                        .dedup_repo
                        .exists_pair(resources[i].id, resources[j].id)
                        .await?;
                    if !already_exists {
                        let warning = self
                            .dedup_repo
                            .create(NewDedupWarning {
                                resource_id_a: resources[i].id,
                                resource_id_b: resources[j].id,
                                similarity_score: score,
                            })
                            .await?;
                        warnings.push(warning);
                    }
                }
            }
        }
        Ok(warnings)
    }

    pub fn compute_similarity(&self, a: &Resource, b: &Resource) -> f64 {
        if a.resource_type != b.resource_type {
            return 0.0;
        }
        strsim::jaro_winkler(&a.title.to_lowercase(), &b.title.to_lowercase())
    }

    pub async fn list_pending(&self) -> Result<Vec<DedupWarning>, DomainError> {
        self.dedup_repo.list_pending().await
    }

    pub async fn dismiss(&self, warning_id: Uuid) -> Result<(), DomainError> {
        self.dedup_repo.dismiss(warning_id).await
    }

    pub async fn merge(
        &self,
        warning_id: Uuid,
        keep_id: Uuid,
        discard_id: Uuid,
    ) -> Result<Resource, DomainError> {
        if keep_id == discard_id {
            return Err(DomainError::ValidationError(
                "keep_id and discard_id must be different".to_string(),
            ));
        }

        // Validate warning exists, is pending, and matches the submitted pair
        let warning = self.dedup_repo.get_by_id(warning_id).await?;
        if warning.status != domain::dedup::DedupWarningStatus::Pending {
            return Err(DomainError::ValidationError(
                "warning is not in pending state".to_string(),
            ));
        }
        let pair_matches = (warning.resource_id_a == keep_id
            && warning.resource_id_b == discard_id)
            || (warning.resource_id_a == discard_id && warning.resource_id_b == keep_id);
        if !pair_matches {
            return Err(DomainError::ValidationError(
                "keep_id/discard_id do not match the warning pair".to_string(),
            ));
        }

        let keep = self.resource_repo.get_by_id(keep_id).await?;
        let discard = self.resource_repo.get_by_id(discard_id).await?;

        // Merge notes
        let merged_notes = match (&keep.notes, &discard.notes) {
            (Some(a), Some(b)) if a != b => Some(format!("{a}\n---\n{b}")),
            (None, Some(b)) => Some(b.clone()),
            _ => keep.notes.clone(),
        };

        // Update keep resource if notes changed
        if merged_notes != keep.notes {
            self.resource_repo
                .update(
                    keep_id,
                    UpdateResource {
                        title: None,
                        notes: merged_notes,
                    },
                )
                .await?;
        }

        // Move locations from discard to keep
        let discard_locations = self.location_repo.list(discard_id).await?;
        for loc in &discard_locations {
            self.location_repo
                .add(
                    keep_id,
                    NewResourceLocation {
                        device_id: loc.device_id.clone(),
                        path_or_url: loc.path_or_url.clone(),
                        storage_type: loc.storage_type.clone(),
                    },
                )
                .await?;
        }

        // Delete discard resource
        self.resource_repo.delete(discard_id).await?;

        // Mark warning as merged
        self.dedup_repo.mark_merged(warning_id).await?;

        // Return updated keep resource
        self.resource_repo.get_by_id(keep_id).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use chrono::Utc;
    use domain::{NewResource, Resource, ResourceLocation, ResourceType, StorageType};
    use futures::executor::block_on;
    use std::sync::Mutex;

    // -- Mock DedupWarningRepository --
    struct MockDedupRepo {
        warnings: Mutex<Vec<DedupWarning>>,
    }

    impl MockDedupRepo {
        fn new() -> Self {
            Self {
                warnings: Mutex::new(Vec::new()),
            }
        }
    }

    #[async_trait]
    impl DedupWarningRepository for MockDedupRepo {
        async fn create(&self, input: NewDedupWarning) -> Result<DedupWarning, DomainError> {
            let w = DedupWarning {
                id: Uuid::new_v4(),
                resource_id_a: input.resource_id_a,
                resource_id_b: input.resource_id_b,
                similarity_score: input.similarity_score,
                status: domain::dedup::DedupWarningStatus::Pending,
                created_at: Utc::now(),
                resolved_at: None,
            };
            self.warnings.lock().unwrap().push(w.clone());
            Ok(w)
        }
        async fn list_pending(&self) -> Result<Vec<DedupWarning>, DomainError> {
            Ok(self
                .warnings
                .lock()
                .unwrap()
                .iter()
                .filter(|w| w.status == domain::dedup::DedupWarningStatus::Pending)
                .cloned()
                .collect())
        }
        async fn get_by_id(&self, id: Uuid) -> Result<DedupWarning, DomainError> {
            self.warnings
                .lock()
                .unwrap()
                .iter()
                .find(|w| w.id == id)
                .cloned()
                .ok_or(DomainError::NotFound("warning not found".to_string()))
        }
        async fn dismiss(&self, id: Uuid) -> Result<(), DomainError> {
            let mut ws = self.warnings.lock().unwrap();
            if let Some(w) = ws.iter_mut().find(|w| w.id == id) {
                w.status = domain::dedup::DedupWarningStatus::Dismissed;
                Ok(())
            } else {
                Err(DomainError::NotFound("warning not found".to_string()))
            }
        }
        async fn mark_merged(&self, id: Uuid) -> Result<(), DomainError> {
            let mut ws = self.warnings.lock().unwrap();
            if let Some(w) = ws.iter_mut().find(|w| w.id == id) {
                w.status = domain::dedup::DedupWarningStatus::Merged;
                Ok(())
            } else {
                Err(DomainError::NotFound("warning not found".to_string()))
            }
        }
        async fn exists_pair(&self, a: Uuid, b: Uuid) -> Result<bool, DomainError> {
            let ws = self.warnings.lock().unwrap();
            Ok(ws.iter().any(|w| {
                (w.resource_id_a == a && w.resource_id_b == b)
                    || (w.resource_id_a == b && w.resource_id_b == a)
            }))
        }
    }

    // -- Mock ResourceRepository --
    struct MockResourceRepo {
        resources: Mutex<Vec<Resource>>,
    }

    impl MockResourceRepo {
        fn new() -> Self {
            Self {
                resources: Mutex::new(Vec::new()),
            }
        }
    }

    #[async_trait]
    impl ResourceRepository for MockResourceRepo {
        async fn create(&self, input: NewResource) -> Result<Resource, DomainError> {
            let r = Resource {
                id: Uuid::new_v4(),
                title: input.title,
                notes: input.notes,
                resource_type: input.resource_type,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            };
            self.resources.lock().unwrap().push(r.clone());
            Ok(r)
        }
        async fn get_by_id(&self, id: Uuid) -> Result<Resource, DomainError> {
            self.resources
                .lock()
                .unwrap()
                .iter()
                .find(|r| r.id == id)
                .cloned()
                .ok_or(DomainError::NotFound("not found".into()))
        }
        async fn list(&self) -> Result<Vec<Resource>, DomainError> {
            Ok(self.resources.lock().unwrap().clone())
        }
        async fn update(&self, id: Uuid, input: UpdateResource) -> Result<Resource, DomainError> {
            let mut rs = self.resources.lock().unwrap();
            let r = rs
                .iter_mut()
                .find(|r| r.id == id)
                .ok_or(DomainError::NotFound("not found".into()))?;
            if let Some(title) = input.title {
                r.title = title;
            }
            if let Some(notes) = input.notes {
                r.notes = Some(notes);
            }
            Ok(r.clone())
        }
        async fn delete(&self, id: Uuid) -> Result<(), DomainError> {
            self.resources.lock().unwrap().retain(|r| r.id != id);
            Ok(())
        }
        async fn search(&self, _query: &str) -> Result<Vec<Resource>, DomainError> {
            Ok(vec![])
        }
    }

    // -- Mock LocationRepository --
    struct MockLocationRepo {
        locations: Mutex<Vec<(Uuid, ResourceLocation)>>,
    }

    impl MockLocationRepo {
        fn new() -> Self {
            Self {
                locations: Mutex::new(Vec::new()),
            }
        }
    }

    #[async_trait]
    impl LocationRepository for MockLocationRepo {
        async fn add(
            &self,
            resource_id: Uuid,
            input: NewResourceLocation,
        ) -> Result<ResourceLocation, DomainError> {
            let loc = ResourceLocation {
                id: Uuid::new_v4(),
                resource_id,
                device_id: input.device_id,
                path_or_url: input.path_or_url,
                storage_type: input.storage_type,
            };
            self.locations
                .lock()
                .unwrap()
                .push((resource_id, loc.clone()));
            Ok(loc)
        }
        async fn list(&self, resource_id: Uuid) -> Result<Vec<ResourceLocation>, DomainError> {
            Ok(self
                .locations
                .lock()
                .unwrap()
                .iter()
                .filter(|(rid, _)| *rid == resource_id)
                .map(|(_, l)| l.clone())
                .collect())
        }
        async fn remove(&self, _resource_id: Uuid, location_id: Uuid) -> Result<(), DomainError> {
            self.locations
                .lock()
                .unwrap()
                .retain(|(_, l)| l.id != location_id);
            Ok(())
        }
    }

    fn make_service() -> (
        DedupService,
        Arc<MockResourceRepo>,
        Arc<MockLocationRepo>,
        Arc<MockDedupRepo>,
    ) {
        let rr = Arc::new(MockResourceRepo::new());
        let lr = Arc::new(MockLocationRepo::new());
        let dr = Arc::new(MockDedupRepo::new());
        let svc = DedupService::new(rr.clone(), lr.clone(), dr.clone());
        (svc, rr, lr, dr)
    }

    #[test]
    fn compute_similarity_same_type_similar_titles() {
        let (svc, _, _, _) = make_service();
        let a = Resource {
            id: Uuid::new_v4(),
            title: "Zelda TOTK".into(),
            notes: None,
            resource_type: ResourceType::Game,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        let b = Resource {
            id: Uuid::new_v4(),
            title: "Zelda Totk".into(),
            notes: None,
            resource_type: ResourceType::Game,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        let score = svc.compute_similarity(&a, &b);
        assert!(score > 0.9, "expected high similarity, got {score}");
    }

    #[test]
    fn compute_similarity_different_types_returns_zero() {
        let (svc, _, _, _) = make_service();
        let a = Resource {
            id: Uuid::new_v4(),
            title: "Zelda".into(),
            notes: None,
            resource_type: ResourceType::Game,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        let b = Resource {
            id: Uuid::new_v4(),
            title: "Zelda".into(),
            notes: None,
            resource_type: ResourceType::Ebook,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        assert_eq!(svc.compute_similarity(&a, &b), 0.0);
    }

    #[test]
    fn scan_creates_warnings_for_similar_resources() {
        let (svc, rr, _, _) = make_service();
        block_on(async {
            rr.create(NewResource {
                title: "Zelda TOTK".into(),
                notes: None,
                resource_type: ResourceType::Game,
            })
            .await
            .unwrap();
            rr.create(NewResource {
                title: "Zelda Totk".into(),
                notes: None,
                resource_type: ResourceType::Game,
            })
            .await
            .unwrap();
            rr.create(NewResource {
                title: "Completely Different".into(),
                notes: None,
                resource_type: ResourceType::Game,
            })
            .await
            .unwrap();

            let warnings = svc.scan_for_duplicates().await.unwrap();
            assert_eq!(warnings.len(), 1);
            assert!(warnings[0].similarity_score > 0.85);
        });
    }

    #[test]
    fn scan_does_not_create_duplicate_warnings() {
        let (svc, rr, _, _) = make_service();
        block_on(async {
            rr.create(NewResource {
                title: "Zelda TOTK".into(),
                notes: None,
                resource_type: ResourceType::Game,
            })
            .await
            .unwrap();
            rr.create(NewResource {
                title: "Zelda Totk".into(),
                notes: None,
                resource_type: ResourceType::Game,
            })
            .await
            .unwrap();

            svc.scan_for_duplicates().await.unwrap();
            let warnings = svc.scan_for_duplicates().await.unwrap();
            assert_eq!(
                warnings.len(),
                0,
                "second scan should not create duplicate warnings"
            );
        });
    }

    #[test]
    fn dismiss_warning() {
        let (svc, rr, _, _dr) = make_service();
        block_on(async {
            rr.create(NewResource {
                title: "Zelda TOTK".into(),
                notes: None,
                resource_type: ResourceType::Game,
            })
            .await
            .unwrap();
            rr.create(NewResource {
                title: "Zelda Totk".into(),
                notes: None,
                resource_type: ResourceType::Game,
            })
            .await
            .unwrap();

            let warnings = svc.scan_for_duplicates().await.unwrap();
            svc.dismiss(warnings[0].id).await.unwrap();

            let pending = svc.list_pending().await.unwrap();
            assert_eq!(pending.len(), 0);
        });
    }

    #[test]
    fn merge_combines_resources() {
        let (svc, rr, lr, _) = make_service();
        block_on(async {
            let a = rr
                .create(NewResource {
                    title: "Zelda TOTK".into(),
                    notes: Some("Note A".into()),
                    resource_type: ResourceType::Game,
                })
                .await
                .unwrap();
            let b = rr
                .create(NewResource {
                    title: "Zelda Totk".into(),
                    notes: Some("Note B".into()),
                    resource_type: ResourceType::Game,
                })
                .await
                .unwrap();

            lr.add(
                b.id,
                NewResourceLocation {
                    device_id: "my-pc".into(),
                    path_or_url: "https://example.com".into(),
                    storage_type: StorageType::Platform,
                },
            )
            .await
            .unwrap();

            let warnings = svc.scan_for_duplicates().await.unwrap();
            let merged = svc.merge(warnings[0].id, a.id, b.id).await.unwrap();

            assert_eq!(merged.id, a.id);
            assert!(merged.notes.as_ref().unwrap().contains("Note A"));
            assert!(merged.notes.as_ref().unwrap().contains("Note B"));

            let locations = lr.list(a.id).await.unwrap();
            assert_eq!(locations.len(), 1);
            assert_eq!(locations[0].path_or_url, "https://example.com");

            let result = rr.get_by_id(b.id).await;
            assert!(result.is_err(), "discarded resource should be deleted");
        });
    }
}
