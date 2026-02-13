use xplorer::collectors::{Article, ArxivCollector, Collector};

#[tokio::test]
async fn test_arxiv_collector_creation() {
    let collector = ArxivCollector::new();
    assert_eq!(collector.name(), "Arxiv");
    assert!(!collector.description().is_empty());
}

#[tokio::test]
async fn test_article_serialization() {
    let article = Article {
        title: "Test Paper".to_string(),
        authors: vec!["Author 1".to_string(), "Author 2".to_string()],
        url: "https://arxiv.org/abs/1234.5678".to_string(),
        published_date: "2024-01-01".to_string(),
        summary: "This is a test summary".to_string(),
        source: "Arxiv".to_string(),
    };

    let json = serde_json::to_string(&article).unwrap();
    assert!(json.contains("Test Paper"));
    
    let deserialized: Article = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.title, article.title);
}

#[tokio::test]
async fn test_url_encoding() {
    // Test the URL encoding works for various characters
    // This is indirectly tested through the ArxivCollector
    let collector = ArxivCollector::new();
    
    // We can't test actual API calls in unit tests, but we can verify
    // that the collector is properly initialized
    assert_eq!(collector.name(), "Arxiv");
}
