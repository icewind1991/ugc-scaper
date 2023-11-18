pub mod data;
mod error;
#[doc(hidden)]
pub mod parser;

use crate::data::{MembershipHistory, Player, RosterHistory, Seasons, Team, TeamRef, TeamSeason};
use crate::parser::{
    Parser, PlayerDetailsParser, PlayerParser, SeasonsParser, TeamLookupParser, TeamMatchesParser,
    TeamParser, TeamRosterHistoryParser,
};
pub use error::*;
use reqwest::redirect::Policy;
use reqwest::{Client, StatusCode};
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
    seasons_parser: SeasonsParser,
    team_lookup_parser: TeamLookupParser,
}

/// "API client" for ugc by scraping the website
impl UgcClient {
    pub fn new() -> Self {
        UgcClient {
            client: Client::builder().redirect(Policy::none()).build().unwrap(),
            player_parser: PlayerParser::new(),
            player_detail_parser: PlayerDetailsParser::new(),
            team_parser: TeamParser::new(),
            team_roster_history_parser: TeamRosterHistoryParser::new(),
            team_matches_parser: TeamMatchesParser::new(),
            seasons_parser: SeasonsParser::new(),
            team_lookup_parser: TeamLookupParser::new(),
        }
    }

    /// Retrieve player information
    pub async fn player(&self, steam_id: SteamID) -> Result<Player> {
        let res = self
            .client
            .get(&format!(
                "https://www.ugcleague.com/players_page.cfm?player_id={}",
                u64::from(steam_id)
            ))
            .send()
            .await?;
        if res.status() == StatusCode::FOUND {
            return Err(ScrapeError::NotFound);
        }
        let body = res.text().await?;
        self.player_parser.parse(&body)
    }

    /// Retrieve team membership history for a player
    pub async fn player_team_history(&self, steam_id: SteamID) -> Result<Vec<MembershipHistory>> {
        let res = self
            .client
            .get(&format!(
                "https://www.ugcleague.com/players_page_details.cfm?player_id={}",
                u64::from(steam_id)
            ))
            .send()
            .await?;
        if res.status() == StatusCode::FOUND {
            return Err(ScrapeError::NotFound);
        }
        let body = res.text().await?;
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

    /// Get all historical seasons by game mode
    pub async fn previous_seasons(&self) -> Result<Vec<Seasons>> {
        let body = self
            .client
            .get("https://www.ugcleague.com")
            .send()
            .await?
            .text()
            .await?;
        self.seasons_parser.parse(&body)
    }

    async fn teams(&self, link: &str) -> Result<Vec<TeamRef>> {
        let body = self.client.get(link).send().await?.text().await?;
        self.team_lookup_parser.parse(&body)
    }

    /// Get a list of all 9v9 teams
    pub async fn teams_9v9(&self) -> Result<Vec<TeamRef>> {
        self.teams("https://www.ugcleague.com/team_lookup_tf2h.cfm")
            .await
    }

    /// Get a list of all 6v6 teams
    pub async fn teams_6v6(&self) -> Result<Vec<TeamRef>> {
        self.teams("https://www.ugcleague.com/team_lookup_tf26.cfm")
            .await
    }

    /// Get a list of all 4v4 teams
    pub async fn teams_4v4(&self) -> Result<Vec<TeamRef>> {
        self.teams("https://www.ugcleague.com/team_lookup_tf24.cfm")
            .await
    }

    /// Get a list of all 2v2 teams
    pub async fn teams_2v2(&self) -> Result<Vec<TeamRef>> {
        self.teams("https://www.ugcleague.com/team_lookup_tf22.cfm")
            .await
    }
}
