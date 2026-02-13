# xplorer Architecture

## Overview

xplorer is a Discord bot written in Rust that collects and shares academic papers and articles from various sources. The architecture is designed to be extensible, allowing easy addition of new content sources.

## Architecture Diagram

```
┌─────────────────────────────────────────────────────────────┐
│                        Discord Bot                          │
│                      (Serenity Client)                      │
└──────────────────────┬──────────────────────────────────────┘
                       │
           ┌───────────┴───────────┐
           │                       │
    ┌──────▼──────┐         ┌─────▼─────┐
    │  Commands   │         │ Scheduler │
    │  Handler    │         │  (Cron)   │
    └──────┬──────┘         └─────┬─────┘
           │                      │
           └──────────┬───────────┘
                      │
              ┌───────▼────────┐
              │   Bot Logic    │
              │  (Collection)  │
              └───────┬────────┘
                      │
          ┌───────────┴───────────┐
          │                       │
    ┌─────▼──────┐         ┌──────▼─────┐
    │   Arxiv    │   ...   │  Example   │
    │ Collector  │         │ Collector  │
    └────────────┘         └────────────┘
```

## Core Components

### 1. Main Entry Point (`main.rs`)

- Initializes the Discord client
- Loads configuration from environment variables
- Sets up the scheduler for periodic collection
- Starts the bot

**Key Responsibilities:**
- Environment setup
- Client initialization
- Task spawning (scheduler)

### 2. Bot Handler (`bot.rs`)

The `Bot` struct implements Discord's `EventHandler` trait and handles:

- **Command Registration**: Registers slash commands when bot is ready
- **Command Handling**: Processes `/collect`, `/sources`, and `/schedule` commands
- **Collection Logic**: Coordinates between collectors and Discord responses
- **Response Formatting**: Formats article data for Discord display

**Key Methods:**
- `handle_collect_command()`: Processes collection requests
- `handle_sources_command()`: Lists available collectors
- `handle_schedule_command()`: Shows schedule information
- `format_articles_response()`: Formats articles for Discord

### 3. Commands Module (`commands.rs`)

Defines the structure of Discord slash commands using Serenity's builder pattern.

**Commands:**
- `/collect`: Collect articles with parameters
- `/sources`: List available sources
- `/schedule`: Show collection schedule

### 4. Collectors Module (`collectors/`)

The heart of the extensibility - a trait-based system for content collection.

#### Collector Trait

```rust
#[async_trait]
pub trait Collector: Send + Sync {
    fn name(&self) -> &str;
    async fn collect(&self, query: &str, max_results: usize) 
        -> CollectorResult<Vec<Article>>;
    fn description(&self) -> &str;
}
```

This trait defines the interface that all collectors must implement.

#### Article Struct

```rust
pub struct Article {
    pub title: String,
    pub authors: Vec<String>,
    pub url: String,
    pub published_date: String,
    pub summary: String,
    pub source: String,
}
```

A standardized representation of collected content, serializable with Serde.

#### Implementations

**ArxivCollector** (`arxiv.rs`)
- Queries arXiv API
- Parses XML responses using quick-xml
- Converts to Article structs
- Handles URL encoding for search queries

**ExampleArticleCollector** (`example.rs`)
- Template/placeholder for additional collectors
- Demonstrates the interface
- Can be replaced with real implementations (Medium, Dev.to, etc.)

## Data Flow

### Command-Triggered Collection

1. User issues `/collect` command in Discord
2. Discord sends interaction to bot
3. Bot's `interaction_create` handler receives it
4. `handle_collect_command()` is called:
   - Parses command parameters
   - Defers response (collection may take time)
   - Locks collector list
   - Calls `collect()` on selected collector(s)
   - Formats results
   - Edits deferred response with results

### Periodic Collection (Scheduled)

1. Cron scheduler triggers at specified time
2. Scheduler executes async job
3. Job would:
   - Create collection context
   - Call collectors
   - Format results
   - Post to specified channel

*Note: Full periodic collection implementation requires shared state between the scheduler and bot instance.*

## Extension Points

### Adding a New Collector

1. Create new file in `src/collectors/`
2. Implement the `Collector` trait
3. Add to exports in `src/collectors/mod.rs`
4. Register in `Bot::new()` in `src/bot.rs`

Example:

```rust
// src/collectors/hackernews.rs
pub struct HackerNewsCollector {
    client: reqwest::Client,
}

#[async_trait]
impl Collector for HackerNewsCollector {
    fn name(&self) -> &str { "Hacker News" }
    
    fn description(&self) -> &str { 
        "Collects top stories from Hacker News" 
    }
    
    async fn collect(&self, query: &str, max_results: usize) 
        -> CollectorResult<Vec<Article>> 
    {
        // Implementation
        Ok(vec![])
    }
}
```

### Configuration

All configuration is managed through environment variables (`.env` file):

- **Required**: `DISCORD_TOKEN`
- **Optional**: 
  - `GUILD_ID` - For faster command registration in dev
  - `CHANNEL_ID` - For periodic collection posts
  - `ARXIV_SEARCH_QUERY` - Default search query
  - `ARXIV_MAX_RESULTS` - Default result count
  - `COLLECTION_SCHEDULE` - Cron schedule string

## Dependencies

### Core Libraries

- **serenity** (0.12): Discord API client
- **tokio** (1.35): Async runtime
- **reqwest** (0.11): HTTP client
- **serde/serde_json**: Serialization
- **async-trait**: Async trait support

### Specific Use

- **quick-xml**: XML parsing for arXiv
- **tokio-cron-scheduler**: Periodic task scheduling
- **tracing/tracing-subscriber**: Logging
- **dotenv**: Environment variable loading

## Security Considerations

1. **Token Security**: Discord token should never be committed
2. **Input Validation**: User inputs are validated before use
3. **Error Handling**: All external API calls have proper error handling
4. **URL Encoding**: Proper UTF-8 byte encoding for URL safety
5. **Rate Limiting**: Should be added for production use

## Future Enhancements

1. **Database**: Store collected articles for history
2. **Filtering**: Allow users to set preferences/filters
3. **Notifications**: @mention users for specific topics
4. **Analytics**: Track popular queries and sources
5. **More Collectors**: Medium, Dev.to, Google Scholar, etc.
6. **Full Periodic Collection**: Complete scheduler integration
7. **Rate Limiting**: Implement rate limiting for API calls
8. **Caching**: Cache recent results to reduce API calls
