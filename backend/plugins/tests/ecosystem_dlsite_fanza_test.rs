/// Integration tests for DLSite ecosystem fixture-based loading and parsing.

#[cfg(test)]
mod dlsite_fixture_tests {
    use std::fs;
    use std::path::PathBuf;

    #[derive(Debug, Clone, serde::Deserialize)]
    struct DLSiteWork {
        pub workno: String,
        pub work_name: String,
        pub work_type: String,
        pub maker_name: Option<String>,
    }

    fn load_dlsite_fixture() -> Result<Vec<DLSiteWork>, Box<dyn std::error::Error>> {
        let fixture_path = PathBuf::from("tests/fixtures/dlsite_purchases.json");
        let content = fs::read_to_string(&fixture_path)?;
        let works: Vec<DLSiteWork> = serde_json::from_str(&content)?;
        Ok(works)
    }

    #[test]
    fn test_load_dlsite_fixture_succeeds() {
        let result = load_dlsite_fixture();
        assert!(result.is_ok(), "Failed to load dlsite fixture: {:?}", result);
        let works = result.unwrap();
        assert!(!works.is_empty(), "Fixture should contain works");
    }

    #[test]
    fn test_dlsite_fixture_has_expected_count() {
        let works = load_dlsite_fixture().expect("load fixture");
        assert_eq!(works.len(), 5, "Expected 5 works in fixture");
    }

    #[test]
    fn test_dlsite_fixture_contains_game() {
        let works = load_dlsite_fixture().expect("load fixture");
        let game = works.iter().find(|w| w.workno == "RJ001234");
        assert!(game.is_some(), "Fixture should contain RJ001234");
        let game = game.unwrap();
        assert_eq!(game.work_name, "魔法少女大戦");
        assert_eq!(game.work_type, "GAM");
    }

    #[test]
    fn test_dlsite_fixture_manga_detection() {
        let works = load_dlsite_fixture().expect("load fixture");
        let manga = works.iter().find(|w| w.workno == "RJ001235");
        assert!(manga.is_some());
        let manga = manga.unwrap();
        assert_eq!(manga.work_type, "MNG", "Should have MNG type for manga");
    }

    #[test]
    fn test_dlsite_fixture_video_detection() {
        let works = load_dlsite_fixture().expect("load fixture");
        let video = works.iter().find(|w| w.workno == "RJ001237");
        assert!(video.is_some());
        let video = video.unwrap();
        assert_eq!(video.work_type, "MOV");
    }

    #[test]
    fn test_dlsite_fixture_cg_detection() {
        let works = load_dlsite_fixture().expect("load fixture");
        let cg = works.iter().find(|w| w.workno == "RJ001238");
        assert!(cg.is_some());
        let cg = cg.unwrap();
        assert_eq!(cg.work_type, "CG");
    }

    #[test]
    fn test_dlsite_fixture_work_ids_valid() {
        let works = load_dlsite_fixture().expect("load fixture");
        for work in &works {
            assert!(work.workno.starts_with("RJ"), "Work ID should start with RJ");
            assert!(!work.work_name.is_empty(), "Work name should not be empty");
        }
    }

    #[test]
    fn test_dlsite_fixture_maker_extraction() {
        let works = load_dlsite_fixture().expect("load fixture");
        let with_maker = works.iter().filter(|w| w.maker_name.is_some()).count();
        assert!(with_maker > 0, "Some works should have maker info");
    }

    #[test]
    fn test_dlsite_fixture_malformed_json_error() {
        let invalid_json = r#"[{ "broken": json }]"#;
        let result: Result<Vec<DLSiteWork>, _> = serde_json::from_str(invalid_json);
        assert!(result.is_err(), "Should fail to parse invalid JSON");
    }
}

/// Integration tests for FANZA ecosystem fixture-based loading and parsing.
#[cfg(test)]
mod fanza_fixture_tests {
    use std::fs;
    use std::path::PathBuf;

    #[derive(Debug, Clone, serde::Deserialize)]
    struct FanzaItem {
        pub product_id: String,
        pub title: String,
        pub category: String,
        pub content_type: String,
        pub purchase_date: String,
        pub thumbnail: Option<String>,
    }

    fn load_fanza_fixture() -> Result<Vec<FanzaItem>, Box<dyn std::error::Error>> {
        let fixture_path = PathBuf::from("tests/fixtures/fanza_library.json");
        let content = fs::read_to_string(&fixture_path)?;
        let items: Vec<FanzaItem> = serde_json::from_str(&content)?;
        Ok(items)
    }

    #[test]
    fn test_load_fanza_fixture_succeeds() {
        let result = load_fanza_fixture();
        assert!(result.is_ok(), "Failed to load fanza fixture: {:?}", result);
        let items = result.unwrap();
        assert!(!items.is_empty(), "Fixture should contain items");
    }

    #[test]
    fn test_fanza_fixture_has_expected_count() {
        let items = load_fanza_fixture().expect("load fixture");
        assert_eq!(items.len(), 5, "Expected 5 items in fixture");
    }

    #[test]
    fn test_fanza_fixture_contains_game() {
        let items = load_fanza_fixture().expect("load fixture");
        let game = items.iter().find(|i| i.product_id == "26123456");
        assert!(game.is_some(), "Fixture should contain product 26123456");
        let game = game.unwrap();
        assert_eq!(game.title, "Harem Fantasy RPG 2024");
        assert_eq!(game.content_type, "game");
    }

    #[test]
    fn test_fanza_fixture_video_detection() {
        let items = load_fanza_fixture().expect("load fixture");
        let video = items.iter().find(|i| i.product_id == "26123457");
        assert!(video.is_some());
        let video = video.unwrap();
        assert_eq!(video.content_type, "video");
    }

    #[test]
    fn test_fanza_fixture_image_detection() {
        let items = load_fanza_fixture().expect("load fixture");
        let image = items.iter().find(|i| i.product_id == "26123458");
        assert!(image.is_some());
        let image = image.unwrap();
        assert_eq!(image.content_type, "image");
    }

    #[test]
    fn test_fanza_fixture_purchase_dates() {
        let items = load_fanza_fixture().expect("load fixture");
        let dates: Vec<&str> = items.iter().map(|i| i.purchase_date.as_str()).collect();
        assert!(dates.iter().all(|d| !d.is_empty()), "All items should have purchase dates");
    }

    #[test]
    fn test_fanza_fixture_thumbnails() {
        let items = load_fanza_fixture().expect("load fixture");
        let with_thumb = items.iter().filter(|i| i.thumbnail.is_some()).count();
        assert!(with_thumb > 0, "Some items should have thumbnails");
    }

    #[test]
    fn test_fanza_fixture_product_ids_unique() {
        let items = load_fanza_fixture().expect("load fixture");
        let ids: std::collections::HashSet<_> = items.iter().map(|i| &i.product_id).collect();
        assert_eq!(ids.len(), items.len(), "Product IDs should be unique");
    }

    #[test]
    fn test_fanza_fixture_malformed_json_error() {
        let invalid_json = r#"[{ "broken": json }]"#;
        let result: Result<Vec<FanzaItem>, _> = serde_json::from_str(invalid_json);
        assert!(result.is_err(), "Should fail to parse invalid JSON");
    }
}
