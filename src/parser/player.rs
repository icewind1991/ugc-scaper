use super::{ElementExt, Parser};
use crate::data::{Class, Honors, Player, TeamMemberShip, TeamRef};
use crate::parser::{select_last_text, select_text, team_id_from_link, DATE_FORMAT};
use crate::{ParseError, Result};
use scraper::{Html, Selector};
use std::iter::repeat;
use std::str::FromStr;
use steamid_ng::SteamID;
use time::Date;

const SELECTOR_PLAYER_NAME: &str = ".container .col-md-4 > h3 > b";
const SELECTOR_PLAYER_ID: &str = ".container .col-md-4 > p.nomargin";

const SELECTOR_PLAYER_HONORS_GROUP: &str =
    ".container .col-md-6:nth-child(2) .white-row-small .row-fluid";
const SELECTOR_PLAYER_HONORS_HEADER: &str = "h5";
const SELECTOR_PLAYER_HONORS_LEAGUE: &str = "li div";
const SELECTOR_PLAYER_HONORS_TEAM: &str = "li small";

const SELECTOR_PLAYER_TEAM_GROUP: &str =
    ".container .col-md-6:nth-child(1) .white-row-small .row-fluid";
const SELECTOR_PLAYER_TEAM_LINK: &str = "p a";
const SELECTOR_PLAYER_TEAM_NAME: &str = "span.text-primary b";
const SELECTOR_PLAYER_TEAM_LEAGUE: &str = "small";
const SELECTOR_PLAYER_TEAM_SINCE: &str = "small";

const SELECTOR_CLASS: &str = r#"img.img-rounded[src*="images/tf2/icon/"]"#;

pub struct PlayerParser {
    selector_name: Selector,
    selector_id: Selector,

    selector_honors_header: Selector,
    selector_honors_group: Selector,
    selector_honors_league: Selector,
    selector_honors_team: Selector,

    selector_team_group: Selector,
    selector_team_link: Selector,
    selector_team_name: Selector,
    selector_team_league: Selector,
    selector_team_since: Selector,

    selector_class: Selector,
}

impl Default for PlayerParser {
    fn default() -> Self {
        Self::new()
    }
}

impl PlayerParser {
    pub fn new() -> Self {
        PlayerParser {
            selector_name: Selector::parse(SELECTOR_PLAYER_NAME).unwrap(),
            selector_id: Selector::parse(SELECTOR_PLAYER_ID).unwrap(),

            selector_honors_header: Selector::parse(SELECTOR_PLAYER_HONORS_HEADER).unwrap(),
            selector_honors_group: Selector::parse(SELECTOR_PLAYER_HONORS_GROUP).unwrap(),
            selector_honors_league: Selector::parse(SELECTOR_PLAYER_HONORS_LEAGUE).unwrap(),
            selector_honors_team: Selector::parse(SELECTOR_PLAYER_HONORS_TEAM).unwrap(),

            selector_team_group: Selector::parse(SELECTOR_PLAYER_TEAM_GROUP).unwrap(),
            selector_team_link: Selector::parse(SELECTOR_PLAYER_TEAM_LINK).unwrap(),
            selector_team_name: Selector::parse(SELECTOR_PLAYER_TEAM_NAME).unwrap(),
            selector_team_league: Selector::parse(SELECTOR_PLAYER_TEAM_LEAGUE).unwrap(),
            selector_team_since: Selector::parse(SELECTOR_PLAYER_TEAM_SINCE).unwrap(),

            selector_class: Selector::parse(SELECTOR_CLASS).unwrap(),
        }
    }
}

impl Parser for PlayerParser {
    type Output = Player;

    fn parse(&self, document: &str) -> Result<Self::Output> {
        let document = Html::parse_document(document);
        let name = document
            .select(&self.selector_name)
            .next()
            .ok_or(ParseError::ElementNotFound {
                selector: SELECTOR_PLAYER_NAME,
                role: "player name",
            })?
            .first_text()
            .unwrap_or_default()
            .to_string();

        let id = document
            .select(&self.selector_id)
            .next()
            .ok_or(ParseError::ElementNotFound {
                selector: SELECTOR_PLAYER_ID,
                role: "player steam id",
            })?
            .nth_text(3)
            .unwrap_or_default()
            .to_string();

        let honors = document
            .select(&self.selector_honors_group)
            .flat_map(|group| {
                let format = select_text(group, &self.selector_honors_header)
                    .ok_or(ParseError::ElementNotFound {
                        selector: SELECTOR_PLAYER_HONORS_HEADER,
                        role: "player honors format",
                    })
                    .map(|format| format.trim_end_matches(" Medals"));
                let leagues = group.select(&self.selector_honors_league);
                let teams = group.select(&self.selector_honors_team);
                repeat(format).zip(leagues).zip(teams)
            })
            .map(|((format_res, season), team)| {
                let format = format_res?;
                Ok(Honors {
                    format: format.to_string(),
                    season: season
                        .first_text()
                        .ok_or(ParseError::EmptyText {
                            selector: SELECTOR_PLAYER_HONORS_LEAGUE,
                            role: "player honors season",
                        })?
                        .to_string(),
                    team: team
                        .first_text()
                        .ok_or(ParseError::EmptyText {
                            selector: SELECTOR_PLAYER_HONORS_TEAM,
                            role: "player honors team",
                        })?
                        .to_string(),
                })
            })
            .collect::<Result<Vec<_>>>()?;

        let favorite_classes = document
            .select(&self.selector_class)
            .filter_map(|class| class.attr("src"))
            .filter_map(|img| {
                img.strip_prefix("images/tf2/icon/")
                    .and_then(|class| class.strip_suffix(".jpg"))
            })
            .filter_map(|class| Class::from_str(class).ok())
            .collect::<Vec<_>>();

        let teams = document
            .select(&self.selector_team_group)
            .filter(|item| item.select(&self.selector_team_link).next().is_some())
            .map(move |item| {
                let link = item
                    .select(&self.selector_team_link)
                    .next()
                    .ok_or(ParseError::ElementNotFound {
                        selector: SELECTOR_PLAYER_TEAM_LINK,
                        role: "players team link",
                    })?
                    .attr("href")
                    .unwrap_or_default();
                let name = select_text(item, &self.selector_team_name).ok_or(
                    ParseError::ElementNotFound {
                        selector: SELECTOR_PLAYER_TEAM_NAME,
                        role: "players team name",
                    },
                )?;
                let league = select_text(item, &self.selector_team_league).ok_or(
                    ParseError::ElementNotFound {
                        selector: SELECTOR_PLAYER_TEAM_LEAGUE,
                        role: "players team league",
                    },
                )?;
                let since = select_last_text(item, &self.selector_team_since).ok_or(
                    ParseError::ElementNotFound {
                        selector: SELECTOR_PLAYER_TEAM_SINCE,
                        role: "players team joined",
                    },
                )?;

                let id = team_id_from_link(link)?;
                let since = match since.rsplit_once('\n') {
                    Some((_, since)) => {
                        Date::parse(since, DATE_FORMAT).map_err(|_| ParseError::InvalidDate {
                            role: "team join date",
                            date: since.to_string(),
                        })?
                    }
                    _ => {
                        return Err(ParseError::InvalidDate {
                            role: "team join date",
                            date: since.to_string(),
                        }
                        .into())
                    }
                };

                Ok(TeamMemberShip {
                    team: TeamRef {
                        name: name.to_string(),
                        id,
                    },
                    league: league.to_string(),
                    since,
                })
            })
            .collect::<Result<Vec<_>>>()?;

        Ok(Player {
            name,
            steam_id: SteamID::from_steam3(&id).unwrap_or_default(),
            honors,
            teams,
            favorite_classes,
        })
    }
}
