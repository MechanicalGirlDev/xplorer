use async_trait::async_trait;
use quick_xml::de::from_str;
use serde::Deserialize;

use super::{Article, Collector, CollectorResult};

pub struct ArxivCollector {
    client: reqwest::Client,
}

impl ArxivCollector {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
        }
    }
}

#[derive(Debug, Deserialize)]
struct ArxivFeed {
    #[serde(rename = "entry", default)]
    entries: Vec<ArxivEntry>,
}

#[derive(Debug, Deserialize)]
struct ArxivEntry {
    title: String,
    #[serde(rename = "author", default)]
    authors: Vec<ArxivAuthor>,
    id: String,
    published: String,
    summary: String,
}

#[derive(Debug, Deserialize)]
struct ArxivAuthor {
    name: String,
}

#[async_trait]
impl Collector for ArxivCollector {
    fn name(&self) -> &str {
        "Arxiv"
    }

    fn description(&self) -> &str {
        "Collects academic papers from arXiv.org"
    }

    async fn collect(&self, query: &str, max_results: usize) -> CollectorResult<Vec<Article>> {
        let url = format!(
            "http://export.arxiv.org/api/query?search_query={}&start=0&max_results={}",
            urlencoding::encode(query),
            max_results
        );

        tracing::info!("Fetching from Arxiv: {}", url);

        let response = self.client.get(&url).send().await?;
        let xml_text = response.text().await?;

        // Parse XML response
        let feed: ArxivFeed = from_str(&xml_text).map_err(|e| {
            tracing::error!("Failed to parse Arxiv XML: {}", e);
            format!("Failed to parse Arxiv response: {}", e)
        })?;

        let articles = feed
            .entries
            .into_iter()
            .map(|entry| Article {
                title: entry.title.trim().replace('\n', " "),
                authors: entry.authors.into_iter().map(|a| a.name).collect(),
                url: entry.id,
                published_date: entry.published,
                summary: entry.summary.trim().replace('\n', " "),
                source: "Arxiv".to_string(),
            })
            .collect();

        Ok(articles)
    }
}

mod urlencoding {
    pub fn encode(s: &str) -> String {
        s.bytes()
            .map(|b| match b {
                b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                    (b as char).to_string()
                }
                _ => format!("%{:02X}", b),
            })
            .collect()
    }
}
