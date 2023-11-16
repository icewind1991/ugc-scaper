pub mod data;
mod error;
#[doc(hidden)]
pub mod parser;

use crate::data::{MembershipHistory, Player};
use crate::parser::{Parser, PlayerDetailsParser, PlayerParser};
pub use error::*;
use reqwest::Client;
use steamid_ng::SteamID;

pub type Result<T, E = ScrapeError> = std::result::Result<T, E>;

#[derive(Default)]
pub struct UgcClient {
    client: Client,
    player_parser: PlayerParser,
    player_detail_parser: PlayerDetailsParser,
}

impl UgcClient {
    pub fn new() -> Self {
        UgcClient {
            client: Client::default(),
            player_parser: PlayerParser::new(),
            player_detail_parser: PlayerDetailsParser::new(),
        }
    }

    pub async fn player(&self, steam_id: SteamID) -> Result<Player> {
        let body = self
            .client
            .get(&format!(
                "https://www.ugcleague.com/players_page.cfm?player_id={}",
                u64::from(steam_id)
            ))
            .send()
            .await?
            .text()
            .await?;
        self.player_parser.parse(&body)
    }

    pub async fn player_team_history(&self, steam_id: SteamID) -> Result<Vec<MembershipHistory>> {
        let body = self
            .client
            .get(&format!(
                "https://www.ugcleague.com/players_page_details.cfm?player_id={}",
                u64::from(steam_id)
            ))
            .send()
            .await?
            .text()
            .await?;
        self.player_detail_parser.parse(&body)
    }
}
