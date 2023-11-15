use super::{ElementExt, Parser};
use crate::data::{MembershipHistory, TeamRef};
use crate::parser::select_text;
use crate::Result;
use scraper::{Html, Selector};
use time::{macros::format_description, Date};

const SELECTOR_TEAM_FORMAT: &str = ".container .white-row-small thead h4";
const SELECTOR_TEAM_GROUP: &str = ".container .white-row-small tbody";
const TEAM_ROW: &str = "tr:not(:first-child)";
const SELECTOR_TEAM_LINK: &str = "td:nth-child(3) a";
const SELECTOR_TEAM_DIVISION: &str = "td:nth-child(3) small";
const SELECTOR_TEAM_JOINED: &str = "td:nth-child(5) span";
const SELECTOR_TEAM_LEFT: &str = "td:nth-child(6) span";

pub struct PlayerDetailsParser {
    selector_team_format: Selector,
    selector_team_group: Selector,
    selector_team_row: Selector,
    selector_team_link: Selector,
    selector_team_division: Selector,
    selector_team_joined: Selector,
    selector_team_left: Selector,
}

impl PlayerDetailsParser {
    pub fn new() -> Self {
        PlayerDetailsParser {
            selector_team_format: Selector::parse(SELECTOR_TEAM_FORMAT).unwrap(),
            selector_team_group: Selector::parse(SELECTOR_TEAM_GROUP).unwrap(),
            selector_team_row: Selector::parse(TEAM_ROW).unwrap(),
            selector_team_link: Selector::parse(SELECTOR_TEAM_LINK).unwrap(),
            selector_team_division: Selector::parse(SELECTOR_TEAM_DIVISION).unwrap(),
            selector_team_joined: Selector::parse(SELECTOR_TEAM_JOINED).unwrap(),
            selector_team_left: Selector::parse(SELECTOR_TEAM_LEFT).unwrap(),
        }
    }
}

impl Parser for PlayerDetailsParser {
    type Output = Vec<MembershipHistory>;

    fn parse(&self, document: &str) -> Result<Self::Output> {
        let document = Html::parse_document(&document);

        Ok(document
            .select(&self.selector_team_format)
            .zip(document.select(&self.selector_team_group))
            .flat_map(|(format, history)| {
                history
                    .select(&self.selector_team_row)
                    .map(move |row| (format, row))
            })
            .map(|(format, team)| {
                let format = format.first_text();
                let link = team
                    .select(&self.selector_team_link)
                    .next()
                    .and_then(|link| link.attr("href"))
                    .unwrap_or_default();
                let name = select_text(team, &self.selector_team_link, "failed to find team name");
                let division =
                    select_text(team, &self.selector_team_joined, "failed to find division");
                let joined = select_text(team, &self.selector_team_joined, "");
                let left = select_text(team, &self.selector_team_left, "");

                let id = match link.rsplit_once("=") {
                    Some((_, id)) => id.parse().unwrap_or_default(),
                    _ => 0,
                };
                let format = format_description!("[month padding:none]/[day padding:none]/[year]");

                MembershipHistory {
                    joined: Date::parse(joined, format).unwrap_or(Date::MIN),
                    left: Date::parse(left, format).ok(),
                    team: TeamRef {
                        name: name.to_string(),
                        id,
                    },
                    division: division.to_string(),
                }
            })
            .collect())
    }
}
