use crate::{Error, Result};
use serde::Serialize;

/// Validated one-based pagination for APIs using `page[number]`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct NumberedPage {
    #[serde(rename = "page[number]")]
    number: i64,
    #[serde(rename = "page[size]")]
    size: i64,
}

impl NumberedPage {
    pub fn new(number: i64, size: i64) -> Result<Self> {
        if number < 1 {
            return Err(Error::ConfigError(
                "page number must be greater than zero".to_string(),
            ));
        }
        if !(1..=100).contains(&size) {
            return Err(Error::ConfigError(
                "page size must be between 1 and 100".to_string(),
            ));
        }
        Ok(Self { number, size })
    }

    pub const fn number(self) -> i64 {
        self.number
    }

    pub const fn size(self) -> i64 {
        self.size
    }
}

impl Default for NumberedPage {
    fn default() -> Self {
        Self {
            number: 1,
            size: 100,
        }
    }
}

/// Validated offset pagination for APIs using `page[offset]`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct OffsetPage {
    #[serde(rename = "page[size]")]
    size: i32,
    #[serde(rename = "page[offset]", skip_serializing_if = "Option::is_none")]
    offset: Option<i64>,
}

impl OffsetPage {
    pub fn new(size: i32, offset: Option<i64>) -> Result<Self> {
        if !(1..=100).contains(&size) {
            return Err(Error::ConfigError(
                "page size must be between 1 and 100".to_string(),
            ));
        }
        if offset.is_some_and(|value| value < 0) {
            return Err(Error::ConfigError(
                "page offset cannot be negative".to_string(),
            ));
        }
        Ok(Self { size, offset })
    }

    pub const fn size(self) -> i32 {
        self.size
    }

    pub const fn offset(self) -> Option<i64> {
        self.offset
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn numbered_pages_are_validated_and_serialized() {
        assert!(NumberedPage::new(0, 100).is_err());
        assert!(NumberedPage::new(1, 101).is_err());
        let query = serde_json::to_value(NumberedPage::new(2, 50).unwrap()).unwrap();
        assert_eq!(query["page[number]"], 2);
        assert_eq!(query["page[size]"], 50);
    }

    #[test]
    fn offset_pages_are_validated_and_serialized() {
        assert!(OffsetPage::new(0, None).is_err());
        assert!(OffsetPage::new(25, Some(-1)).is_err());
        let query = serde_json::to_value(OffsetPage::new(25, Some(50)).unwrap()).unwrap();
        assert_eq!(query["page[size]"], 25);
        assert_eq!(query["page[offset]"], 50);
    }
}
