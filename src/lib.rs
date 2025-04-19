pub mod data;
mod error;
#[doc(hidden)]
pub mod parser;

use crate::data::{
    GameMode, MapHistory, MatchInfo, MembershipHistory, Player, Seasons, Team, TeamRef,
    TeamRosterData, TeamSeason, Transaction,
};
use crate::parser::{
    MapHistoryParser, MatchPageParser, Parser, PlayerDetailsParser, PlayerParser, SeasonsParser,
    TeamLookupParser, TeamMatchesParser, TeamParser, TeamRosterHistoryParser, TransactionParser,
};
pub use error::*;
use reqwest::redirect::Policy;
use reqwest::{Client, IntoUrl, Response, StatusCode};
use std::time::Duration;
pub use steamid_ng::SteamID;
use tokio::time::sleep;
use tracing::warn;

pub type Result<T, E = ScrapeError> = std::result::Result<T, E>;

pub struct UgcClient {
    client: Client,
    player_parser: PlayerParser,
    player_detail_parser: PlayerDetailsParser,
    team_parser: TeamParser,
    team_roster_history_parser: TeamRosterHistoryParser,
    team_matches_parser: TeamMatchesParser,
    seasons_parser: SeasonsParser,
    team_lookup_parser: TeamLookupParser,
    match_page_parser: MatchPageParser,
    transaction_parser: TransactionParser,
    map_history_parser: MapHistoryParser,
}

impl Default for UgcClient {
    fn default() -> Self {
        Self::new()
    }
}

/// "API client" for ugc by scraping the website
impl UgcClient {
    pub fn new() -> Self {
        let redirect_policy = Policy::custom(|attempt| {
            // the different matchpage_* redirect to each other if you use the match id from a different game mode
            if attempt.url().path().contains("matchpage_") {
                attempt.follow()
            } else {
                attempt.stop()
            }
        });
        UgcClient {
            client: Client::builder().redirect(redirect_policy).build().unwrap(),
            player_parser: PlayerParser::new(),
            player_detail_parser: PlayerDetailsParser::new(),
            team_parser: TeamParser::new(),
            team_roster_history_parser: TeamRosterHistoryParser::new(),
            team_matches_parser: TeamMatchesParser::new(),
            seasons_parser: SeasonsParser::new(),
            team_lookup_parser: TeamLookupParser::new(),
            match_page_parser: MatchPageParser::new(),
            transaction_parser: TransactionParser::new(),
            map_history_parser: MapHistoryParser::new(),
        }
    }
    async fn request<U: IntoUrl>(&self, url: U) -> Result<String> {
        let url = url.into_url()?;
        match self.try_request(url.clone()).await {
            Ok(res) => Ok(res),
            Err(ScrapeError::Request(e)) => {
                warn!(url = url.as_str(), error = ?e, "failed to send request, retrying");
                sleep(Duration::from_secs_f32(0.5)).await;
                self.try_request(url).await
            }
            Err(e) => Err(e),
        }
    }

    async fn try_request<U: IntoUrl>(&self, url: U) -> Result<String> {
        Ok(self
            .client
            .get(url)
            .send()
            .await?
            .check_not_found()?
            .error_for_status()?
            .text()
            .await?)
    }

    /// Retrieve player information
    pub async fn player(&self, steam_id: SteamID) -> Result<Player> {
        let body = self
            .request(format!(
                "https://www.ugcleague.com/players_page.cfm?player_id={}",
                u64::from(steam_id)
            ))
            .await?;
        self.player_parser.parse(&body)
    }

    /// Retrieve team membership history for a player
    pub async fn player_team_history(&self, steam_id: SteamID) -> Result<Vec<MembershipHistory>> {
        let body = self
            .request(format!(
                "https://www.ugcleague.com/players_page_details.cfm?player_id={}",
                u64::from(steam_id)
            ))
            .await?;
        self.player_detail_parser.parse(&body)
    }

    /// Retrieve team information
    pub async fn team(&self, id: u32) -> Result<Team> {
        let body = self
            .request(format!(
                "https://www.ugcleague.com/team_page.cfm?clan_id={}",
                id
            ))
            .await?;
        self.team_parser.parse(&body)
    }

    /// Retrieve team roster history
    pub async fn team_roster_history(&self, id: u32) -> Result<TeamRosterData> {
        let body = self
            .request(format!(
                "https://www.ugcleague.com/team_page_rosterhistory.cfm?clan_id={}",
                id
            ))
            .await?;
        self.team_roster_history_parser.parse(&body)
    }

    /// Retrieve team match history
    pub async fn team_matches(&self, id: u32) -> Result<Vec<TeamSeason>> {
        let body = self
            .request(format!(
                "https://www.ugcleague.com/team_page_matches.cfm?clan_id={}",
                id
            ))
            .await?;
        self.team_matches_parser.parse(&body)
    }

    /// Get all historical seasons by game mode
    pub async fn previous_seasons(&self) -> Result<Vec<Seasons>> {
        let body = self.request("https://www.ugcleague.com").await?;
        self.seasons_parser.parse(&body)
    }

    pub async fn teams(&self, format: GameMode) -> Result<Vec<TeamRef>> {
        let link = format!(
            "https://www.ugcleague.com/team_lookup_tf2{}.cfm",
            format.letter()
        );
        let body = self.request(link).await?;
        self.team_lookup_parser.parse(&body)
    }

    /// Get match page info
    pub async fn match_info(&self, id: u32) -> Result<MatchInfo> {
        let body = self
            .request(format!(
                "https://www.ugcleague.com/matchpage_tf2h.cfm?mid={}",
                id
            ))
            .await?;
        self.match_page_parser.parse(&body)
    }

    pub async fn transactions(&self, format: GameMode) -> Result<Vec<Transaction>> {
        let link = format!(
            "https://www.ugcleague.com/rostertransactions_tf2{}_all.cfm",
            format.letter()
        );
        let body = self.request(link).await?;
        self.transaction_parser.parse(&body)
    }

    pub async fn map_history(&self, format: GameMode) -> Result<MapHistory> {
        let link = format!(
            "https://www.ugcleague.com/maplist_tf2{}.cfm",
            format.letter()
        );
        let body = self.request(link).await?;
        self.map_history_parser.parse(&body)
    }
}

trait ResponseExt: Sized {
    fn check_not_found(self) -> Result<Self, ScrapeError>;
}

impl ResponseExt for Response {
    fn check_not_found(self) -> Result<Self, ScrapeError> {
        if self.status() == StatusCode::FOUND {
            Err(ScrapeError::NotFound)
        } else {
            Ok(self)
        }
    }
}
