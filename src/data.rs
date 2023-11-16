use steamid_ng::SteamID;
use time::Date;

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
