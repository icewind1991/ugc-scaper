use super::{ElementExt, Parser};
use crate::data::{Honors, Player, TeamMemberShip, TeamRef};
use crate::{ParseError, Result};
use scraper::{ElementRef, Html, Selector};
use std::iter::repeat;
use steamid_ng::SteamID;
use time::{macros::format_description, Date};

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
        }
    }
}

fn select_text<'a>(el: ElementRef<'a>, selector: &Selector, default: &'static str) -> &'a str {
    el.select(selector)
        .next()
        .and_then(|item| item.text().filter(|s| !s.trim().is_empty()).next())
        .unwrap_or(default)
        .trim()
}

fn select_last_text<'a>(el: ElementRef<'a>, selector: &Selector, default: &'static str) -> &'a str {
    el.select(selector)
        .next()
        .and_then(|item| item.text().last())
        .unwrap_or(default)
        .trim()
}

impl Parser for PlayerParser {
    type Output = Player;

    fn parse(&self, document: &str) -> Result<Self::Output> {
        let document = Html::parse_document(&document);

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
                let format =
                    select_text(group, &self.selector_honors_header, "format not detected")
                        .trim_end_matches(" Medals");
                let leagues = group.select(&self.selector_honors_league);
                let teams = group.select(&self.selector_honors_team);
                repeat(format).zip(leagues).zip(teams)
            })
            .map(|((format, season), team)| Honors {
                format: format.to_string(),
                season: season.text().next().unwrap_or_default().trim().to_string(),
                team: team.text().next().unwrap_or_default().trim().to_string(),
            })
            .collect();

        let teams = document
            .select(&self.selector_team_group)
            .filter(|item| item.select(&self.selector_team_link).next().is_some())
            .map(|item| {
                let link = item
                    .select(&self.selector_team_link)
                    .next()
                    .and_then(|link| link.attr("href"))
                    .unwrap_or("=0");
                let name = select_text(item, &self.selector_team_name, "failed to find name");
                let league = select_text(item, &self.selector_team_league, "failed to find league");
                let since = select_last_text(item, &self.selector_team_since, "");

                let id = match link.rsplit_once("=") {
                    Some((_, id)) => id.parse().unwrap_or_default(),
                    _ => 0,
                };
                let format = format_description!("[month padding:none]/[day padding:none]/[year]");
                let since = match since.rsplit_once("\n") {
                    Some((_, since)) => Date::parse(since, &format).unwrap_or(Date::MIN),
                    _ => Date::MIN,
                };

                TeamMemberShip {
                    team: TeamRef {
                        name: name.to_string(),
                        id,
                    },
                    league: league.to_string(),
                    since,
                }
            })
            .collect();

        Ok(Player {
            name,
            steam_id: SteamID::from_steam3(&id).unwrap_or_default(),
            honors,
            teams,
        })
    }
}
