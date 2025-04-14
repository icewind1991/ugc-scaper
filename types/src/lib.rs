use serde::de::Error;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt::Display;
use std::str::FromStr;
pub use steamid_ng::SteamID;
use thiserror::Error;
use time::{Date, OffsetDateTime};

#[cfg(feature = "serde")]
mod serde_date {
    use serde::de::Error;
    use serde::ser::Error as _;
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use time::format_description::FormatItem;
    use time::macros::format_description;
    use time::Date;

    const DATE_FORMAT: &[FormatItem<'static>] = format_description!("[year]-[month]-[day]");

    pub fn serialize<S: Serializer>(date: &Date, serializer: S) -> Result<S::Ok, S::Error> {
        date.format(DATE_FORMAT)
            .map_err(S::Error::custom)?
            .serialize(serializer)
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Date, D::Error> {
        let str = <&str>::deserialize(deserializer)?;
        Date::parse(str, DATE_FORMAT).map_err(D::Error::custom)
    }

    pub mod opt {
        use super::DATE_FORMAT;
        use serde::de::Error;
        use serde::ser::Error as _;
        use serde::{Deserialize, Deserializer, Serialize, Serializer};
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

        pub fn deserialize<'de, D: Deserializer<'de>>(
            deserializer: D,
        ) -> Result<Option<Date>, D::Error> {
            let str = <Option<&str>>::deserialize(deserializer)?;
            match str {
                Some(str) => Date::parse(str, DATE_FORMAT)
                    .map_err(D::Error::custom)
                    .map(Some),
                None => Ok(None),
            }
        }
    }
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Player {
    pub name: String,
    pub avatar: String,
    #[cfg_attr(feature = "serde", serde(with = "serde_steam_id_as_string"))]
    pub steam_id: SteamID,
    pub honors: Vec<Honors>,
    pub teams: Vec<TeamMemberShip>,
    pub favorite_classes: Vec<Class>,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Honors {
    pub format: String,
    pub season: String,
    pub team: String,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TeamMemberShip {
    pub team: TeamRef,
    pub league: String,
    #[cfg_attr(feature = "serde", serde(with = "serde_date"))]
    pub since: Date,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TeamRef {
    pub name: String,
    pub id: u32,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
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
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Team {
    pub name: String,
    pub tag: String,
    pub image: String,
    pub format: GameMode,
    pub region: Region,
    pub timezone: Option<String>,
    pub steam_group: Option<String>,
    pub division: String,
    pub description: String,
    pub titles: Vec<String>,
    pub members: Vec<Membership>,
    pub results: Vec<Record>,
    pub name_changes: Vec<NameChange>,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "lowercase"))]
pub enum Class {
    Scout,
    Soldier,
    Pyro,
    Demoman,
    Heavy,
    Medic,
    Sniper,
    Spy,
}

impl FromStr for Class {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "scout" => Ok(Class::Scout),
            "soldier" => Ok(Class::Soldier),
            "pyro" => Ok(Class::Pyro),
            "demoman" => Ok(Class::Demoman),
            "heavy" => Ok(Class::Heavy),
            "medic" => Ok(Class::Medic),
            "sniper" => Ok(Class::Sniper),
            "spy" => Ok(Class::Spy),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct NameChange {
    pub from_tag: String,
    pub from: String,
    pub to_tag: String,
    pub to: String,
    #[cfg_attr(feature = "serde", serde(with = "serde_date"))]
    pub date: Date,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Membership {
    pub name: String,
    #[cfg_attr(feature = "serde", serde(with = "serde_steam_id_as_string"))]
    pub steam_id: SteamID,
    pub role: String,
    #[cfg_attr(feature = "serde", serde(with = "time::serde::iso8601"))]
    pub since: OffsetDateTime,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Record {
    pub season: u32,
    pub division: String,
    pub wins: u8,
    pub losses: u8,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TeamRosterData {
    pub steam_group: Option<String>,
    pub history: Vec<RosterHistory>,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct RosterHistory {
    pub name: String,
    #[cfg_attr(feature = "serde", serde(with = "serde_steam_id_as_string"))]
    pub steam_id: SteamID,
    #[cfg_attr(feature = "serde", serde(with = "serde_date"))]
    pub joined: Date,
    #[cfg_attr(feature = "serde", serde(with = "serde_date::opt"))]
    pub left: Option<Date>,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TeamSeason {
    pub season: u32,
    pub matches: Vec<TeamSeasonMatch>,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TeamSeasonMatch {
    pub division: String,
    pub week: u8,
    pub date: String,
    pub side: String,
    pub result: MatchResult,
    pub map: String,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
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
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Seasons {
    pub mode: String,
    pub seasons: Vec<Season>,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Season {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct MatchInfo {
    pub comment: Option<String>,
    pub comment_author: Option<String>,
    pub team_home: TeamRef,
    pub team_away: TeamRef,
    pub score_home: u8,
    pub score_away: u8,
}

#[derive(Debug, Clone, Error)]
#[error("Invalid game mode {text}")]
pub struct InvalidGameMode {
    pub text: String,
}

#[derive(Debug, Clone, Copy)]
pub enum GameMode {
    Highlander,
    Sixes,
    Fours,
    Ultiduo,
}

impl FromStr for GameMode {
    type Err = InvalidGameMode;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "9v9" => Ok(GameMode::Highlander),
            "6v6" => Ok(GameMode::Sixes),
            "4v4" => Ok(GameMode::Fours),
            "2v2" => Ok(GameMode::Ultiduo),
            "TF2 Highlander" => Ok(GameMode::Highlander),
            "ASIA TF2-H" => Ok(GameMode::Highlander),
            "TF2 6vs6" => Ok(GameMode::Sixes),
            "TF2 4vs4" => Ok(GameMode::Fours),
            "TF2 2vs2" => Ok(GameMode::Ultiduo),
            _ => Err(InvalidGameMode {
                text: s.to_string(),
            }),
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

    fn as_str(&self) -> &'static str {
        match self {
            GameMode::Highlander => "9v9",
            GameMode::Sixes => "6v6",
            GameMode::Fours => "4v4",
            GameMode::Ultiduo => "2v2",
        }
    }
}

impl Serialize for GameMode {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.as_str().serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for GameMode {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = <&str>::deserialize(deserializer)?;
        Self::from_str(s).map_err(D::Error::custom)
    }
}

impl Display for GameMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, Error)]
#[error("Invalid team region: {text}")]
pub struct InvalidRegion {
    pub text: String,
}

#[derive(Serialize, Deserialize, Copy, Clone, Debug)]
pub enum Region {
    Europe,
    NorthAmerica,
    SouthAmerica,
    Asia,
    Australia,
}

impl FromStr for Region {
    type Err = InvalidRegion;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Euro" => Ok(Region::Europe),
            "Europe" => Ok(Region::Europe),
            "EU" => Ok(Region::Europe),
            "Asia" => Ok(Region::Asia),
            "ASIA" => Ok(Region::Asia),
            "NA" => Ok(Region::NorthAmerica),
            "North America" => Ok(Region::NorthAmerica),
            "N.Amer" => Ok(Region::NorthAmerica),
            "South American" => Ok(Region::SouthAmerica),
            "SA" => Ok(Region::SouthAmerica),
            "AUS" => Ok(Region::Australia),
            "AUS/NZ" => Ok(Region::Australia),
            _ => Err(InvalidRegion {
                text: s.to_string(),
            }),
        }
    }
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Transaction {
    pub name: String,
    #[cfg_attr(feature = "serde", serde(with = "serde_steam_id_as_string"))]
    pub steam_id: SteamID,
    pub action: TransactionAction,
    pub team: TeamRef,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum TransactionAction {
    Joined,
    Left,
}

/// Tried to parse in invalid transaction action
pub struct MallFormedTransaction {
    pub text: String,
}

impl FromStr for TransactionAction {
    type Err = MallFormedTransaction;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Joined" => Ok(TransactionAction::Joined),
            "Left" => Ok(TransactionAction::Left),
            _ => Err(MallFormedTransaction {
                text: s.to_string(),
            }),
        }
    }
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct MapHistory {
    pub current: CurrentSeasonMapList,
    pub previous: Vec<PreviousSeasonMapList>,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CurrentSeasonMapList {
    pub season: u8,
    pub maps: Vec<CurrentSeasonMap>,
}
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PreviousSeasonMapList {
    pub season: u8,
    pub maps: Vec<PreviousSeasonMap>,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CurrentSeasonMap {
    pub week: u8,
    pub map: String,
    pub date: String,
    pub na_date: Option<String>,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PreviousSeasonMap {
    pub week: u8,
    pub map: String,
    #[cfg_attr(feature = "serde", serde(with = "serde_date"))]
    pub date: Date,
}

#[cfg(feature = "serde")]
mod serde_steam_id_as_string {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use steamid_ng::SteamID;

    pub fn serialize<S: Serializer>(steam_id: &SteamID, serializer: S) -> Result<S::Ok, S::Error> {
        let id = u64::from(*steam_id);
        format!("{id}").serialize(serializer)
    }

    #[allow(dead_code)]
    pub fn deserialize<'de, D>(deserializer: D) -> Result<SteamID, D::Error>
    where
        D: Deserializer<'de>,
    {
        let id = u64::deserialize(deserializer)?;
        Ok(SteamID::from(id))
    }
}
