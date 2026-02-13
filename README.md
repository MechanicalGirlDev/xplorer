# xplorer

A Discord bot written in Rust that collects and shares academic papers and articles from various sources like arXiv.

## Architecture

The bot uses a trait-based architecture for extensibility:

```rust
#[async_trait]
pub trait Collector: Send + Sync {
    fn name(&self) -> &str;
    async fn collect(&self, query: &str, max_results: usize) -> CollectorResult<Vec<Article>>;
    fn description(&self) -> &str;
}
```

Currently implemented collectors:
- **ArxivCollector**: Collects academic papers from arXiv.org
- **ExampleArticleCollector**: Template for adding more sources

## Setup

### Prerequisites

- Rust 1.70 or later
- A Discord Bot Token ([Create one here](https://discord.com/developers/applications))

### Installation

1. Clone the repository:
```bash
git clone https://github.com/MechanicalGirlDev/xplorer.git
cd xplorer
```

2. Copy the example environment file:
```bash
cp .env.example .env
```

3. Edit `.env` and add your Discord bot token:
```env
DISCORD_TOKEN=your_discord_bot_token_here
GUILD_ID=your_guild_id_here  # Optional: for faster command registration during development
CHANNEL_ID=your_channel_id_here  # Optional: for periodic collection posts
```

4. Build and run:
```bash
cargo build --release
cargo run --release
```

## Configuration

Edit the `.env` file to configure the bot:

- `DISCORD_TOKEN`: Your Discord bot token (required)
- `GUILD_ID`: Discord server ID for faster command registration (optional)
- `CHANNEL_ID`: Channel ID where periodic collections will be posted (optional)
- `ARXIV_MAX_RESULTS`: Default maximum results from arXiv (default: 10)
- `ARXIV_SEARCH_QUERY`: Default arXiv search query (default: cat:cs.AI)
- `COLLECTION_SCHEDULE`: Cron schedule for periodic collection (default: "0 0 9 * * *")

## Usage

### Slash Commands

Once the bot is running and added to your server, you can use these commands:

- `/collect source:<source> [query:<query>] [max_results:<number>]`
  - Collect articles from a specific source
  - **source**: Choose "arxiv" or "all"
  - **query**: Search query (optional, uses default from config)
  - **max_results**: Number of results to return (1-20, optional)

- `/sources`
  - List all available article sources

- `/schedule`
  - Show the current periodic collection schedule

### Example Commands

```
/collect source:arxiv query:cat:cs.LG max_results:5
/collect source:all
/sources
/schedule
```

## Adding New Collectors

To add a new article source, implement the `Collector` trait:

```rust
use async_trait::async_trait;
use super::{Article, Collector, CollectorResult};

pub struct MyCollector {
    client: reqwest::Client,
}

#[async_trait]
impl Collector for MyCollector {
    fn name(&self) -> &str {
        "My Source"
    }

    fn description(&self) -> &str {
        "Collects articles from my favorite source"
    }

    async fn collect(&self, query: &str, max_results: usize) -> CollectorResult<Vec<Article>> {
        // Implementation here
        Ok(vec![])
    }
}
```

Then register it in `src/bot.rs`:

```rust
let collectors: Vec<Box<dyn Collector>> = vec![
    Box::new(ArxivCollector::new()),
    Box::new(MyCollector::new()),
];
```

## Development

### Building

```bash
cargo build
```

### Running

```bash
cargo run
```

### Testing

```bash
cargo test
```

### Linting

```bash
cargo clippy
```

## License

See [LICENSE](LICENSE) file for details.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.
