use std::collections::BTreeMap;

use domain::{Resource, ResourceType};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SortField {
    Title,
    DateAdded,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortOrder {
    Asc,
    Desc,
}

pub fn sort_resources(
    mut resources: Vec<Resource>,
    sort_field: SortField,
    sort_order: SortOrder,
) -> Vec<Resource> {
    resources.sort_by(|a, b| {
        let ordering = match sort_field {
            SortField::Title => a.title.cmp(&b.title),
            SortField::DateAdded => a.created_at.cmp(&b.created_at),
        };

        match sort_order {
            SortOrder::Asc => ordering,
            SortOrder::Desc => ordering.reverse(),
        }
    });

    resources
}

pub struct FormatCount {
    pub name: String,
    pub count: usize,
}

pub fn count_formats(resources: &[Resource]) -> Vec<FormatCount> {
    let mut counts: BTreeMap<String, usize> = BTreeMap::new();

    for resource in resources {
        let format = match resource.resource_type {
            ResourceType::Ebook => "ebook".to_string(),
            ResourceType::WebReader => "web-reader".to_string(),
            ResourceType::Image => "image".to_string(),
            ResourceType::Video => "video".to_string(),
            ResourceType::Game => "game".to_string(),
        };

        *counts.entry(format).or_insert(0) += 1;
    }

    counts
        .into_iter()
        .map(|(name, count)| FormatCount { name, count })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use uuid::Uuid;

    fn create_test_resource(title: &str, created_at_offset_days: i64) -> Resource {
        Resource {
            id: Uuid::new_v4(),
            title: title.to_string(),
            notes: None,
            resource_type: ResourceType::Ebook,
            created_at: Utc::now() - chrono::Duration::days(created_at_offset_days),
            updated_at: Utc::now(),
        }
    }

    #[test]
    fn test_sort_by_title_asc() {
        let resources = vec![
            create_test_resource("Charlie", 0),
            create_test_resource("Alice", 0),
            create_test_resource("Bob", 0),
        ];

        let sorted = sort_resources(resources, SortField::Title, SortOrder::Asc);
        assert_eq!(sorted[0].title, "Alice");
        assert_eq!(sorted[1].title, "Bob");
        assert_eq!(sorted[2].title, "Charlie");
    }

    #[test]
    fn test_sort_by_title_desc() {
        let resources = vec![
            create_test_resource("Alice", 0),
            create_test_resource("Charlie", 0),
            create_test_resource("Bob", 0),
        ];

        let sorted = sort_resources(resources, SortField::Title, SortOrder::Desc);
        assert_eq!(sorted[0].title, "Charlie");
        assert_eq!(sorted[1].title, "Bob");
        assert_eq!(sorted[2].title, "Alice");
    }

    #[test]
    fn test_sort_by_date_added_asc() {
        let resources = vec![
            create_test_resource("Old", 10),
            create_test_resource("Recent", 1),
            create_test_resource("Older", 20),
        ];

        let sorted = sort_resources(resources, SortField::DateAdded, SortOrder::Asc);
        assert_eq!(sorted[0].title, "Older");
        assert_eq!(sorted[1].title, "Old");
        assert_eq!(sorted[2].title, "Recent");
    }

    #[test]
    fn test_sort_by_date_added_desc() {
        let resources = vec![
            create_test_resource("Old", 10),
            create_test_resource("Recent", 1),
            create_test_resource("Older", 20),
        ];

        let sorted = sort_resources(resources, SortField::DateAdded, SortOrder::Desc);
        assert_eq!(sorted[0].title, "Recent");
        assert_eq!(sorted[1].title, "Old");
        assert_eq!(sorted[2].title, "Older");
    }

    #[test]
    fn test_count_formats_empty() {
        let resources = vec![];
        let counts = count_formats(&resources);
        assert!(counts.is_empty());
    }

    #[test]
    fn test_count_formats_mixed() {
        let resources = vec![
            create_test_resource("Ebook1", 0),
            create_test_resource("Ebook2", 1),
        ];
        let counts = count_formats(&resources);
        assert_eq!(counts.len(), 1);
        assert_eq!(counts[0].name, "ebook");
        assert_eq!(counts[0].count, 2);
    }
}

