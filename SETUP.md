# xplorer Bot Setup Guide

## Quick Start

### 1. Create a Discord Bot

1. Go to [Discord Developer Portal](https://discord.com/developers/applications)
2. Click "New Application" and give it a name (e.g., "xplorer")
3. Go to the "Bot" section
4. Click "Add Bot"
5. Under "TOKEN", click "Reset Token" and copy it (you'll need this for `.env`)
6. Under "Privileged Gateway Intents", enable:
   - Presence Intent (optional)
   - Server Members Intent (optional)
   - Message Content Intent (if you want to read messages)

### 2. Invite the Bot to Your Server

1. Go to the "OAuth2" > "URL Generator" section
2. Select scopes:
   - `bot`
   - `applications.commands`
3. Select bot permissions:
   - Send Messages
   - Embed Links
   - Read Message History
4. Copy the generated URL and open it in your browser
5. Select your server and authorize the bot

### 3. Configure the Bot

1. Copy the example environment file:
```bash
cp .env.example .env
```

2. Edit `.env` and fill in your values:
```env
DISCORD_TOKEN=your_bot_token_here
GUILD_ID=your_server_id_here  # Optional: Right-click server icon > Copy ID (need Developer Mode enabled)
CHANNEL_ID=your_channel_id_here  # Optional: Right-click channel > Copy ID
ARXIV_SEARCH_QUERY=cat:cs.AI  # Default search query for arXiv
ARXIV_MAX_RESULTS=10  # Default number of results
COLLECTION_SCHEDULE=0 0 9 * * *  # Daily at 9 AM UTC
```

### 4. Build and Run

```bash
# Build the bot
cargo build --release

# Run the bot
cargo run --release
```

## Using the Bot

### Slash Commands

Once the bot is running, you can use these commands in Discord:

#### `/collect`
Collect articles from various sources.

**Parameters:**
- `source` (required): Choose "arxiv" or "all"
- `query` (optional): Search query (defaults to config)
- `max_results` (optional): Number of results (1-20)

**Examples:**
```
/collect source:arxiv
/collect source:arxiv query:cat:cs.LG max_results:5
/collect source:all query:cat:cs.AI
```

#### `/sources`
List all available article sources and their descriptions.

```
/sources
```

#### `/schedule`
Show the current periodic collection schedule.

```
/schedule
```

## arXiv Query Syntax

The bot uses arXiv's query syntax. Here are some common examples:

- `cat:cs.AI` - AI papers
- `cat:cs.LG` - Machine Learning papers
- `cat:cs.CV` - Computer Vision papers
- `cat:cs.CL` - Computational Linguistics papers
- `all:quantum` - Papers with "quantum" in any field
- `ti:neural` - Papers with "neural" in title
- `au:lecun` - Papers by author "lecun"

You can combine queries:
- `ti:deep AND cat:cs.AI` - AI papers with "deep" in title

For more details, see [arXiv API User Manual](https://arxiv.org/help/api/user-manual).

## Troubleshooting

### Bot doesn't respond to commands
- Make sure the bot has "Send Messages" permission in the channel
- Check if slash commands are registered (may take up to 1 hour globally, instant for guild)
- Set `GUILD_ID` in `.env` for faster development command registration

### Can't find server/channel ID
- Enable Developer Mode in Discord: User Settings > Advanced > Developer Mode
- Right-click on server icon or channel and select "Copy ID"

### Bot crashes on startup
- Check that `DISCORD_TOKEN` is correctly set in `.env`
- Ensure all dependencies are installed: `cargo build`
- Check the logs for specific error messages

## Development

### Adding New Collectors

See [README.md](README.md#adding-new-collectors) for instructions on implementing the `Collector` trait.

### Running in Development

```bash
# Watch for changes and recompile
cargo watch -x run

# Run with debug logging
RUST_LOG=debug cargo run
```
