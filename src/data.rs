use steamid_ng::SteamID;
use time::Date;

#[derive(Debug, Clone)]
pub struct Player {
    pub name: String,
    pub steam_id: SteamID,
    pub honors: Vec<Honors>,
    pub teams: Vec<TeamMemberShip>,
}

#[derive(Debug, Clone)]
pub struct Honors {
    pub format: String,
    pub season: String,
    pub team: String,
}

#[derive(Debug, Clone)]
pub struct TeamMemberShip {
    pub team: TeamRef,
    pub league: String,
    pub since: Date,
}

#[derive(Debug, Clone)]
pub struct TeamRef {
    pub name: String,
    pub id: u32,
}

#[derive(Debug, Clone)]
pub struct MembershipHistory {
    pub team: TeamRef,
    pub division: String,
    pub joined: Date,
    pub left: Option<Date>,
}
