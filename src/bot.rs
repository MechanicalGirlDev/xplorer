use serenity::all::{
    CommandInteraction, CreateInteractionResponse, CreateInteractionResponseMessage,
    EditInteractionResponse,
};
use serenity::async_trait;
use serenity::client::{Context, EventHandler};
use serenity::model::gateway::Ready;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::collectors::{Article, ArxivCollector, Collector, ExampleArticleCollector};
use crate::config::Config;

pub struct Bot {
    collectors: Arc<Mutex<Vec<Box<dyn Collector>>>>,
    default_query: String,
    default_max_results: usize,
    schedule: String,
    guild_id: Option<u64>,
}

const MAX_ARTICLES_DISPLAYED: usize = 5;
const DISCORD_MESSAGE_LIMIT: usize = 2000;
const MAX_SUMMARY_LENGTH: usize = 200;
const TRUNCATION_SUFFIX: &str = "...";

impl Bot {
    pub fn new(config: &Config) -> Self {
        let collectors: Vec<Box<dyn Collector>> = vec![
            Box::new(ArxivCollector::new()),
            Box::new(ExampleArticleCollector::new()),
        ];

        Self {
            collectors: Arc::new(Mutex::new(collectors)),
            default_query: config.arxiv_search_query.clone(),
            default_max_results: config.arxiv_max_results,
            schedule: config.collection_schedule.clone(),
            guild_id: config.discord.guild_id,
        }
    }

    async fn handle_collect_command(&self, ctx: &Context, command: &CommandInteraction) {
        let source = command
            .data
            .options
            .iter()
            .find(|opt| opt.name == "source")
            .and_then(|opt| opt.value.as_str())
            .unwrap_or("arxiv");

        let query = command
            .data
            .options
            .iter()
            .find(|opt| opt.name == "query")
            .and_then(|opt| opt.value.as_str())
            .unwrap_or(&self.default_query);

        let max_results = command
            .data
            .options
            .iter()
            .find(|opt| opt.name == "max_results")
            .and_then(|opt| opt.value.as_i64())
            .unwrap_or(self.default_max_results as i64) as usize;

        // Defer the response since collection might take time
        if let Err(why) = command.defer(&ctx.http).await {
            tracing::error!("Cannot defer response: {}", why);
            return;
        }

        let collectors = self.collectors.lock().await;
        let mut all_articles = Vec::new();

        if source == "all" {
            for collector in collectors.iter() {
                match collector.collect(query, max_results).await {
                    Ok(articles) => {
                        tracing::info!(
                            "Collected {} articles from {}",
                            articles.len(),
                            collector.name()
                        );
                        all_articles.extend(articles);
                    }
                    Err(e) => {
                        tracing::error!("Error collecting from {}: {}", collector.name(), e);
                    }
                }
            }
        } else {
            let collector = collectors
                .iter()
                .find(|c| c.name().to_lowercase() == source.to_lowercase());

            if let Some(collector) = collector {
                match collector.collect(query, max_results).await {
                    Ok(articles) => {
                        tracing::info!(
                            "Collected {} articles from {}",
                            articles.len(),
                            collector.name()
                        );
                        all_articles = articles;
                    }
                    Err(e) => {
                        tracing::error!("Error collecting from {}: {}", collector.name(), e);
                        let _ = command
                            .edit_response(
                                &ctx.http,
                                EditInteractionResponse::new().content(format!("❌ Error: {}", e)),
                            )
                            .await;
                        return;
                    }
                }
            } else {
                let _ = command
                    .edit_response(
                        &ctx.http,
                        EditInteractionResponse::new()
                            .content(format!("❌ Unknown source: {}", source)),
                    )
                    .await;
                return;
            }
        }

        let response = self.format_articles_response(&all_articles, source);

        if let Err(why) = command
            .edit_response(&ctx.http, EditInteractionResponse::new().content(response))
            .await
        {
            tracing::error!("Cannot respond to slash command: {}", why);
        }
    }

    async fn handle_sources_command(&self, ctx: &Context, command: &CommandInteraction) {
        let collectors = self.collectors.lock().await;
        let mut response = "📚 **Available Sources:**\n\n".to_string();

        for collector in collectors.iter() {
            response.push_str(&format!(
                "• **{}**: {}\n",
                collector.name(),
                collector.description()
            ));
        }

        let data = CreateInteractionResponseMessage::new().content(response);
        let builder = CreateInteractionResponse::Message(data);

        if let Err(why) = command.create_response(&ctx.http, builder).await {
            tracing::error!("Cannot respond to slash command: {}", why);
        }
    }

    async fn handle_schedule_command(&self, ctx: &Context, command: &CommandInteraction) {
        let schedule =
            std::env::var("COLLECTION_SCHEDULE").unwrap_or_else(|_| "0 0 9 * * *".to_string());

        let response = format!(
            "📅 **Collection Schedule:**\n\nCron: `{}`\n\nThe bot will automatically collect articles based on this schedule.",
            self.schedule
        );

        let data = CreateInteractionResponseMessage::new().content(response);
        let builder = CreateInteractionResponse::Message(data);

        if let Err(why) = command.create_response(&ctx.http, builder).await {
            tracing::error!("Cannot respond to slash command: {}", why);
        }
    }

    fn format_articles_response(&self, articles: &[Article], source: &str) -> String {
        if articles.is_empty() {
            return format!("No articles found from {}.", source);
        }

        let mut response = format!(
            "📰 **Found {} article(s) from {}:**\n\n",
            articles.len(),
            source
        );

        for (i, article) in articles.iter().take(MAX_ARTICLES_DISPLAYED).enumerate() {
            response.push_str(&format!("**{}. {}**\n", i + 1, article.title));
            response.push_str(&format!("👤 Authors: {}\n", article.authors.join(", ")));
            response.push_str(&format!("📅 Published: {}\n", article.published_date));
            response.push_str(&format!("🔗 URL: {}\n", article.url));
            
            let summary = if article.summary.chars().count() > MAX_SUMMARY_LENGTH {
                let mut s: String = article.summary.chars().take(MAX_SUMMARY_LENGTH).collect();
                s.push_str(TRUNCATION_SUFFIX);
                s
            } else {
                article.summary.clone()
            };
            response.push_str(&format!("📝 Summary: {}\n\n", summary));
        }

        if articles.len() > MAX_ARTICLES_DISPLAYED {
            response.push_str(&format!("_...and {} more articles_\n", articles.len() - MAX_ARTICLES_DISPLAYED));
        }

        // Discord message limit is 2000 characters
        if response.chars().count() > DISCORD_MESSAGE_LIMIT {
            let mut truncated: String = response.chars().take(DISCORD_MESSAGE_LIMIT - TRUNCATION_SUFFIX.chars().count()).collect();
            truncated.push_str(TRUNCATION_SUFFIX);
            response = truncated;
        }

        response
    }

    #[allow(dead_code)]
    pub async fn periodic_collection(&self, ctx: Context, channel_id: u64) {
        tracing::info!("Running periodic collection");

        let collectors = self.collectors.lock().await;
        let mut all_articles = Vec::new();

        for collector in collectors.iter() {
            match collector
                .collect(&self.default_query, self.default_max_results)
                .await
            {
                Ok(articles) => {
                    tracing::info!(
                        "Periodic collection: {} articles from {}",
                        articles.len(),
                        collector.name()
                    );
                    all_articles.extend(articles);
                }
                Err(e) => {
                    tracing::error!("Periodic collection error from {}: {}", collector.name(), e);
                }
            }
        }

        if !all_articles.is_empty() {
            let response = self.format_articles_response(&all_articles, "scheduled collection");
            let channel = serenity::model::id::ChannelId::new(channel_id);

            if let Err(why) = channel.say(&ctx.http, response).await {
                tracing::error!("Error sending periodic collection message: {}", why);
            }
        }
    }
}

#[async_trait]
impl EventHandler for Bot {
    async fn ready(&self, ctx: Context, ready: Ready) {
        tracing::info!("{} is connected!", ready.user.name);

        let commands = vec![
            crate::commands::collect_command(),
            crate::commands::sources_command(),
            crate::commands::schedule_command(),
        ];

        if let Some(guild_id) = self.guild_id {
            let guild_id = serenity::model::id::GuildId::new(guild_id);
            if let Err(why) = guild_id.set_commands(&ctx.http, commands).await {
                tracing::error!("Cannot register guild commands: {}", why);
            } else {
                tracing::info!("Registered commands for guild {}", guild_id);
            }
            return;
        }

        // Register commands globally
        if let Err(why) = serenity::all::Command::set_global_commands(&ctx.http, commands).await {
            tracing::error!("Cannot register global commands: {}", why);
        } else {
            tracing::info!("Registered global commands");
        }
    }

    async fn interaction_create(
        &self,
        ctx: Context,
        interaction: serenity::model::application::Interaction,
    ) {
        if let serenity::model::application::Interaction::Command(command) = interaction {
            tracing::info!("Received command: {}", command.data.name);

            match command.data.name.as_str() {
                "collect" => self.handle_collect_command(&ctx, &command).await,
                "sources" => self.handle_sources_command(&ctx, &command).await,
                "schedule" => self.handle_schedule_command(&ctx, &command).await,
                _ => {
                    tracing::warn!("Unknown command: {}", command.data.name);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::collectors::Article;

    fn create_test_article(id: usize) -> Article {
        Article {
            title: format!("Test Article {}", id),
            authors: vec!["Author A".to_string(), "Author B".to_string()],
            url: format!("http://example.com/{}", id),
            published_date: "2024-01-01".to_string(),
            summary: "This is a test summary.".to_string(),
            source: "Test".to_string(),
        }
    }

    #[test]
    fn test_format_articles_response_max_articles() {
        let bot = Bot::new("test".to_string(), 10);
        let mut articles = Vec::new();
        for i in 0..MAX_ARTICLES_DISPLAYED + 2 {
            articles.push(create_test_article(i));
        }

        let response = bot.format_articles_response(&articles, "Test");

        // Verify that we only show MAX_ARTICLES_DISPLAYED articles
        let expected_count_str = format!("Found {} article(s)", articles.len());
        assert!(response.contains(&expected_count_str));

        // The response should contain the "more articles" message
        let expected_more_str = format!("_...and {} more articles_", articles.len() - MAX_ARTICLES_DISPLAYED);
        assert!(response.contains(&expected_more_str));
    }

    #[test]
    fn test_format_articles_response_truncation() {
        let bot = Bot::new("test".to_string(), 10);
        let mut articles = Vec::new();

        // Create articles with long content to force message truncation
        for i in 0..MAX_ARTICLES_DISPLAYED {
            let mut article = create_test_article(i);
            article.title = "A".repeat(300); // 300 chars
            article.summary = "S".repeat(MAX_SUMMARY_LENGTH + 50); // Will be truncated to MAX_SUMMARY_LENGTH
            articles.push(article);
        }

        // Total length will be roughly:
        // Header ~ 30
        // 5 * (Title(300) + Authors(20) + Date(20) + URL(20) + Summary(200+3) + overhead)
        // 5 * 563 = 2815
        // This should definitely trigger the 2000 char limit.

        let response = bot.format_articles_response(&articles, "Test");

        assert!(response.chars().count() <= DISCORD_MESSAGE_LIMIT);
        assert!(response.ends_with(TRUNCATION_SUFFIX));
    }

    #[test]
    fn test_format_articles_response_summary_truncation() {
        let bot = Bot::new("test".to_string(), 10);
        let mut article = create_test_article(1);
        let long_summary = "A".repeat(MAX_SUMMARY_LENGTH + 10);
        article.summary = long_summary.clone();

        let articles = vec![article];
        let response = bot.format_articles_response(&articles, "Test");

        // The summary should be truncated
        let expected_summary = format!("{}{}", &long_summary[..MAX_SUMMARY_LENGTH], TRUNCATION_SUFFIX);
        assert!(response.contains(&expected_summary));
    }
}
