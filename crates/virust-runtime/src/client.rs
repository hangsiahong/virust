use axum::response::Html;

pub async fn serve_client_script() -> Html<&'static str> {
    Html(include_str!("../../virust-bun/bundled/client.js"))
}
