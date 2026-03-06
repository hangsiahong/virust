use std::collections::HashMap;
use std::path::{Path, PathBuf};
use anyhow::Result;
use serde::{Serialize, Deserialize};

const VALID_EXTENSIONS: &[&str] = &["jsx", "js", "tsx", "ts"];

pub struct ComponentRegistry {
    components: HashMap<String, PathBuf>,
}

impl ComponentRegistry {
    pub fn new() -> Self {
        Self {
            components: HashMap::new(),
        }
    }

    pub fn discover(&mut self, web_dir: &Path) -> Result<()> {
        let components_dir = web_dir.join("components");
        if !components_dir.exists() {
            return Ok(());
        }

        self.scan_directory(&components_dir)?;
        Ok(())
    }

    fn scan_directory(&mut self, dir: &Path) -> Result<()> {
        use std::fs;

        let entries = fs::read_dir(dir)?
            .collect::<Result<Vec<_>, _>>()?;

        for entry in entries {
            let file_type = entry.file_type()?;
            let path = entry.path();

            // Only recurse into directories, not symlinks (security fix)
            if file_type.is_dir() && !file_type.is_symlink() {
                self.scan_directory(&path)?;
            } else if file_type.is_file() {
                if let Some(ext) = path.extension() {
                    // Case-insensitive extension matching
                    if ext.to_str()
                        .map(|e| VALID_EXTENSIONS.contains(&e.to_lowercase().as_str()))
                        .unwrap_or(false)
                    {
                        if let Some(name) = self.extract_component_name(&path) {
                            // Check for duplicate component names
                            if self.components.contains_key(&name) {
                                eprintln!(
                                    "Warning: Duplicate component name '{}', skipping {:?}",
                                    name, path
                                );
                                continue;
                            }
                            // Canonicalize to absolute path for Bun renderer
                            let abs_path = fs::canonicalize(&path)
                                .map_err(|e| anyhow::anyhow!("Failed to canonicalize {:?}: {}", path, e))?;
                            self.components.insert(name, abs_path);
                        }
                    }
                }
            }
        }
        Ok(())
    }

    fn extract_component_name(&self, path: &Path) -> Option<String> {
        let stem = path.file_stem()?;
        let name = stem.to_string_lossy();

        // Validate component name: must start with letter and contain only alphanumeric/underscore
        if name.is_empty() || !name.chars().next()?.is_alphabetic() {
            return None;
        }

        if !name.chars().all(|c| c.is_alphanumeric() || c == '_') {
            return None;
        }

        Some(name.to_string())
    }

    pub fn get(&self, name: &str) -> Option<&PathBuf> {
        self.components.get(name)
    }

    pub fn list(&self) -> Vec<&str> {
        self.components.keys().map(|s| s.as_str()).collect()
    }
}

/// Output from server-side rendering a component
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenderedOutput {
    /// Rendered HTML string
    pub html: String,
    /// Serialized props for client hydration
    pub hydration_data: String,
}

impl RenderedOutput {
    /// Create a new RenderedOutput with both HTML and hydration data
    pub fn new(html: String, hydration_data: String) -> Self {
        Self {
            html,
            hydration_data,
        }
    }

    /// Create a RenderedOutput with HTML only (empty hydration data)
    pub fn with_html(html: String) -> Self {
        Self {
            html,
            hydration_data: String::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{self, File};
    use std::io::Write;

    #[test]
    fn test_new_registry() {
        let registry = ComponentRegistry::new();
        assert_eq!(registry.list().len(), 0);
    }

    #[test]
    fn test_discover_components() {
        let temp_dir = tempfile::tempdir().unwrap();
        let components_dir = temp_dir.path().join("components");
        fs::create_dir_all(&components_dir).unwrap();

        // Create test components
        let app_jsx = components_dir.join("App.jsx");
        let mut file = File::create(&app_jsx).unwrap();
        writeln!(file, "export default function App() {{ return <div /> }}").unwrap();

        let like_button = components_dir.join("LikeButton.jsx");
        let mut file = File::create(&like_button).unwrap();
        writeln!(file, "export default function LikeButton() {{ return <button /> }}").unwrap();

        let mut registry = ComponentRegistry::new();
        registry.discover(temp_dir.path()).unwrap();

        assert_eq!(registry.list().len(), 2);
        assert!(registry.get("App").is_some());
        assert!(registry.get("LikeButton").is_some());
        assert!(registry.get("NonExistent").is_none());
    }

    #[test]
    fn test_nested_components() {
        let temp_dir = tempfile::tempdir().unwrap();
        let components_dir = temp_dir.path().join("components");
        let nested_dir = components_dir.join("ui");
        fs::create_dir_all(&nested_dir).unwrap();

        // Create components in nested directory
        let header = nested_dir.join("Header.tsx");
        let mut file = File::create(&header).unwrap();
        writeln!(file, "export default function Header() {{ return <header /> }}").unwrap();

        let footer = nested_dir.join("Footer.tsx");
        let mut file = File::create(&footer).unwrap();
        writeln!(file, "export default function Footer() {{ return <footer /> }}").unwrap();

        let mut registry = ComponentRegistry::new();
        registry.discover(temp_dir.path()).unwrap();

        assert_eq!(registry.list().len(), 2);
        assert!(registry.get("Header").is_some());
        assert!(registry.get("Footer").is_some());
    }

    #[test]
    fn test_ignore_non_component_files() {
        let temp_dir = tempfile::tempdir().unwrap();
        let components_dir = temp_dir.path().join("components");
        fs::create_dir_all(&components_dir).unwrap();

        // Create test component
        let app_jsx = components_dir.join("App.jsx");
        let mut file = File::create(&app_jsx).unwrap();
        writeln!(file, "export default function App() {{ return <div /> }}").unwrap();

        // Create non-component files that should be ignored
        let css_file = components_dir.join("styles.css");
        File::create(&css_file).unwrap();

        let json_file = components_dir.join("config.json");
        File::create(&json_file).unwrap();

        let md_file = components_dir.join("README.md");
        File::create(&md_file).unwrap();

        let mut registry = ComponentRegistry::new();
        registry.discover(temp_dir.path()).unwrap();

        // Should only discover the JSX component
        assert_eq!(registry.list().len(), 1);
        assert!(registry.get("App").is_some());
        assert!(registry.get("styles").is_none());
        assert!(registry.get("config").is_none());
        assert!(registry.get("README").is_none());
    }

    #[test]
    fn test_case_insensitive_extensions() {
        let temp_dir = tempfile::tempdir().unwrap();
        let components_dir = temp_dir.path().join("components");
        fs::create_dir_all(&components_dir).unwrap();

        // Create components with uppercase extensions (different names to avoid duplicates)
        let component_jsx = components_dir.join("ComponentJSX.JSX");
        File::create(&component_jsx).unwrap();

        let component_tsx = components_dir.join("ComponentTSX.TSX");
        File::create(&component_tsx).unwrap();

        let mixed_case = components_dir.join("MixedCase.JsX");
        File::create(&mixed_case).unwrap();

        let mut registry = ComponentRegistry::new();
        registry.discover(temp_dir.path()).unwrap();

        assert_eq!(registry.list().len(), 3);
        assert!(registry.get("ComponentJSX").is_some());
        assert!(registry.get("ComponentTSX").is_some());
        assert!(registry.get("MixedCase").is_some());
    }

    #[test]
    fn test_invalid_component_names() {
        let temp_dir = tempfile::tempdir().unwrap();
        let components_dir = temp_dir.path().join("components");
        fs::create_dir_all(&components_dir).unwrap();

        // Create valid component
        let valid = components_dir.join("Valid.jsx");
        File::create(&valid).unwrap();

        // Create invalid component names
        let starts_with_digit = components_dir.join("123Invalid.jsx");
        File::create(&starts_with_digit).unwrap();

        let special_chars = components_dir.join("Invalid-Name.jsx");
        File::create(&special_chars).unwrap();

        let empty = components_dir.join(".jsx");
        File::create(&empty).unwrap();

        let mut registry = ComponentRegistry::new();
        registry.discover(temp_dir.path()).unwrap();

        // Should only discover the valid component
        assert_eq!(registry.list().len(), 1);
        assert!(registry.get("Valid").is_some());
        assert!(registry.get("123Invalid").is_none());
        assert!(registry.get("Invalid-Name").is_none());
    }

    // RenderedOutput tests
    #[test]
    fn test_new_rendered_output() {
        let html = "<div>Hello World</div>".to_string();
        let hydration_data = r#"{"props":{"name":"World"}}"#.to_string();

        let output = RenderedOutput::new(html.clone(), hydration_data.clone());

        assert_eq!(output.html, html);
        assert_eq!(output.hydration_data, hydration_data);
    }

    #[test]
    fn test_with_html() {
        let html = "<div>Static Content</div>".to_string();

        let output = RenderedOutput::with_html(html.clone());

        assert_eq!(output.html, html);
        assert_eq!(output.hydration_data, "");
    }

    #[test]
    fn test_serialize() {
        let html = "<div>Hello</div>".to_string();
        let hydration_data = r#"{"props":{}}"#.to_string();

        let output = RenderedOutput::new(html.clone(), hydration_data.clone());

        // Test serialization to JSON
        let json = serde_json::to_string(&output).unwrap();
        // JSON serialization will escape the inner quotes in hydration_data
        let expected = r#"{"html":"<div>Hello</div>","hydration_data":"{\"props\":{}}"}"#;
        assert_eq!(json, expected);

        // Test deserialization
        let deserialized: RenderedOutput = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.html, html);
        assert_eq!(deserialized.hydration_data, hydration_data);
    }
}
