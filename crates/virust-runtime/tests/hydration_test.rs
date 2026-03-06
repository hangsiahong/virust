#[cfg(test)]
mod hydration_tests {
    use virust_runtime::RenderedHtml;

    #[tokio::test]
    async fn test_hydration_script_included() {
        let html = RenderedHtml::new("App").render().await.unwrap();
        assert!(html.contains("/bun/client.js"));
        assert!(html.contains("__VIRUST_PROPS__"));
    }
}
