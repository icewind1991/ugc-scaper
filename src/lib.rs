pub mod data;
mod error;
#[doc(hidden)]
pub mod parser;

use crate::data::{MembershipHistory, Player, RosterHistory, Team, TeamSeason};
use crate::parser::{
    Parser, PlayerDetailsParser, PlayerParser, TeamMatchesParser, TeamParser,
    TeamRosterHistoryParser,
};
pub use error::*;
use reqwest::Client;
pub use steamid_ng::SteamID;

pub type Result<T, E = ScrapeError> = std::result::Result<T, E>;

#[derive(Default)]
pub struct UgcClient {
    client: Client,
    player_parser: PlayerParser,
    player_detail_parser: PlayerDetailsParser,
    team_parser: TeamParser,
    team_roster_history_parser: TeamRosterHistoryParser,
    team_matches_parser: TeamMatchesParser,
}

/// "API client" for ugc by scraping the website
impl UgcClient {
    pub fn new() -> Self {
        UgcClient {
            client: Client::default(),
            player_parser: PlayerParser::new(),
            player_detail_parser: PlayerDetailsParser::new(),
            team_parser: TeamParser::new(),
            team_roster_history_parser: TeamRosterHistoryParser::new(),
            team_matches_parser: TeamMatchesParser::new(),
        }
    }

    /// Retrieve player information
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

    /// Retrieve team membership history for a player
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

    /// Retrieve team information
    pub async fn team(&self, id: u32) -> Result<Team> {
        let body = self
            .client
            .get(&format!(
                "https://www.ugcleague.com/team_page.cfm?clan_id={}",
                id
            ))
            .send()
            .await?
            .text()
            .await?;
        self.team_parser.parse(&body)
    }

    /// Retrieve team roster history
    pub async fn team_roster_history(&self, id: u32) -> Result<Vec<RosterHistory>> {
        let body = self
            .client
            .get(&format!(
                "https://www.ugcleague.com/team_page_rosterhistory.cfm?clan_id={}",
                id
            ))
            .send()
            .await?
            .text()
            .await?;
        self.team_roster_history_parser.parse(&body)
    }

    /// Retrieve team match history
    pub async fn team_matches(&self, id: u32) -> Result<Vec<TeamSeason>> {
        let body = self
            .client
            .get(&format!(
                "https://www.ugcleague.com/team_page_matches.cfm?clan_id={}",
                id
            ))
            .send()
            .await?
            .text()
            .await?;
        self.team_matches_parser.parse(&body)
    }
}
