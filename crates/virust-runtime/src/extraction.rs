//! Parameter extraction utilities for path and body parameters
//!
//! This module provides the infrastructure for extracting parameters from
//! HTTP requests and injecting them into handler functions.

use std::collections::HashMap;

/// Metadata for path parameters
#[derive(Debug, Clone, Copy)]
pub struct PathParamMetadata {
    pub handler_name: &'static str,
    pub param_name: &'static str,
}

inventory::collect!(PathParamMetadata);

/// Metadata for body parameters
#[derive(Debug, Clone, Copy)]
pub struct BodyParamMetadata {
    pub handler_name: &'static str,
    pub param_name: &'static str,
}

inventory::collect!(BodyParamMetadata);

/// Extracted parameters from an HTTP request
#[derive(Debug, Clone, Default)]
pub struct ExtractedParams {
    /// Path parameters extracted from URL (e.g., /users/:id)
    pub path_params: HashMap<String, String>,
    /// Body parameters parsed from JSON payload
    pub body_params: Option<serde_json::Value>,
}

impl ExtractedParams {
    /// Create a new empty parameter set
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a path parameter
    pub fn add_path_param(&mut self, name: String, value: String) {
        self.path_params.insert(name, value);
    }

    /// Get a path parameter by name
    pub fn get_path_param(&self, name: &str) -> Option<&String> {
        self.path_params.get(name)
    }

    /// Set the body parameters
    pub fn set_body_params(&mut self, params: serde_json::Value) {
        self.body_params = Some(params);
    }

    /// Get the body parameters
    pub fn get_body_params(&self) -> Option<&serde_json::Value> {
        self.body_params.as_ref()
    }
}

/// Extractor for path parameters from URL patterns
///
/// # Example
///
/// ```
/// use virust_runtime::extraction::PathExtractor;
///
/// let extractor = PathExtractor::new("/users/:id");
/// let params = extractor.extract("/users/123");
/// assert_eq!(params.get_path_param("id"), Some(&"123".to_string()));
/// ```
#[derive(Debug, Clone)]
pub struct PathExtractor {
    pattern: String,
    param_names: Vec<String>,
}

impl PathExtractor {
    /// Create a new path extractor from a URL pattern
    ///
    /// # Arguments
    ///
    /// * `pattern` - URL pattern with :param placeholders (e.g., "/users/:id")
    pub fn new(pattern: &str) -> Self {
        let param_names = Self::extract_param_names(pattern);
        Self {
            pattern: pattern.to_string(),
            param_names,
        }
    }

    /// Extract parameter names from a URL pattern
    fn extract_param_names(pattern: &str) -> Vec<String> {
        pattern
            .split('/')
            .filter_map(|segment| {
                if segment.starts_with(':') {
                    Some(segment[1..].to_string())
                } else {
                    None
                }
            })
            .collect()
    }

    /// Extract path parameters from a concrete URL
    ///
    /// # Arguments
    ///
    /// * `url` - The actual URL to extract parameters from
    pub fn extract(&self, url: &str) -> ExtractedParams {
        let mut params = ExtractedParams::new();

        let pattern_segments: Vec<&str> = self.pattern.split('/').collect();
        let url_segments: Vec<&str> = url.split('/').collect();

        for (i, pattern_seg) in pattern_segments.iter().enumerate() {
            if i < url_segments.len() {
                if pattern_seg.starts_with(':') {
                    let param_name = &pattern_seg[1..];
                    params.add_path_param(param_name.to_string(), url_segments[i].to_string());
                }
            }
        }

        params
    }

    /// Check if a URL matches this pattern
    pub fn matches(&self, url: &str) -> bool {
        let pattern_segments: Vec<&str> = self.pattern.split('/').collect();
        let url_segments: Vec<&str> = url.split('/').collect();

        if pattern_segments.len() != url_segments.len() {
            return false;
        }

        pattern_segments
            .iter()
            .zip(url_segments.iter())
            .all(|(pattern, url)| pattern.starts_with(':') || pattern == url)
    }

    /// Get the parameter names for this pattern
    pub fn param_names(&self) -> &[String] {
        &self.param_names
    }
}

/// Extractor for JSON body parameters
///
/// # Example
///
/// ```
/// use virust_runtime::extraction::BodyExtractor;
/// use serde_json::json;
///
/// let extractor = BodyExtractor::new();
/// let params = extractor.extract(&json!({"name": "John"}));
/// assert!(params.get_body_params().is_some());
/// ```
#[derive(Debug, Clone, Default)]
pub struct BodyExtractor;

impl BodyExtractor {
    /// Create a new body extractor
    pub fn new() -> Self {
        Self
    }

    /// Extract body parameters from a JSON value
    ///
    /// # Arguments
    ///
    /// * `body` - The JSON body to extract parameters from
    pub fn extract(&self, body: &serde_json::Value) -> ExtractedParams {
        let mut params = ExtractedParams::new();
        params.set_body_params(body.clone());
        params
    }
}

/// Get all registered path parameter metadata
pub fn get_path_param_metadata() -> Vec<PathParamMetadata> {
    inventory::iter::<PathParamMetadata>.into_iter().copied().collect()
}

/// Get all registered body parameter metadata
pub fn get_body_param_metadata() -> Vec<BodyParamMetadata> {
    inventory::iter::<BodyParamMetadata>.into_iter().copied().collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_path_extractor_simple() {
        let extractor = PathExtractor::new("/users/:id");
        let params = extractor.extract("/users/123");

        assert_eq!(params.get_path_param("id"), Some(&"123".to_string()));
    }

    #[test]
    fn test_path_extractor_multiple_params() {
        let extractor = PathExtractor::new("/users/:userId/posts/:postId");
        let params = extractor.extract("/users/42/posts/99");

        assert_eq!(
            params.get_path_param("userId"),
            Some(&"42".to_string())
        );
        assert_eq!(
            params.get_path_param("postId"),
            Some(&"99".to_string())
        );
    }

    #[test]
    fn test_path_extractor_matches() {
        let extractor = PathExtractor::new("/users/:id");

        assert!(extractor.matches("/users/123"));
        assert!(extractor.matches("/users/abc"));
        assert!(!extractor.matches("/users/123/extra"));
        assert!(!extractor.matches("/posts/123"));
    }

    #[test]
    fn test_body_extractor() {
        let extractor = BodyExtractor::new();
        let body = serde_json::json!({
            "name": "John",
            "age": 30
        });

        let params = extractor.extract(&body);
        let extracted = params.get_body_params().unwrap();

        assert_eq!(extracted["name"], "John");
        assert_eq!(extracted["age"], 30);
    }

    #[test]
    fn test_extracted_params() {
        let mut params = ExtractedParams::new();
        params.add_path_param("id".to_string(), "123".to_string());
        params.set_body_params(serde_json::json!({"name": "test"}));

        assert_eq!(params.get_path_param("id"), Some(&"123".to_string()));
        assert!(params.get_body_params().is_some());
    }
}
