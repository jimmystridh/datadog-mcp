//! Pagination utilities for Datadog API responses
//!
//! Provides helpers for working with paginated API endpoints.

use serde::{Deserialize, Serialize};

/// Page-based pagination parameters
#[derive(Debug, Clone, Serialize)]
pub struct PageParams {
    /// Number of items per page
    #[serde(rename = "page[size]")]
    pub page_size: i32,
    /// Page offset (0-indexed)
    #[serde(rename = "page[offset]", skip_serializing_if = "Option::is_none")]
    pub page_offset: Option<i64>,
}

impl PageParams {
    /// Create new page parameters with the given size
    pub fn new(page_size: i32) -> Self {
        Self {
            page_size,
            page_offset: None,
        }
    }

    /// Create page parameters with size and offset
    pub fn with_offset(page_size: i32, offset: i64) -> Self {
        Self {
            page_size,
            page_offset: Some(offset),
        }
    }
}

impl Default for PageParams {
    fn default() -> Self {
        Self::new(100)
    }
}

/// Cursor-based pagination parameters
#[derive(Debug, Clone, Serialize)]
pub struct CursorParams {
    /// Number of items per page
    #[serde(rename = "page[limit]")]
    pub limit: i32,
    /// Cursor for next page
    #[serde(rename = "page[cursor]", skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,
}

impl CursorParams {
    /// Create new cursor parameters with the given limit
    pub fn new(limit: i32) -> Self {
        Self {
            limit,
            cursor: None,
        }
    }

    /// Create cursor parameters with limit and cursor
    pub fn with_cursor(limit: i32, cursor: String) -> Self {
        Self {
            limit,
            cursor: Some(cursor),
        }
    }
}

impl Default for CursorParams {
    fn default() -> Self {
        Self::new(100)
    }
}

/// Pagination metadata from API responses
#[derive(Debug, Clone, Deserialize, Default)]
pub struct PaginationMeta {
    /// Total count of items (if available)
    #[serde(default)]
    pub total_count: Option<i64>,
    /// Total number of pages (if available)
    #[serde(default)]
    pub total_pages: Option<i64>,
    /// Offset for the next page (if available)
    #[serde(default)]
    pub next_offset: Option<i64>,
    /// Cursor for the next page (if available)
    #[serde(default)]
    pub next_cursor: Option<String>,
}

impl PaginationMeta {
    /// Check if there are more pages
    pub fn has_next(&self) -> bool {
        self.next_offset.is_some() || self.next_cursor.is_some()
    }
}

/// Result of a paginated request
#[derive(Debug, Clone)]
pub struct PaginatedResponse<T> {
    /// Items from this page
    pub items: Vec<T>,
    /// Pagination metadata
    pub meta: PaginationMeta,
}

impl<T> PaginatedResponse<T> {
    /// Create a new paginated response
    pub fn new(items: Vec<T>, meta: PaginationMeta) -> Self {
        Self { items, meta }
    }

    /// Check if there are more pages
    pub fn has_next(&self) -> bool {
        self.meta.has_next()
    }

    /// Get the total count if available
    pub fn total_count(&self) -> Option<i64> {
        self.meta.total_count
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_page_params_default() {
        let params = PageParams::default();
        assert_eq!(params.page_size, 100);
        assert!(params.page_offset.is_none());
    }

    #[test]
    fn test_page_params_with_offset() {
        let params = PageParams::with_offset(50, 100);
        assert_eq!(params.page_size, 50);
        assert_eq!(params.page_offset, Some(100));
    }

    #[test]
    fn test_cursor_params_default() {
        let params = CursorParams::default();
        assert_eq!(params.limit, 100);
        assert!(params.cursor.is_none());
    }

    #[test]
    fn test_cursor_params_with_cursor() {
        let params = CursorParams::with_cursor(50, "abc123".to_string());
        assert_eq!(params.limit, 50);
        assert_eq!(params.cursor, Some("abc123".to_string()));
    }

    #[test]
    fn test_pagination_meta_has_next() {
        let meta_with_offset = PaginationMeta {
            next_offset: Some(100),
            ..Default::default()
        };
        assert!(meta_with_offset.has_next());

        let meta_with_cursor = PaginationMeta {
            next_cursor: Some("abc".to_string()),
            ..Default::default()
        };
        assert!(meta_with_cursor.has_next());

        let meta_empty = PaginationMeta::default();
        assert!(!meta_empty.has_next());
    }

    #[test]
    fn test_paginated_response() {
        let response = PaginatedResponse::new(
            vec![1, 2, 3],
            PaginationMeta {
                total_count: Some(100),
                next_offset: Some(3),
                ..Default::default()
            },
        );

        assert_eq!(response.items.len(), 3);
        assert!(response.has_next());
        assert_eq!(response.total_count(), Some(100));
    }
}
