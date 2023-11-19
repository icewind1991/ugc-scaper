use crate::ParseError;
use std::str::FromStr;
pub use steamid_ng::SteamID;
use time::{Date, OffsetDateTime};

#[cfg(feature = "serde")]
mod serde_date {
    use serde::ser::Error as _;
    use serde::{Serialize, Serializer};
    use time::format_description::FormatItem;
    use time::macros::format_description;
    use time::Date;

    const DATE_FORMAT: &[FormatItem<'static>] = format_description!("[year]-[month]-[day]");

    pub fn serialize<S: Serializer>(date: &Date, serializer: S) -> Result<S::Ok, S::Error> {
        date.format(DATE_FORMAT)
            .map_err(S::Error::custom)?
            .serialize(serializer)
    }

    pub mod opt {
        use super::DATE_FORMAT;
        use serde::ser::Error as _;
        use serde::{Serialize, Serializer};
        use time::Date;

        pub fn serialize<S: Serializer>(
            option: &Option<Date>,
            serializer: S,
        ) -> Result<S::Ok, S::Error> {
            option
                .map(|odt| odt.format(DATE_FORMAT))
                .transpose()
                .map_err(S::Error::custom)?
                .serialize(serializer)
        }
    }
}

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
    #[cfg_attr(feature = "serde", serde(with = "serde_date"))]
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
    #[cfg_attr(feature = "serde", serde(with = "serde_date"))]
    pub joined: Date,
    #[cfg_attr(feature = "serde", serde(with = "serde_date::opt"))]
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
    #[cfg_attr(feature = "serde", serde(with = "serde_date"))]
    pub date: Date,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Membership {
    pub name: String,
    pub steam_id: SteamID,
    pub role: String,
    #[cfg_attr(feature = "serde", serde(with = "time::serde::iso8601"))]
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
    #[cfg_attr(feature = "serde", serde(with = "serde_date"))]
    pub joined: Date,
    #[cfg_attr(feature = "serde", serde(with = "serde_date::opt"))]
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
        id: u32,
        opponent: TeamRef,
        score: u8,
        score_opponent: u8,
        match_points: f32,
        match_points_opponent: f32,
    },
    Pending {
        id: u32,
        opponent: TeamRef,
        score: u8,
        score_opponent: u8,
    },
    ByeWeek,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Seasons {
    pub mode: String,
    pub seasons: Vec<Season>,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Season {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct MatchInfo {
    pub comment: Option<String>,
    pub comment_author: Option<String>,
    pub team_home: TeamRef,
    pub team_away: TeamRef,
    pub score_home: u8,
    pub score_away: u8,
}

pub enum GameMode {
    Highlander,
    Sixes,
    Fours,
    Ultiduo,
}

impl FromStr for GameMode {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "9v9" => Ok(GameMode::Highlander),
            "6v6" => Ok(GameMode::Sixes),
            "4v4" => Ok(GameMode::Fours),
            "2v2" => Ok(GameMode::Ultiduo),
            _ => Err(()),
        }
    }
}

impl GameMode {
    pub fn letter(&self) -> char {
        match self {
            GameMode::Highlander => 'h',
            GameMode::Sixes => '6',
            GameMode::Fours => '4',
            GameMode::Ultiduo => '2',
        }
    }
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Transaction {
    pub name: String,
    pub steam_id: SteamID,
    pub action: TranactionAction,
    pub team: TeamRef,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum TranactionAction {
    Joined,
    Left,
}

impl FromStr for TranactionAction {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Joined" => Ok(TranactionAction::Joined),
            "Left" => Ok(TranactionAction::Left),
            _ => Err(ParseError::InvalidText {
                role: "transaction action",
                text: s.to_string(),
            }),
        }
    }
}
