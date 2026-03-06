mod error;

pub use error::{BuildError, Result};

use std::path::PathBuf;
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
}
