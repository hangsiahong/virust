use virust_build::{SsgBuilder, SsgRoute};
use std::path::PathBuf;

#[tokio::test]
async fn test_ssg_builder() {
    let output = PathBuf::from("/tmp/test_ssg_builder");
    let _ = std::fs::remove_dir_all(&output);

    let mut builder = SsgBuilder::new(output.clone());
    builder.routes.push(SsgRoute {
        path: "/test".to_string(),
        handler: "test_handler".to_string(),
        revalidate: Some(60),
    });

    let stats = builder.build().await.unwrap();

    assert_eq!(stats.pages_built, 1);
    assert!(output.join("test/index.html").exists());
}
