use super::{ElementExt, Parser};
use crate::data::{Class, GameMode, Honors, Player, TeamMemberShip, TeamRef};
use crate::parser::{select_last_text, select_text, team_id_from_link, DATE_FORMAT};
use crate::{ParseError, Result};
use scraper::{Html, Selector};
use std::iter::repeat;
use std::str::FromStr;
use steamid_ng::SteamID;
use time::Date;

const SELECTOR_PLAYER_NAME: &str = ".container .col-md-4 > h3 > b";
const SELECTOR_PLAYER_ID: &str = r#"a[href*="steam://friends/add"]"#;
const SELECTOR_PLAYER_FLAG: &str = r#"img[data-cfsrc*="/images/flags/"]"#;

const SELECTOR_PLAYER_HONORS_GROUP: &str =
    ".container .col-md-6:nth-child(2) .white-row-small .row-fluid";
const SELECTOR_PLAYER_HONORS_HEADER: &str = "h5";
const SELECTOR_PLAYER_HONORS_LEAGUE: &str = "li div";
const SELECTOR_PLAYER_HONORS_TEAM: &str = "li small a";

const SELECTOR_PLAYER_TEAM_GROUP: &str =
    ".container .col-md-6:nth-child(1) .white-row-small .row-fluid";
const SELECTOR_PLAYER_TEAM_LINK: &str = "p a";
const SELECTOR_PLAYER_TEAM_NAME: &str = "span.text-primary b";
const SELECTOR_PLAYER_TEAM_LEAGUE: &str = "small";
const SELECTOR_PLAYER_TEAM_SINCE: &str = "small";

const SELECTOR_AVATAR: &str =
    r#"a[href*="https://www.ugcleague.com/players_page.cfm?player_id="] img.img-responsive"#;
const SELECTOR_CLASS: &str =
    r#"img.img-rounded[src*="images/tf2/icon/"], img.img-rounded[data-cfsrc*="images/tf2/icon/"]"#;

pub struct PlayerParser {
    selector_name: Selector,
    selector_id: Selector,
    selector_flag: Selector,

    selector_honors_header: Selector,
    selector_honors_group: Selector,
    selector_honors_league: Selector,
    selector_honors_team: Selector,

    selector_team_group: Selector,
    selector_team_link: Selector,
    selector_team_name: Selector,
    selector_team_league: Selector,
    selector_team_since: Selector,

    selector_avatar: Selector,
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
            selector_flag: Selector::parse(SELECTOR_PLAYER_FLAG).unwrap(),

            selector_honors_header: Selector::parse(SELECTOR_PLAYER_HONORS_HEADER).unwrap(),
            selector_honors_group: Selector::parse(SELECTOR_PLAYER_HONORS_GROUP).unwrap(),
            selector_honors_league: Selector::parse(SELECTOR_PLAYER_HONORS_LEAGUE).unwrap(),
            selector_honors_team: Selector::parse(SELECTOR_PLAYER_HONORS_TEAM).unwrap(),

            selector_team_group: Selector::parse(SELECTOR_PLAYER_TEAM_GROUP).unwrap(),
            selector_team_link: Selector::parse(SELECTOR_PLAYER_TEAM_LINK).unwrap(),
            selector_team_name: Selector::parse(SELECTOR_PLAYER_TEAM_NAME).unwrap(),
            selector_team_league: Selector::parse(SELECTOR_PLAYER_TEAM_LEAGUE).unwrap(),
            selector_team_since: Selector::parse(SELECTOR_PLAYER_TEAM_SINCE).unwrap(),

            selector_avatar: Selector::parse(SELECTOR_AVATAR).unwrap(),
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
            .attr("href")
            .unwrap_or_default()
            .strip_prefix("steam://friends/add/")
            .unwrap_or_default()
            .to_string();

        let country = document
            .select(&self.selector_flag)
            .next()
            .and_then(|e| e.attr("title"))
            .map(String::from);

        let avatar_element =
            document
                .select(&self.selector_avatar)
                .next()
                .ok_or(ParseError::ElementNotFound {
                    selector: SELECTOR_AVATAR,
                    role: "player avatar",
                })?;
        let avatar = avatar_element
            .attr("src")
            .or_else(|| avatar_element.attr("data-cfsrc"))
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
                let game_mode =
                    GameMode::from_str(format).map_err(|_| ParseError::InvalidText {
                        text: format.into(),
                        role: "player honors format",
                    })?;
                let division = season.first_text().ok_or(ParseError::EmptyText {
                    selector: SELECTOR_PLAYER_HONORS_LEAGUE,
                    role: "player honors division",
                })?;
                let season = division
                    .split(' ')
                    .nth(1)
                    .unwrap_or_default()
                    .parse()
                    .map_err(|_| ParseError::InvalidText {
                        text: division.into(),
                        role: "player honors season",
                    })?;
                let team_name = team.first_text().unwrap_or_default().to_string();
                let team_link = team.attr("href").unwrap_or_default();
                let team_id: u32 = team_link
                    .rsplit('=')
                    .next()
                    .unwrap_or_default()
                    .parse()
                    .map_err(|_| ParseError::InvalidLink {
                        link: team_link.into(),
                        role: "player honors team",
                    })?;

                Ok(Honors {
                    format: game_mode,
                    division: division.splitn(3, ' ').last().unwrap_or_default().into(),
                    season,
                    team: TeamRef {
                        id: team_id,
                        name: team_name,
                    },
                })
            })
            .collect::<Result<Vec<_>>>()?;

        let favorite_classes = document
            .select(&self.selector_class)
            .filter_map(|class| class.attr("src").or_else(|| class.attr("data-cfsrc")))
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
                let name = select_text(item, &self.selector_team_name).unwrap_or_default();
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
                let since = match since.rsplit_once(['\n', ' ', '\t']) {
                    Some((_, since)) => {
                        let since = since.trim();
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
            avatar,
            steam_id: SteamID::try_from(id.as_str()).unwrap_or_default(),
            honors,
            teams,
            favorite_classes,
            country,
        })
    }
}
