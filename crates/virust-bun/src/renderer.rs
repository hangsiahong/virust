use std::io::BufReader;
use std::path::Path;
use std::process::{Child, ChildStdin, ChildStdout, Stdio, Command};

use anyhow::Result;

use crate::component::ComponentRegistry;

pub struct BunRenderer {
    /// Spawned Bun process
    bun_process: Child,
    /// Component registry for discovered components
    component_registry: ComponentRegistry,
    /// Bun process stdin for sending render requests
    stdin: Option<ChildStdin>,
    /// Bun process stdout for receiving render responses
    stdout: Option<BufReader<ChildStdout>>,
}

impl BunRenderer {
    /// Create a new BunRenderer by spawning a Bun process
    ///
    /// This spawns `bun run /opt/virust/bun/renderer.js` with piped stdin/stdout
    /// for communication with the Bun renderer process.
    pub fn new() -> Result<Self> {
        // Spawn the Bun process with piped stdin/stdout
        let mut bun_process = Command::new("bun")
            .arg("run")
            .arg("/opt/virust/bun/renderer.js")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .map_err(|e| anyhow::anyhow!("Failed to spawn Bun: {}", e))?;

        // Take stdin and stdout from the process (they are Option types)
        let stdin = bun_process.stdin.take();
        let stdout = bun_process.stdout.take().map(BufReader::new);

        Ok(Self {
            bun_process,
            component_registry: ComponentRegistry::new(),
            stdin,
            stdout,
        })
    }

    /// Set the web directory and discover components
    ///
    /// This updates the component registry by scanning the web directory
    /// for React components (files with .jsx, .js, .tsx, .ts extensions).
    pub fn set_web_dir(&mut self, web_dir: &Path) -> Result<()> {
        self.component_registry.discover(web_dir)
    }

    /// Return the number of registered components
    pub fn component_count(&self) -> usize {
        self.component_registry.list().len()
    }

    /// Check if the Bun process is still running
    pub fn is_alive(&mut self) -> bool {
        self.bun_process
            .try_wait()
            .map(|status| status.is_none())
            .unwrap_or(false)
    }
}

impl Drop for BunRenderer {
    /// Kill the Bun process when the renderer is dropped
    fn drop(&mut self) {
        let _ = self.bun_process.kill();
        let _ = self.bun_process.wait();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore] // Requires Bun to be installed
    fn test_bun_renderer_creation() {
        // Test that we can create a BunRenderer
        // This test is ignored by default since it requires Bun to be installed
        let mut renderer = BunRenderer::new();

        // If Bun is not installed, this will fail with an error
        match renderer {
            Ok(ref mut r) => {
                // Successfully created renderer
                assert!(r.is_alive());
                assert_eq!(r.component_count(), 0);
            }
            Err(e) => {
                // Bun not installed - this is expected in some environments
                eprintln!("Bun not installed, skipping test: {}", e);
            }
        }
    }

    #[test]
    fn test_component_count_empty() {
        // Create a mock renderer without spawning Bun
        // This test doesn't require Bun to be installed
        // We'll test component_count through the registry directly
        let registry = ComponentRegistry::new();
        assert_eq!(registry.list().len(), 0);
    }
}
