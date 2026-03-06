mod discovery;
mod error;

pub use discovery::discover_ssg_routes;
pub use error::{BuildError, Result};

use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};

pub struct SsgBuilder {
    routes: Vec<SsgRoute>,
    output_dir: PathBuf,
    parallel_jobs: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SsgRoute {
    pub path: String,
    pub handler: String,
    pub revalidate: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SsgStats {
    pub pages_built: usize,
    pub pages_failed: usize,
    pub build_time_ms: u64,
    pub routes_with_isr: usize,
}

impl SsgBuilder {
    pub fn new(output_dir: PathBuf) -> Self {
        Self {
            routes: Vec::new(),
            output_dir,
            parallel_jobs: num_cpus::get(),
        }
    }

    pub async fn discover_routes(&mut self, api_dir: &Path) -> Result<()> {
        self.routes = discover_ssg_routes(api_dir)?;
        Ok(())
    }

    pub fn routes(&self) -> &[SsgRoute] {
        &self.routes
    }

    pub fn routes_mut(&mut self) -> &mut Vec<SsgRoute> {
        &mut self.routes
    }
}

/// Metadata trait for SSG routes
///
/// This trait is automatically implemented by the #[ssg] macro for each
/// marked function. It provides compile-time metadata about SSG routes.
pub trait SsgRouteMetadata {
    /// ISR revalidation interval in seconds, if any
    const REVALIDATE: Option<u64>;

    /// The function name (route identifier)
    fn route_path() -> &'static str;
}
