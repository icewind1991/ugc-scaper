use reqwest::{Client, ClientBuilder, Error, Response, StatusCode};
use serde::de::DeserializeOwned;
use thiserror::Error;
use ugc_scraper_types::{
    GameMode, MapHistory, MatchInfo, MembershipHistory, Player, RosterHistory, SteamID, Team,
    TeamRosterData, TeamSeason, TeamSeasonMatch, Transaction,
};

#[derive(Debug, Error)]
pub enum UgcClientError {
    #[error("Error sending request to {endpoint:?}: {error:#}")]
    Request { endpoint: Endpoint, error: Error },
    #[error("Error parsing response from {endpoint:?}: {error:#}")]
    Response { endpoint: Endpoint, error: Error },
    #[error("{endpoint:?} not found")]
    NotFound { endpoint: Endpoint },
}

#[derive(Debug)]
pub struct UgcClient {
    client: Client,
    api_url: String,
}

#[allow(dead_code)]
impl UgcClient {
    pub fn new(api_url: String) -> Self {
        let client = ClientBuilder::new()
            .user_agent("UGC_ARCHIVER")
            .build()
            .expect("failed to build client");
        Self { client, api_url }
    }

    async fn send_request<T: DeserializeOwned>(
        &self,
        endpoint: Endpoint,
    ) -> Result<T, UgcClientError> {
        self.client
            .get(endpoint.build_url(&self.api_url))
            .send()
            .await
            .map_err(|error| UgcClientError::Request { endpoint, error })?
            .check_not_found(endpoint)?
            .json()
            .await
            .map_err(|error| UgcClientError::Response { endpoint, error })
    }

    pub async fn get_match(&self, id: u32) -> Result<MatchInfo, UgcClientError> {
        self.send_request(Endpoint::Match { id }).await
    }

    pub async fn get_team(&self, id: u32) -> Result<Team, UgcClientError> {
        self.send_request(Endpoint::Team { id }).await
    }

    pub async fn get_team_roster(&self, id: u32) -> Result<Vec<RosterHistory>, UgcClientError> {
        self.send_request::<TeamRosterData>(Endpoint::TeamRoster { id })
            .await
            .map(|data| data.history)
    }

    pub async fn get_team_matches(&self, id: u32) -> Result<TeamSeason, UgcClientError> {
        self.send_request(Endpoint::TeamMatches { id }).await
    }

    pub async fn get_player(&self, id: SteamID) -> Result<Player, UgcClientError> {
        self.send_request(Endpoint::Player { id }).await
    }

    pub async fn get_player_history(
        &self,
        id: SteamID,
    ) -> Result<MembershipHistory, UgcClientError> {
        self.send_request(Endpoint::PlayerHistory { id }).await
    }

    pub async fn get_maps(&self, format: GameMode) -> Result<MapHistory, UgcClientError> {
        self.send_request(Endpoint::Maps { format }).await
    }

    pub async fn get_transactions(
        &self,
        format: GameMode,
    ) -> Result<Vec<Transaction>, UgcClientError> {
        self.send_request(Endpoint::Transactions { format }).await
    }
}

#[derive(Debug, Copy, Clone)]
pub enum Endpoint {
    Match { id: u32 },
    Player { id: SteamID },
    PlayerHistory { id: SteamID },
    Transactions { format: GameMode },
    Team { id: u32 },
    TeamRoster { id: u32 },
    TeamMatches { id: u32 },
    Maps { format: GameMode },
}

impl Endpoint {
    pub fn build_url(&self, api_url: &str) -> String {
        match self {
            Endpoint::Match { id } => format!("{}/match/{id}", api_url),
            Endpoint::Player { id } => format!("{}/player/{}", api_url, u64::from(*id)),
            Endpoint::PlayerHistory { id } => {
                format!("{}/player/{}/history", api_url, u64::from(*id))
            }
            Endpoint::Transactions { format } => format!("{}/transactions/{format}", api_url),
            Endpoint::Team { id } => format!("{}/team/{id}", api_url),
            Endpoint::TeamRoster { id } => format!("{}/team/{id}/roster", api_url),
            Endpoint::TeamMatches { id } => format!("{}/team/{id}/matches", api_url),
            Endpoint::Maps { format } => format!("{}/maps/{format}", api_url),
        }
    }
}

trait ResponseExt: Sized {
    fn check_not_found(self, endpoint: Endpoint) -> Result<Self, UgcClientError>;
}

impl ResponseExt for Response {
    fn check_not_found(self, endpoint: Endpoint) -> Result<Self, UgcClientError> {
        if self.status() == StatusCode::NOT_FOUND {
            Err(UgcClientError::NotFound { endpoint })
        } else {
            Ok(self)
        }
    }
}
