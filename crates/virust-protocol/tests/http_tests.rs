use virust_protocol::{HttpRequest, HttpResponse, HttpMethod};
use serde_json::json;

#[test]
fn test_http_request_with_path_params() {
    let req = HttpRequest {
        method: HttpMethod::Get,
        path: vec![("id".to_string(), "123".to_string())].into_iter().collect(),
        query: Default::default(),
        body: None,
    };

    assert_eq!(req.method, HttpMethod::Get);
    assert_eq!(req.path.get("id"), Some(&"123".to_string()));
}

#[test]
fn test_http_response_serialization() {
    let resp = HttpResponse {
        status: 200,
        body: Some(json!({"message": "ok"})),
    };

    assert_eq!(resp.status, 200);
    assert_eq!(resp.body, Some(json!({"message": "ok"})));
}
