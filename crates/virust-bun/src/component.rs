use std::collections::HashMap;
use std::path::{Path, PathBuf};
use anyhow::Result;

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

        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                self.scan_directory(&path)?;
            } else {
                if let Some(ext) = path.extension() {
                    if ext == "jsx" || ext == "js" || ext == "tsx" || ext == "ts" {
                        if let Some(name) = self.extract_component_name(&path) {
                            self.components.insert(name, path);
                        }
                    }
                }
            }
        }
        Ok(())
    }

    fn extract_component_name(&self, path: &Path) -> Option<String> {
        path.file_stem()
            .map(|s| s.to_string_lossy().to_string())
    }

    pub fn get(&self, name: &str) -> Option<&PathBuf> {
        self.components.get(name)
    }

    pub fn list(&self) -> Vec<&str> {
        self.components.keys().map(|s| s.as_str()).collect()
    }
}

// Placeholder for Task 3
pub struct RenderedOutput;

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
}
