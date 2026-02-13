use async_trait::async_trait;

use super::{Article, Collector, CollectorResult};

/// Example collector for general article/news sites
/// This is a mock implementation that demonstrates how to add more collectors
pub struct ExampleArticleCollector {
    #[allow(dead_code)]
    client: reqwest::Client,
}

impl ExampleArticleCollector {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
        }
    }
}

#[async_trait]
impl Collector for ExampleArticleCollector {
    fn name(&self) -> &str {
        "Example Articles"
    }

    fn description(&self) -> &str {
        "Example collector for article sites (placeholder implementation)"
    }

    async fn collect(&self, query: &str, max_results: usize) -> CollectorResult<Vec<Article>> {
        tracing::info!(
            "ExampleArticleCollector called with query: {}, max_results: {}",
            query,
            max_results
        );

        // This is a placeholder implementation
        // In a real implementation, you would:
        // 1. Make HTTP requests to article sites/APIs
        // 2. Parse the HTML or JSON responses
        // 3. Extract article information
        // 4. Convert to Article structs

        // For demonstration, return an empty list
        // You can replace this with actual API calls to sites like:
        // - Medium API
        // - Dev.to API
        // - Hacker News API
        // - Research paper databases

        Ok(vec![])
    }
}
