use async_trait::async_trait;
use serde::{Deserialize, Serialize};

pub mod arxiv;
pub mod example;

pub use arxiv::ArxivCollector;
pub use example::ExampleArticleCollector;

/// Represents a collected article/paper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Article {
    pub title: String,
    pub authors: Vec<String>,
    pub url: String,
    pub published_date: String,
    pub summary: String,
    pub source: String,
}

/// Result type for collection operations
pub type CollectorResult<T> = Result<T, Box<dyn std::error::Error + Send + Sync>>;

/// Trait for content collectors
/// This abstracts the collection of articles/papers from various sources
#[async_trait]
pub trait Collector: Send + Sync {
    /// Returns the name of this collector
    fn name(&self) -> &str;

    /// Collects articles based on a query
    async fn collect(&self, query: &str, max_results: usize) -> CollectorResult<Vec<Article>>;

    /// Returns a description of what this collector does
    fn description(&self) -> &str;
}
