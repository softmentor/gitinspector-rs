use regex::Regex;

#[derive(Debug, Clone)]
pub enum FilterType {
    Author,
    Email,
    Revision,
    Message,
    File,
    General,
}

#[derive(Debug, Clone)]
struct TypedRegex {
    re: Regex,
    filter_type: FilterType,
}

/// Handles filtering logic based on exclude patterns.
#[derive(Debug, Clone)]
pub struct Filter {
    typed_excludes: Vec<TypedRegex>,
}

impl Filter {
    /// Creates a new `Filter` with a set of exclude patterns.
    /// Patterns can be prefixed with `author:`, `email:`, `revision:`, `message:`, `file:`.
    pub fn new(patterns: &[String]) -> Result<Self, regex::Error> {
        let mut typed_excludes = Vec::new();
        for pattern in patterns {
            let (filter_type, raw_pattern) = if let Some(stripped) = pattern.strip_prefix("author:") {
                (FilterType::Author, stripped)
            } else if let Some(stripped) = pattern.strip_prefix("email:") {
                (FilterType::Email, stripped)
            } else if let Some(stripped) = pattern.strip_prefix("revision:") {
                (FilterType::Revision, stripped)
            } else if let Some(stripped) = pattern.strip_prefix("message:") {
                (FilterType::Message, stripped)
            } else if let Some(stripped) = pattern.strip_prefix("file:") {
                (FilterType::File, stripped)
            } else {
                (FilterType::General, pattern.as_str())
            };

            typed_excludes.push(TypedRegex {
                re: Regex::new(raw_pattern)?,
                filter_type,
            });
        }
        Ok(Self { typed_excludes })
    }

    /// Returns `true` if the given text should be excluded based on the filter type.
    pub fn is_excluded(&self, text: &str, filter_type: FilterType) -> bool {
        self.typed_excludes.iter().any(|tr| {
            match (&tr.filter_type, &filter_type) {
                (FilterType::General, _) => tr.re.is_match(text),
                (t1, t2) if std::mem::discriminant(t1) == std::mem::discriminant(t2) => tr.re.is_match(text),
                _ => false,
            }
        })
    }

    /// Legacy support for file filtering.
    pub fn should_exclude(&self, file_path: &str) -> bool {
        self.is_excluded(file_path, FilterType::File) || self.is_excluded(file_path, FilterType::General)
    }
}

impl Default for Filter {
    fn default() -> Self {
        Self { typed_excludes: Vec::new() }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_typed_filter() {
        let filter = Filter::new(&[
            "author:John".to_string(),
            "email:gmail\\.com$".to_string(),
            "message:BUGFIX".to_string(),
            "file:tests/".to_string(),
        ]).unwrap();
        
        assert!(filter.is_excluded("John Doe", FilterType::Author));
        assert!(filter.is_excluded("user@gmail.com", FilterType::Email));
        assert!(filter.is_excluded("Fixed a BUGFIX in core", FilterType::Message));
        assert!(filter.is_excluded("tests/main.rs", FilterType::File));
        
        assert!(!filter.is_excluded("Jane Doe", FilterType::Author));
        assert!(!filter.is_excluded("src/main.rs", FilterType::File));
    }
}
