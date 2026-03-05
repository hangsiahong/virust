use virust_typescript::generate_typescript;

#[test]
fn test_generate_interface() {
    let ts = generate_typescript(
        "chat",
        "ChatMessage",
        r#"{ message: string }"#,
        "ChatResponse",
        r#"{ ok: boolean }"#,
    );

    assert!(ts.contains("export interface ChatMessage"));
    assert!(ts.contains("message: string"));
    assert!(ts.contains("export interface ChatResponse"));
    assert!(ts.contains("ok: boolean"));
}