mod error;
mod parser;
pub mod data;

use main_error::MainResult;
use reqwest::get;
pub use error::*;
use crate::parser::{PlayerParser, Parser};

pub type Result<T, E = ScrapeError> = std::result::Result<T, E>;

#[tokio::main]
async fn main() -> MainResult {
    let body = get("https://www.ugcleague.com/players_page.cfm?player_id=76561198024494988").await?.text().await?;
    let parser = PlayerParser::new();
    dbg!(parser.parse(&body)?);
    Ok(())
}
