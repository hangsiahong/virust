use virust_macros::render_component;
use virust_runtime::RenderedHtml;
use serde::Serialize;

#[derive(Serialize)]
struct SearchData {
    query: String,
    filters: Vec<String>,
}

#[render_component("App")]
async fn index() -> RenderedHtml {
    RenderedHtml::new("App")
}

#[render_component("BlogPost")]
async fn blog_post(#[path] id: String) -> RenderedHtml {
    RenderedHtml::with_props("BlogPost", serde_json::json!({"id": id}))
}

#[render_component("SearchResults")]
async fn search(#[path] query: String, #[body] data: SearchData) -> RenderedHtml {
    RenderedHtml::with_props("SearchResults", serde_json::json!({
        "query": query,
        "data": data
    }))
}

fn main() {
    println!("render_component macro test compiled successfully!");
}
