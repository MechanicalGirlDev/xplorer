use serde::Deserialize;
use std::fs;
use std::error::Error;

#[derive(Deserialize, Debug, Clone)]
pub struct DiscordConfig {
    pub token: String,
    pub guild_id: Option<u64>,
    pub channel_id: Option<u64>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Config {
    pub discord: DiscordConfig,
    #[serde(default = "default_query")]
    pub arxiv_search_query: String,
    #[serde(default = "default_max_results")]
    pub arxiv_max_results: usize,
    #[serde(default = "default_schedule")]
    pub collection_schedule: String,
}

fn default_query() -> String {
    "cat:cs.AI".to_string()
}

fn default_max_results() -> usize {
    10
}

fn default_schedule() -> String {
    "0 0 9 * * *".to_string()
}

impl Config {
    pub fn load() -> Result<Self, Box<dyn Error>> {
        let config_str = fs::read_to_string("config.toml")
            .map_err(|e| format!("Failed to read config.toml: {}", e))?;
        let config: Config = toml::from_str(&config_str)
            .map_err(|e| format!("Failed to parse config.toml: {}", e))?;
        Ok(config)
    }
}
