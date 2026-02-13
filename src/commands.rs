use serenity::builder::{CreateCommand, CreateCommandOption};
use serenity::model::application::{CommandOptionType};

/// Creates the /collect command
pub fn collect_command() -> CreateCommand {
    CreateCommand::new("collect")
        .description("Collect articles from various sources")
        .add_option(
            CreateCommandOption::new(
                CommandOptionType::String,
                "source",
                "Source to collect from (arxiv, all)",
            )
            .required(true)
            .add_string_choice("Arxiv", "arxiv")
            .add_string_choice("All Sources", "all"),
        )
        .add_option(
            CreateCommandOption::new(
                CommandOptionType::String,
                "query",
                "Search query",
            )
            .required(false),
        )
        .add_option(
            CreateCommandOption::new(
                CommandOptionType::Integer,
                "max_results",
                "Maximum number of results (1-20)",
            )
            .required(false)
            .min_int_value(1)
            .max_int_value(20),
        )
}

/// Creates the /sources command
pub fn sources_command() -> CreateCommand {
    CreateCommand::new("sources")
        .description("List all available article sources")
}

/// Creates the /schedule command
pub fn schedule_command() -> CreateCommand {
    CreateCommand::new("schedule")
        .description("Show the current collection schedule")
}
