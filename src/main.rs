pub mod data;
mod error;
mod parser;

use crate::parser::{Parser, PlayerDetailsParser, PlayerParser};
pub use error::*;
use main_error::MainResult;
use reqwest::get;

pub type Result<T, E = ScrapeError> = std::result::Result<T, E>;

#[tokio::main]
async fn main() -> MainResult {
    let body =
        get("https://www.ugcleague.com/players_page_details.cfm?player_id=76561198024494988")
            .await?
            .text()
            .await?;
    let parser = PlayerDetailsParser::new();
    dbg!(parser.parse(&body)?);
    Ok(())
}
