use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::SystemTime;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;

/// Represents a change to a component file
#[derive(Debug, Clone, PartialEq)]
pub struct ComponentChange {
    /// Name of the component (file stem without extension)
    pub component_name: String,
    /// Path to the component file
    pub path: PathBuf,
    /// Type of change that occurred
    pub change_type: ChangeType,
}

/// The type of change that occurred to a component
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChangeType {
    /// File was modified
    Modified,
    /// File was created
    Created,
    /// File was deleted
    Deleted,
}

/// Watch for component file changes in the given directory
///
/// This function spawns a background task that polls the directory every 500ms
/// for changes to .jsx, .js, .tsx, and .ts files.
///
/// # Arguments
/// * `components_dir` - Path to the components directory to watch
/// * `tx` - Channel to send component change events
///
/// # Returns
/// A JoinHandle for the watcher task
pub fn watch_components(
    components_dir: PathBuf,
    tx: mpsc::Sender<ComponentChange>,
) -> JoinHandle<()> {
    tokio::spawn(async move {
        let mut last_state: HashMap<PathBuf, SystemTime> = HashMap::new();
        let mut initialized = false;

        loop {
            // Scan for current files
            let current_files = scan_directory(&components_dir);

            // Track current files for change detection
            let mut current_state: HashMap<PathBuf, SystemTime> = HashMap::new();

            for path in &current_files {
                if let Ok(metadata) = std::fs::metadata(&path) {
                    if let Ok(modified) = metadata.modified() {
                        current_state.insert(path.clone(), modified);

                        // Only check for changes after initialization
                        if initialized {
                            // Check if this is a new file or modified
                            if let Some(&last_modified) = last_state.get(path) {
                                // File existed before - check if modified
                                if modified != last_modified {
                                    let component_name = extract_component_name(&path);
                                    let change = ComponentChange {
                                        component_name,
                                        path: path.clone(),
                                        change_type: ChangeType::Modified,
                                    };
                                    let _ = tx.send(change).await;
                                }
                            } else {
                                // New file
                                let component_name = extract_component_name(&path);
                                let change = ComponentChange {
                                    component_name,
                                    path: path.clone(),
                                    change_type: ChangeType::Created,
                                };
                                let _ = tx.send(change).await;
                            }
                        }
                    }
                }
            }

            // Only check for deleted files after initialization
            if initialized {
                // Check for deleted files
                for path in last_state.keys() {
                    if !current_state.contains_key(path) {
                        let component_name = extract_component_name(path);
                        let change = ComponentChange {
                            component_name,
                            path: path.clone(),
                            change_type: ChangeType::Deleted,
                        };
                        let _ = tx.send(change).await;
                    }
                }
            }

            // Update last state
            last_state = current_state;
            initialized = true;

            // Wait before next poll
            tokio::time::sleep(std::time::Duration::from_millis(500)).await;
        }
    })
}

/// Recursively scan a directory for component files
///
/// # Arguments
/// * `dir` - Directory to scan
///
/// # Returns
/// Vector of paths to component files (.jsx, .js, .tsx, .ts)
fn scan_directory(dir: &Path) -> Vec<PathBuf> {
    let mut component_files = Vec::new();

    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();

            if path.is_dir() {
                // Recursively scan subdirectories
                component_files.extend(scan_directory(&path));
            } else if is_component_file(&path) {
                component_files.push(path);
            }
        }
    }

    component_files
}

/// Check if a file is a component file based on its extension
fn is_component_file(path: &Path) -> bool {
    if let Some(extension) = path.extension() {
        let ext = extension.to_string_lossy().to_lowercase();
        matches!(ext.as_str(), "jsx" | "js" | "tsx" | "ts")
    } else {
        false
    }
}

/// Extract the component name from a file path
///
/// The component name is the file stem (filename without extension)
fn extract_component_name(path: &Path) -> String {
    path.file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("unknown")
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs::File;
    use std::io::Write;
    use std::thread;
    use std::time::Duration;

    #[tokio::test]
    async fn test_watch_components_detects_new_file() {
        // Create temporary directory
        let temp_dir = TempDir::new().unwrap();
        let components_dir = temp_dir.path().join("components");
        std::fs::create_dir(&components_dir).unwrap();

        // Create channel for changes
        let (tx, mut rx) = mpsc::channel(100);

        // Start watcher
        let components_dir_clone = components_dir.clone();
        watch_components(components_dir_clone, tx);

        // Give watcher time to start
        tokio::time::sleep(Duration::from_millis(100)).await;

        // Create a test file
        let test_file = components_dir.join("Test.jsx");
        let mut file = File::create(&test_file).unwrap();
        writeln!(file, "export default function Test() {{ return <div />; }}").unwrap();

        // Wait for change to be detected
        tokio::time::sleep(Duration::from_millis(600)).await;

        // Verify change was detected
        let change = rx.recv().await.unwrap();
        assert_eq!(change.component_name, "Test");
        assert_eq!(change.path, test_file);
        assert_eq!(change.change_type, ChangeType::Created);
    }

    #[tokio::test]
    async fn test_watch_components_detects_modified_file() {
        // Create temporary directory
        let temp_dir = TempDir::new().unwrap();
        let components_dir = temp_dir.path().join("components");
        std::fs::create_dir(&components_dir).unwrap();

        // Create initial file
        let test_file = components_dir.join("Test.jsx");
        {
            let mut file = File::create(&test_file).unwrap();
            writeln!(file, "export default function Test() {{ return <div />; }}").unwrap();
        }

        // Give the file a distinct initial modification time
        thread::sleep(Duration::from_millis(100));

        // Create channel for changes
        let (tx, mut rx) = mpsc::channel(100);

        // Start watcher
        let components_dir_clone = components_dir.clone();
        watch_components(components_dir_clone, tx);

        // Give watcher time to scan initial state
        tokio::time::sleep(Duration::from_millis(600)).await;

        // Modify the file
        thread::sleep(Duration::from_millis(100));
        {
            let mut file = File::create(&test_file).unwrap();
            writeln!(file, "export default function Test() {{ return <span />; }}").unwrap();
        }

        // Wait for change to be detected
        tokio::time::sleep(Duration::from_millis(600)).await;

        // Verify change was detected
        let change = rx.recv().await.unwrap();
        assert_eq!(change.component_name, "Test");
        assert_eq!(change.path, test_file);
        assert_eq!(change.change_type, ChangeType::Modified);
    }

    #[tokio::test]
    async fn test_watch_components_detects_deleted_file() {
        // Create temporary directory
        let temp_dir = TempDir::new().unwrap();
        let components_dir = temp_dir.path().join("components");
        std::fs::create_dir(&components_dir).unwrap();

        // Create initial file
        let test_file = components_dir.join("Test.jsx");
        {
            let mut file = File::create(&test_file).unwrap();
            writeln!(file, "export default function Test() {{ return <div />; }}").unwrap();
        }

        // Create channel for changes
        let (tx, mut rx) = mpsc::channel(100);

        // Start watcher
        let components_dir_clone = components_dir.clone();
        watch_components(components_dir_clone, tx);

        // Give watcher time to scan initial state
        tokio::time::sleep(Duration::from_millis(600)).await;

        // Delete the file
        std::fs::remove_file(&test_file).unwrap();

        // Wait for change to be detected
        tokio::time::sleep(Duration::from_millis(600)).await;

        // Verify change was detected
        let change = rx.recv().await.unwrap();
        assert_eq!(change.component_name, "Test");
        assert_eq!(change.path, test_file);
        assert_eq!(change.change_type, ChangeType::Deleted);
    }

    #[test]
    fn test_scan_directory_filters_component_files() {
        let temp_dir = TempDir::new().unwrap();
        let components_dir = temp_dir.path().join("components");
        std::fs::create_dir(&components_dir).unwrap();

        // Create various files
        File::create(components_dir.join("Test.jsx")).unwrap();
        File::create(components_dir.join("App.tsx")).unwrap();
        File::create(components_dir.join("utils.ts")).unwrap();
        File::create(components_dir.join("legacy.js")).unwrap();
        File::create(components_dir.join("data.json")).unwrap();
        File::create(components_dir.join("styles.css")).unwrap();

        // Scan directory
        let files = scan_directory(&components_dir);

        // Should only find component files
        assert_eq!(files.len(), 4);
        let file_names: Vec<String> = files
            .iter()
            .filter_map(|p| p.file_name().and_then(|s| s.to_str()))
            .map(String::from)
            .collect();

        assert!(file_names.contains(&"Test.jsx".to_string()));
        assert!(file_names.contains(&"App.tsx".to_string()));
        assert!(file_names.contains(&"utils.ts".to_string()));
        assert!(file_names.contains(&"legacy.js".to_string()));
        assert!(!file_names.contains(&"data.json".to_string()));
        assert!(!file_names.contains(&"styles.css".to_string()));
    }

    #[test]
    fn test_scan_directory_recursive() {
        let temp_dir = TempDir::new().unwrap();
        let components_dir = temp_dir.path().join("components");
        std::fs::create_dir(&components_dir).unwrap();

        // Create nested directory structure
        let nested_dir = components_dir.join("ui");
        std::fs::create_dir(&nested_dir).unwrap();

        File::create(components_dir.join("App.jsx")).unwrap();
        File::create(nested_dir.join("Button.jsx")).unwrap();

        // Scan directory
        let files = scan_directory(&components_dir);

        // Should find both files
        assert_eq!(files.len(), 2);
        let file_names: Vec<String> = files
            .iter()
            .filter_map(|p| p.file_name().and_then(|s| s.to_str()))
            .map(String::from)
            .collect();

        assert!(file_names.contains(&"App.jsx".to_string()));
        assert!(file_names.contains(&"Button.jsx".to_string()));
    }

    #[test]
    fn test_extract_component_name() {
        let path = PathBuf::from("/components/Test.jsx");
        assert_eq!(extract_component_name(&path), "Test");

        let path = PathBuf::from("/components/ui/Button.tsx");
        assert_eq!(extract_component_name(&path), "Button");

        let path = PathBuf::from("/components/utils.js");
        assert_eq!(extract_component_name(&path), "utils");
    }

    #[test]
    fn test_is_component_file() {
        assert!(is_component_file(&PathBuf::from("/test.jsx")));
        assert!(is_component_file(&PathBuf::from("/test.js")));
        assert!(is_component_file(&PathBuf::from("/test.tsx")));
        assert!(is_component_file(&PathBuf::from("/test.ts")));
        assert!(!is_component_file(&PathBuf::from("/test.json")));
        assert!(!is_component_file(&PathBuf::from("/test.css")));
        assert!(!is_component_file(&PathBuf::from("/test.md")));
    }

    #[test]
    fn test_component_change_equality() {
        let change1 = ComponentChange {
            component_name: "Test".to_string(),
            path: PathBuf::from("/components/Test.jsx"),
            change_type: ChangeType::Modified,
        };

        let change2 = ComponentChange {
            component_name: "Test".to_string(),
            path: PathBuf::from("/components/Test.jsx"),
            change_type: ChangeType::Modified,
        };

        assert_eq!(change1, change2);
    }

    #[test]
    fn test_change_type_equality() {
        assert_eq!(ChangeType::Modified, ChangeType::Modified);
        assert_eq!(ChangeType::Created, ChangeType::Created);
        assert_eq!(ChangeType::Deleted, ChangeType::Deleted);
        assert_ne!(ChangeType::Modified, ChangeType::Created);
    }
}
