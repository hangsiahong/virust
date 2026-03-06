use virust_macros::{get, ssg};
use serde::Serialize;

#[derive(Serialize)]
pub struct HomePageData {
    pub title: String,
    pub subtitle: String,
    pub features: Vec<String>,
}

/// Home page data (SSG - generated at build time)
#[get]
#[ssg]
async fn route() -> String {
    let data = HomePageData {
        title: "Welcome to {{project_name}}".to_string(),
        subtitle: "A full-stack application built with Virust".to_string(),
        features: vec![
            "Static Site Generation".to_string(),
            "Incremental Static Regeneration".to_string(),
            "Server-Side Rendering".to_string(),
            "Intelligent Caching".to_string(),
            "TypeScript + Tailwind".to_string(),
        ],
    };

    serde_json::to_string(&data).unwrap_or_else(|_| "{}".to_string())
}
