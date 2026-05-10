use gitinspector_core::blame::compute_blame;
use gitinspector_core::config::Config;
use gitinspector_core::filtering::Filter;
use gitinspector_core::provider::GitProvider;
use std::sync::Arc;
use async_trait::async_trait;
use futures::Stream;
use std::pin::Pin;

struct MockBlameProvider;

#[async_trait]
impl GitProvider for MockBlameProvider {
    async fn get_commits(&self, _config: &Config) -> Result<Pin<Box<dyn Stream<Item = Result<String, String>> + Send>>, String> {
        Err("Not implemented".into())
    }
    async fn get_tracked_files(&self, _config: &Config) -> Result<Vec<String>, String> {
        Ok((0..5000).map(|i| format!("file_{}.rs", i)).collect())
    }
    async fn get_blame_file(&self, _config: &Config, _file_path: &str) -> Result<String, String> {
        Ok("author Name\nauthor-mail <name@example.com>\n\tline1".repeat(10))
    }
    async fn get_branches(&self, _config: &Config) -> Result<Vec<String>, String> {
        Ok(vec!["main".into()])
    }
}

#[tokio::test]
async fn test_blame_concurrency_scaling() {
    let provider = Arc::new(MockBlameProvider);
    let config = Arc::new(Config::default());
    let filter = Arc::new(Filter::new(&[]).unwrap());
    let files = provider.get_tracked_files(&config).await.unwrap();

    let start = std::time::Instant::now();
    let result = compute_blame(provider, config, filter, files).await.unwrap();
    let duration = start.elapsed();

    println!("Processed 5,000 files in {:?}", duration);
    assert!(!result.is_empty());
    assert!(duration.as_secs() < 10, "Should be fast with concurrent processing");
}
