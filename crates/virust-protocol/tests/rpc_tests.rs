use virust_protocol::{RpcRequest, RpcResponse};
use serde_json::json;

#[test]
fn test_rpc_request_serialization() {
    let req = RpcRequest {
        id: 1,
        method: "chat.send".to_string(),
        params: json!({"message": "hello"}),
    };

    let serialized = serde_json::to_string(&req).unwrap();
    let parsed: RpcRequest = serde_json::from_str(&serialized).unwrap();

    assert_eq!(parsed.id, 1);
    assert_eq!(parsed.method, "chat.send");
    assert_eq!(parsed.params, json!({"message": "hello"}));
}

#[test]
fn test_rpc_response_with_result() {
    let resp = RpcResponse {
        id: 1,
        result: Some(json!({"ok": true})),
        error: None,
    };

    let serialized = serde_json::to_string(&resp).unwrap();
    let parsed: RpcResponse = serde_json::from_str(&serialized).unwrap();

    assert_eq!(parsed.id, 1);
    assert_eq!(parsed.result, Some(json!({"ok": true})));
    assert!(parsed.error.is_none());
}

#[test]
fn test_rpc_response_with_error() {
    let resp = RpcResponse {
        id: 1,
        result: None,
        error: Some(virust_protocol::RpcError {
            code: -32001,
            message: "Not found".to_string(),
            details: None,
        }),
    };

    let serialized = serde_json::to_string(&resp).unwrap();
    let parsed: RpcResponse = serde_json::from_str(&serialized).unwrap();

    assert_eq!(parsed.id, 1);
    assert!(parsed.result.is_none());
    assert!(parsed.error.is_some());
}
