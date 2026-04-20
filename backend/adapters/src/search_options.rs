use domain::DomainError;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum SortField {
    Title,
    #[serde(rename = "date_added")]
    DateAdded,
}

impl SortField {
    pub fn from_str(s: &str) -> Result<Self, DomainError> {
        match s.trim().to_lowercase().as_str() {
            "title" => Ok(SortField::Title),
            "date_added" => Ok(SortField::DateAdded),
            _ => Err(DomainError::ValidationError(
                "sort_by must be 'title' or 'date_added'".into(),
            )),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum SortOrder {
    Asc,
    Desc,
}

impl SortOrder {
    pub fn from_str(s: &str) -> Result<Self, DomainError> {
        match s.trim().to_lowercase().as_str() {
            "asc" => Ok(SortOrder::Asc),
            "desc" => Ok(SortOrder::Desc),
            _ => Err(DomainError::ValidationError(
                "sort_order must be 'asc' or 'desc'".into(),
            )),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct SearchOptionsQuery {
    pub sort_by: Option<String>,
    pub sort_order: Option<String>,
    pub with_facets: Option<bool>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct FacetCount {
    pub name: String,
    pub count: usize,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct SearchFacets {
    pub formats: Vec<FacetCount>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sort_field_from_str_title() {
        let field = SortField::from_str("title").unwrap();
        assert_eq!(field, SortField::Title);
    }

    #[test]
    fn test_sort_field_from_str_date_added() {
        let field = SortField::from_str("date_added").unwrap();
        assert_eq!(field, SortField::DateAdded);
    }

    #[test]
    fn test_sort_field_from_str_invalid() {
        let result = SortField::from_str("invalid");
        assert!(result.is_err());
    }

    #[test]
    fn test_sort_field_from_str_case_insensitive() {
        let field = SortField::from_str("TITLE").unwrap();
        assert_eq!(field, SortField::Title);
    }

    #[test]
    fn test_sort_order_from_str_asc() {
        let order = SortOrder::from_str("asc").unwrap();
        assert_eq!(order, SortOrder::Asc);
    }

    #[test]
    fn test_sort_order_from_str_desc() {
        let order = SortOrder::from_str("desc").unwrap();
        assert_eq!(order, SortOrder::Desc);
    }

    #[test]
    fn test_sort_order_from_str_case_insensitive() {
        let order = SortOrder::from_str("DESC").unwrap();
        assert_eq!(order, SortOrder::Desc);
    }

    #[test]
    fn test_sort_order_from_str_invalid() {
        let result = SortOrder::from_str("invalid");
        assert!(result.is_err());
    }
}
