use std::io::{BufRead, BufReader, Write};
use std::path::Path;
use std::process::{Child, ChildStdin, ChildStdout, Stdio, Command};

use anyhow::Result;
use serde_json::Value;

use crate::component::{ComponentRegistry, RenderedOutput};

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

    /// Send a ping request to verify the Bun process is responsive
    pub async fn ping(&mut self) -> Result<()> {
        let request = serde_json::json!({
            "type": "ping"
        });

        self.send_request(&request)?;
        let response = self.receive_response()?;

        if response.get("pong").is_some() {
            Ok(())
        } else {
            Err(anyhow::anyhow!("Invalid ping response"))
        }
    }

    fn send_request(&mut self, request: &Value) -> Result<()> {
        let stdin = self.stdin.as_mut().ok_or_else(|| anyhow::anyhow!("stdin not available"))?;

        let request_str = serde_json::to_string(request)?;
        writeln!(stdin, "{}", request_str)?;
        stdin.flush()?;

        Ok(())
    }

    fn receive_response(&mut self) -> Result<Value> {
        let stdout = self.stdout.as_mut().ok_or_else(|| anyhow::anyhow!("stdout not available"))?;

        let mut line = String::new();
        stdout.read_line(&mut line)?;

        if line.is_empty() {
            return Err(anyhow::anyhow!("Empty response from Bun"));
        }

        let response: Value = serde_json::from_str(&line.trim())?;
        Ok(response)
    }

    pub async fn render_component(&mut self, name: &str, props: Value) -> Result<RenderedOutput> {
        // Find component path
        let component_path = self.component_registry.get(name)
            .ok_or_else(|| anyhow::anyhow!("Component not found: {}", name))?;

        let request = serde_json::json!({
            "type": "render",
            "component": component_path.to_string_lossy(),
            "props": props
        });

        self.send_request(&request)?;
        let response = self.receive_response()?;

        if let Some(error) = response.get("error") {
            return Err(anyhow::anyhow!("Component render error: {}", error));
        }

        let html = response["html"].as_str()
            .ok_or_else(|| anyhow::anyhow!("Response missing 'html' field"))?
            .to_string();

        let hydration_data = response["hydrationData"].as_str()
            .ok_or_else(|| anyhow::anyhow!("Response missing 'hydrationData' field"))?
            .to_string();

        Ok(RenderedOutput::new(html, hydration_data))
    }
}

impl Drop for BunRenderer {
    /// Kill the Bun process when the renderer is dropped
    fn drop(&mut self) {
        let _ = self.bun_process.kill();
        let _ = self.bun_process.wait();
    }
}

/// Supervisor for managing the Bun renderer process
///
/// The supervisor ensures that a Bun renderer is always available,
/// automatically restarting it if it crashes.
pub struct BunSupervisor {
    renderer: Option<BunRenderer>,
}

impl BunSupervisor {
    /// Create a new supervisor with no active renderer
    pub fn new() -> Self {
        Self { renderer: None }
    }

    /// Ensure a Bun renderer is running, creating a new one if needed
    ///
    /// This method checks if the current renderer is alive and creates a new one
    /// if it isn't. This provides automatic restart capability.
    pub async fn ensure_running(&mut self) -> Result<&mut BunRenderer> {
        if self.renderer.is_none() {
            self.renderer = Some(BunRenderer::new()?);
        } else if !self.renderer.as_mut().unwrap().is_alive() {
            self.renderer = Some(BunRenderer::new()?);
        }
        Ok(self.renderer.as_mut().unwrap())
    }

    /// Shutdown the supervisor and clean up the renderer
    pub fn shutdown(mut self) -> Result<()> {
        if let Some(renderer) = self.renderer.take() {
            drop(renderer);
        }
        Ok(())
    }
}

impl Default for BunSupervisor {
    fn default() -> Self {
        Self::new()
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

#[cfg(test)]
mod supervisor_tests {
    use super::*;

    #[tokio::test]
    #[ignore]
    async fn test_supervisor_ensures_running() {
        let mut supervisor = BunSupervisor::new();

        let renderer = supervisor.ensure_running().await.unwrap();
        assert!(renderer.is_alive());

        let renderer2 = supervisor.ensure_running().await.unwrap();
        assert!(renderer2.is_alive());
    }
}
