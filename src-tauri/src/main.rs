#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    // Load environment variables from .env files
    let _ = dotenvy::dotenv();

    ai_news_aggregator::run();
}
