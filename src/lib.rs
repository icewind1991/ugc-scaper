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
    match_page_parser: MatchPageParser,
    transaction_parser: TransactionParser,
    map_history_parser: MapHistoryParser,
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
            match_page_parser: MatchPageParser::new(),
            transaction_parser: TransactionParser::new(),
            map_history_parser: MapHistoryParser::new(),
        }
    }

    /// Retrieve player information
    pub async fn player(&self, steam_id: SteamID) -> Result<Player> {
        let res = self
            .client
            .get(format!(
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
            .get(format!(
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
            .get(format!(
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
    pub async fn team_roster_history(&self, id: u32) -> Result<TeamRosterData> {
        let body = self
            .client
            .get(format!(
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
            .get(format!(
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

    pub async fn teams(&self, format: GameMode) -> Result<Vec<TeamRef>> {
        let link = format!(
            "https://www.ugcleague.com/team_lookup_tf2{}.cfm",
            format.letter()
        );
        let body = self.client.get(link).send().await?.text().await?;
        self.team_lookup_parser.parse(&body)
    }

    /// Get match page info
    pub async fn match_info(&self, id: u32) -> Result<MatchInfo> {
        let body = self
            .client
            .get(format!(
                "https://www.ugcleague.com/matchpage_tf2h.cfm?mid={}",
                id
            ))
            .send()
            .await?
            .text()
            .await?;
        self.match_page_parser.parse(&body)
    }

    pub async fn transactions(&self, format: GameMode) -> Result<Vec<Transaction>> {
        let link = format!(
            "https://www.ugcleague.com/rostertransactions_tf2{}_all.cfm",
            format.letter()
        );
        let body = self.client.get(link).send().await?.text().await?;
        self.transaction_parser.parse(&body)
    }

    pub async fn map_history(&self, format: GameMode) -> Result<MapHistory> {
        let link = format!(
            "https://www.ugcleague.com/maplist_tf2{}.cfm",
            format.letter()
        );
        let body = self.client.get(link).send().await?.text().await?;
        self.map_history_parser.parse(&body)
    }
}
