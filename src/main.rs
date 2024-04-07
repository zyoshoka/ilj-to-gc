use std::fs;

use serde::Deserialize;

mod book;
mod calendar;

#[derive(Deserialize)]
struct Config {
    base_url: String,
    userid: String,
    password: String,
    google_private_key_id: String,
    google_private_key: String,
    google_client_email: String,
    calendar_id: String,
}

async fn load_config() -> Result<Config, Box<dyn std::error::Error>> {
    let config_path = "config.toml";
    let config_file = fs::read_to_string(config_path)?;
    let config_data: Config = toml::from_str(&config_file)?;

    Ok(config_data)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = reqwest::ClientBuilder::new()
        .cookie_store(true)
        .build()?;
    let config = load_config().await?;

    let books = book::get_borrowed_books(&client, &config).await?;

    calendar::subscribe_to_calender(&client, &config, &books).await;

    Ok(())
}
