use crate::{Result, SsgRoute};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::SystemTime;
use tokio::task::JoinSet;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SsgStats {
    pub pages_built: usize,
    pub pages_failed: usize,
    pub build_time_ms: u64,
    pub routes_with_isr: usize,
}

pub struct SsgBuilder {
    pub routes: Vec<SsgRoute>,
    pub output_dir: PathBuf,
    pub parallel_jobs: usize,
}

impl SsgBuilder {
    pub fn new(output_dir: PathBuf) -> Self {
        Self {
            routes: Vec::new(),
            output_dir,
            parallel_jobs: num_cpus::get(),
        }
    }

    pub async fn build(&self) -> Result<SsgStats> {
        let start = SystemTime::now();

        // Create output directory
        tokio::fs::create_dir_all(&self.output_dir).await?;

        // Build routes in parallel
        let mut join_set = JoinSet::new();
        let semaphore = std::sync::Arc::new(
            tokio::sync::Semaphore::new(self.parallel_jobs)
        );

        for route in &self.routes {
            let route = route.clone();
            let output_dir = self.output_dir.clone();
            let permit = semaphore.clone();

            join_set.spawn(async move {
                let _permit = permit.acquire().await.unwrap();
                Self::build_route(&route, &output_dir).await
            });
        }

        let mut built = 0;
        let mut failed = 0;

        while let Some(result) = join_set.join_next().await {
            match result {
                Ok(Ok(())) => built += 1,
                Ok(Err(e)) => {
                    eprintln!("Build error: {}", e);
                    failed += 1;
                }
                Err(e) => {
                    eprintln!("Task error: {}", e);
                    failed += 1;
                }
            }
        }

        let build_time = start.elapsed()?.as_millis() as u64;
        let routes_with_isr = self.routes.iter()
            .filter(|r| r.revalidate.is_some())
            .count();

        Ok(SsgStats {
            pages_built: built,
            pages_failed: failed,
            build_time_ms: build_time,
            routes_with_isr,
        })
    }

    async fn build_route(route: &SsgRoute, output_dir: &std::path::Path) -> Result<()> {
        // Create output file path
        let route_path = route.path.trim_start_matches('/');
        let file_path = output_dir.join(route_path).join("index.html");

        // Create parent directories
        if let Some(parent) = file_path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        // TODO: Execute route handler and render component
        // For now, create a placeholder
        let html = format!(
            r#"<!DOCTYPE html>
<html>
<head><title>{}</title></head>
<body><h1>SSG Page: {}</h1></body>
</html>"#,
            route.path, route.path
        );

        tokio::fs::write(file_path, html).await?;

        Ok(())
    }
}
