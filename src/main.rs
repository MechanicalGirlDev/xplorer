mod bot;
mod collectors;
mod commands;

use bot::Bot;
use serenity::all::GatewayIntents;
use serenity::Client;
use std::env;
use tokio_cron_scheduler::{Job, JobScheduler};

#[tokio::main]
async fn main() {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    // Load environment variables from .env file
    dotenv::dotenv().ok();

    // Get Discord token
    let token = env::var("DISCORD_TOKEN").expect("DISCORD_TOKEN must be set in environment");

    // Get configuration
    let default_query = env::var("ARXIV_SEARCH_QUERY").unwrap_or_else(|_| "cat:cs.AI".to_string());
    let default_max_results = env::var("ARXIV_MAX_RESULTS")
        .unwrap_or_else(|_| "10".to_string())
        .parse::<usize>()
        .unwrap_or(10);
    let schedule = env::var("COLLECTION_SCHEDULE").unwrap_or_else(|_| "0 0 9 * * *".to_string());

    tracing::info!("Starting xplorer Discord bot");
    tracing::info!("Default query: {}", default_query);
    tracing::info!("Default max results: {}", default_max_results);
    tracing::info!("Collection schedule: {}", schedule);

    // Create bot instance
    let bot = Bot::new(default_query, default_max_results);

    // Set up Discord client
    let intents = GatewayIntents::GUILDS | GatewayIntents::GUILD_MESSAGES;

    let mut client = Client::builder(&token, intents)
        .event_handler(bot)
        .await
        .expect("Error creating client");

    // Set up scheduler for periodic collection
    tokio::spawn(async move {
        tracing::info!("Setting up scheduler");

        let scheduler = JobScheduler::new().await;

        if let Ok(scheduler) = scheduler {
            // Get channel ID from environment if set
            let channel_id = env::var("CHANNEL_ID")
                .ok()
                .and_then(|s| s.parse::<u64>().ok());

            if let Some(channel_id) = channel_id {
                tracing::info!("Periodic collection will post to channel {}", channel_id);

                let schedule_str = schedule.clone();
                let job = Job::new_async(schedule_str.as_str(), move |_uuid, _l| {
                    Box::pin(async move {
                        tracing::info!("Scheduler triggered - periodic collection placeholder");
                        // Note: To implement periodic collection, you would need to:
                        // 1. Create a shared state structure containing collectors and HTTP client
                        // 2. Pass it into this closure
                        // 3. Call the collection logic here
                        // 4. Post results to the channel using the HTTP client
                    })
                });

                if let Ok(job) = job {
                    if let Err(e) = scheduler.add(job).await {
                        tracing::error!("Failed to add scheduled job: {}", e);
                    } else {
                        tracing::info!("Scheduled job added successfully");
                    }
                }
            } else {
                tracing::warn!("CHANNEL_ID not set, periodic collection disabled");
            }

            if let Err(e) = scheduler.start().await {
                tracing::error!("Failed to start scheduler: {}", e);
            } else {
                tracing::info!("Scheduler started");
            }
        } else {
            tracing::error!("Failed to create scheduler");
        }
    });

    // Start the client
    if let Err(why) = client.start().await {
        tracing::error!("Client error: {:?}", why);
    }
}
