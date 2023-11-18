pub use steamid_ng::SteamID;
use time::{Date, OffsetDateTime};

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Player {
    pub name: String,
    pub steam_id: SteamID,
    pub honors: Vec<Honors>,
    pub teams: Vec<TeamMemberShip>,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Honors {
    pub format: String,
    pub season: String,
    pub team: String,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct TeamMemberShip {
    pub team: TeamRef,
    pub league: String,
    pub since: Date,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct TeamRef {
    pub name: String,
    pub id: u32,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct MembershipHistory {
    pub format: String,
    pub team: TeamRef,
    pub division: String,
    pub joined: Date,
    pub left: Option<Date>,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Team {
    pub name: String,
    pub tag: String,
    pub image: String,
    pub format: String,
    pub timezone: String,
    pub division: String,
    pub description: String,
    pub titles: Vec<String>,
    pub members: Vec<Membership>,
    pub results: Vec<Record>,
    pub name_changes: Vec<NameChange>,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct NameChange {
    pub from_tag: String,
    pub from: String,
    pub to_tag: String,
    pub to: String,
    pub date: Date,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Membership {
    pub name: String,
    pub steam_id: SteamID,
    pub role: String,
    pub since: OffsetDateTime,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Record {
    pub season: u32,
    pub division: String,
    pub wins: u8,
    pub losses: u8,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct RosterHistory {
    pub name: String,
    pub steam_id: SteamID,
    pub joined: Date,
    pub left: Option<Date>,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct TeamSeason {
    pub season: u32,
    pub matches: Vec<TeamSeasonMatch>,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct TeamSeasonMatch {
    pub division: String,
    pub week: u8,
    pub date: String,
    pub side: String,
    pub result: MatchResult,
    pub map: String,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum MatchResult {
    Played {
        opponent: TeamRef,
        score: u8,
        score_opponent: u8,
        match_points: f32,
        match_points_opponent: f32,
    },
    Pending {
        opponent: TeamRef,
        score: u8,
        score_opponent: u8,
    },
    ByeWeek,
}
