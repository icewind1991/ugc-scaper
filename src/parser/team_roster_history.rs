use super::Parser;
use crate::data::{RosterHistory, TeamRosterData, MembershipRole};
use crate::parser::{select_text, ROSTER_HISTORY_DATE_FORMAT};
use crate::{ParseError, Result};
use scraper::{Html, Selector};
use steamid_ng::SteamID;
use time::Date;

const SELECTOR_ROSTER_ITEM: &str =
    ".container .white-row-small .row-fluid > .col-md-12 > .clearfix";
const SELECTOR_ROSTER_NAME: &str = "h5 b";
const SELECTOR_ROSTER_ID: &str = "h5 small";
const SELECTOR_ROSTER_ROLE: &str = "div > small";
const SELECTOR_ROSTER_JOINED: &str = "span.text-success small";
const SELECTOR_ROSTER_LEFT: &str = "span.text-danger small";

const SELECTOR_STEAM: &str = r#"p.muted a[href*="//steamcommunity.com/groups"]"#;

pub struct TeamRosterHistoryParser {
    selector_item: Selector,
    selector_name: Selector,
    selector_id: Selector,
    selector_joined: Selector,
    selector_left: Selector,
    selector_steam_group: Selector,
    selector_role: Selector,
}

impl Default for TeamRosterHistoryParser {
    fn default() -> Self {
        Self::new()
    }
}

impl TeamRosterHistoryParser {
    pub fn new() -> Self {
        TeamRosterHistoryParser {
            selector_item: Selector::parse(SELECTOR_ROSTER_ITEM).unwrap(),
            selector_name: Selector::parse(SELECTOR_ROSTER_NAME).unwrap(),
            selector_id: Selector::parse(SELECTOR_ROSTER_ID).unwrap(),
            selector_joined: Selector::parse(SELECTOR_ROSTER_JOINED).unwrap(),
            selector_left: Selector::parse(SELECTOR_ROSTER_LEFT).unwrap(),
            selector_steam_group: Selector::parse(SELECTOR_STEAM).unwrap(),
            selector_role: Selector::parse(SELECTOR_ROSTER_ROLE).unwrap(),
        }
    }
}

impl Parser for TeamRosterHistoryParser {
    type Output = TeamRosterData;

    fn parse(&self, document: &str) -> Result<Self::Output> {
        let document = Html::parse_document(document);

        let steam_group = document.select(&self.selector_steam_group).next();
        let steam_group = steam_group
            .and_then(|link| link.attr("href"))
            .map(|href| href.replace("http://http", "http"));

        let history = document
            .select(&self.selector_item)
            .map(|item| {
                let name =
                    select_text(item, &self.selector_name).ok_or(ParseError::ElementNotFound {
                        selector: SELECTOR_ROSTER_NAME,
                        role: "member name",
                    })?;
                let steam_id =
                    select_text(item, &self.selector_id).ok_or(ParseError::ElementNotFound {
                        selector: SELECTOR_ROSTER_ID,
                        role: "member steam id",
                    })?;
                let joined = select_text(item, &self.selector_joined).ok_or(
                    ParseError::ElementNotFound {
                        selector: SELECTOR_ROSTER_JOINED,
                        role: "member joined date",
                    },
                )?;
                let left = select_text(item, &self.selector_left);
                let role = select_text(item, &self.selector_role)
                    .ok_or(ParseError::ElementNotFound {
                        selector: SELECTOR_ROSTER_ROLE,
                        role: "member role",
                    })?
                    .trim_start_matches("Former ")
                    .parse::<MembershipRole>().unwrap_or(MembershipRole::Member);

                Ok(RosterHistory {
                    name: name.to_string(),
                    steam_id: SteamID::from_steam3(steam_id).map_err(|_| {
                        ParseError::InvalidText {
                            text: steam_id.to_string(),
                            role: "member steam id",
                        }
                    })?,
                    joined: Date::parse(joined, ROSTER_HISTORY_DATE_FORMAT).map_err(|_| {
                        ParseError::InvalidDate {
                            date: steam_id.to_string(),
                            role: "member join date",
                        }
                    })?,
                    left: left
                        .map(|left| {
                            Date::parse(left, ROSTER_HISTORY_DATE_FORMAT).map_err(|_| {
                                ParseError::InvalidDate {
                                    date: steam_id.to_string(),
                                    role: "member join date",
                                }
                            })
                        })
                        .transpose()?,
                    role,
                })
            })
            .collect::<Result<Vec<_>>>()?;
        Ok(TeamRosterData {
            history,
            steam_group,
        })
    }
}
