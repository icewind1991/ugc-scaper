use super::{ElementExt, Parser};
use crate::data::{Membership, NameChange, Record, Team};
use crate::parser::{
    select_text, steam_id_from_link, DATE_FORMAT, MEMBER_DATE_ALT_FORMAT, MEMBER_DATE_FORMAT,
};
use crate::{ParseError, Result, ScrapeError};
use scraper::{Html, Selector};
use time::{Date, PrimitiveDateTime, Time, UtcOffset};
use ugc_scraper_types::GameMode;

const SELECTOR_TEAM_NAME: &str = ".container .col-md-12 h1 > b";
const SELECTOR_TEAM_TAG: &str = ".container .col-md-12 h1 > span";
const SELECTOR_TEAM_IMAGE: &str = ".container .col-md-12 a > img";

const SELECTOR_TEAM_FORMAT: &str = ".container .col-md-3 .white-row-small h5 .text-danger b";
const SELECTOR_TEAM_DIVISION: &str = ".container .col-md-3 .white-row-small h5 > b";
const SELECTOR_TEAM_TIMEZONE: &str = ".container .col-md-3 .white-row-small p > small > b";
const SELECTOR_TEAM_DESCRIPTION: &str =
    ".container .col-md-3 .white-row-small p:nth-child(4) > small";
const SELECTOR_TEAM_TITLES: &str = ".container .col-md-3 .white-row-small p > .text-warning";

const SELECTOR_TEAM_MEMBER_ROW: &str =
    ".container .white-row-small > .row-fluid > .col-md-12 > .white-row-light-small";
const SELECTOR_TEAM_MEMBER_LINK: &str = "b > a[href^=\"players_page\"]";
const SELECTOR_TEAM_MEMBER_ROLE: &str = ".tinytext";
const SELECTOR_TEAM_MEMBER_SINCE: &str = ".tinytext > em";

const SELECTOR_TEAM_RECORDS: &str =
    ".container .col-md-3 .white-row-small .table-responsive > table tbody tr";
const SELECTOR_TEAM_RECORD_SEASON: &str = "td:nth-child(1) small span b";
const SELECTOR_TEAM_RECORD_DIVISION: &str = "td:nth-child(2) small";
const SELECTOR_TEAM_RECORD_RESULT: &str = "td:nth-child(3)";

const SELECTOR_TEAM_NAME_CHANGE: &str =
    ".white-row-small:nth-child(3) .table-responsive table tbody tr";
const SELECTOR_TEAM_NAME_FROM_TAG: &str = "td:nth-child(1) small";
const SELECTOR_TEAM_NAME_FROM_NAME: &str = "td:nth-child(2) small";
const SELECTOR_TEAM_NAME_TO_TAG: &str = "td:nth-child(3) small";
const SELECTOR_TEAM_NAME_TO_NAME: &str = "td:nth-child(4) small";
const SELECTOR_TEAM_NAME_DATE: &str = "td:nth-child(5) small";

const SELECTOR_STEAM: &str = r#"a.btn.btn-xs.btn-default[href*="//steamcommunity.com/groups"]"#;

pub struct TeamParser {
    selector_name: Selector,
    selector_tag: Selector,
    selector_image: Selector,

    selector_team_format: Selector,
    selector_team_division: Selector,
    selector_team_timezone: Selector,
    selector_team_description: Selector,
    selector_team_titles: Selector,
    selector_steam_group: Selector,

    selector_team_member_row: Selector,
    selector_team_member_link: Selector,
    selector_team_member_role: Selector,
    selector_team_member_since: Selector,

    selector_team_records: Selector,
    selector_team_record_season: Selector,
    selector_team_record_division: Selector,
    selector_team_record_result: Selector,

    selector_team_name_item: Selector,
    selector_team_name_from_tag: Selector,
    selector_team_name_from_name: Selector,
    selector_team_name_to_tag: Selector,
    selector_team_name_to_name: Selector,
    selector_team_name_date: Selector,
}

impl Default for TeamParser {
    fn default() -> Self {
        Self::new()
    }
}

impl TeamParser {
    pub fn new() -> Self {
        TeamParser {
            selector_name: Selector::parse(SELECTOR_TEAM_NAME).unwrap(),
            selector_tag: Selector::parse(SELECTOR_TEAM_TAG).unwrap(),
            selector_image: Selector::parse(SELECTOR_TEAM_IMAGE).unwrap(),

            selector_team_format: Selector::parse(SELECTOR_TEAM_FORMAT).unwrap(),
            selector_team_division: Selector::parse(SELECTOR_TEAM_DIVISION).unwrap(),
            selector_team_timezone: Selector::parse(SELECTOR_TEAM_TIMEZONE).unwrap(),
            selector_team_description: Selector::parse(SELECTOR_TEAM_DESCRIPTION).unwrap(),
            selector_team_titles: Selector::parse(SELECTOR_TEAM_TITLES).unwrap(),
            selector_steam_group: Selector::parse(SELECTOR_STEAM).unwrap(),

            selector_team_member_row: Selector::parse(SELECTOR_TEAM_MEMBER_ROW).unwrap(),
            selector_team_member_link: Selector::parse(SELECTOR_TEAM_MEMBER_LINK).unwrap(),
            selector_team_member_role: Selector::parse(SELECTOR_TEAM_MEMBER_ROLE).unwrap(),
            selector_team_member_since: Selector::parse(SELECTOR_TEAM_MEMBER_SINCE).unwrap(),

            selector_team_records: Selector::parse(SELECTOR_TEAM_RECORDS).unwrap(),
            selector_team_record_season: Selector::parse(SELECTOR_TEAM_RECORD_SEASON).unwrap(),
            selector_team_record_division: Selector::parse(SELECTOR_TEAM_RECORD_DIVISION).unwrap(),
            selector_team_record_result: Selector::parse(SELECTOR_TEAM_RECORD_RESULT).unwrap(),

            selector_team_name_item: Selector::parse(SELECTOR_TEAM_NAME_CHANGE).unwrap(),
            selector_team_name_from_tag: Selector::parse(SELECTOR_TEAM_NAME_FROM_TAG).unwrap(),
            selector_team_name_from_name: Selector::parse(SELECTOR_TEAM_NAME_FROM_NAME).unwrap(),
            selector_team_name_to_tag: Selector::parse(SELECTOR_TEAM_NAME_TO_TAG).unwrap(),
            selector_team_name_to_name: Selector::parse(SELECTOR_TEAM_NAME_TO_NAME).unwrap(),
            selector_team_name_date: Selector::parse(SELECTOR_TEAM_NAME_DATE).unwrap(),
        }
    }
}

impl Parser for TeamParser {
    type Output = Team;

    fn parse(&self, document: &str) -> Result<Self::Output> {
        let document = Html::parse_document(document);
        let root = document.root_element();
        let mut name = select_text(root, &self.selector_name)
            .unwrap_or_default()
            .to_string();

        let tag = select_text(root, &self.selector_tag)
            .unwrap_or_default()
            .to_string();

        match (tag.as_str(), name.as_str()) {
            ("", "") => return Err(ScrapeError::NotFound),
            (_, "") => name = tag.clone(),
            _ => {}
        };

        let image =
            document
                .select(&self.selector_image)
                .next()
                .ok_or(ParseError::ElementNotFound {
                    selector: SELECTOR_TEAM_IMAGE,
                    role: "team image",
                })?;
        let image = image
            .attr("data-cfsrc")
            .or_else(|| image.attr("src"))
            .unwrap_or_default()
            .to_string();

        let format = select_text(root, &self.selector_team_format)
            .ok_or(ParseError::ElementNotFound {
                selector: SELECTOR_TEAM_FORMAT,
                role: "team format",
            })?
            .parse::<GameMode>()
            .map_err(|e| ParseError::InvalidText {
                text: e.text,
                role: "team game mode",
            })?;

        let steam_group = root
            .select(&self.selector_steam_group)
            .next()
            .and_then(|link| {
                link.attr("href")
                    .map(|group| group.replace("http://http", "http"))
            });

        let division = select_text(root, &self.selector_team_division)
            .ok_or(ParseError::ElementNotFound {
                selector: SELECTOR_TEAM_DIVISION,
                role: "team division",
            })?
            .to_string();

        let timezone = select_text(root, &self.selector_team_timezone).map(String::from);

        let description = select_text(root, &self.selector_team_description)
            .unwrap_or_default()
            .replace('\n', " ");

        let titles = document
            .select(&self.selector_team_titles)
            .next()
            .map(|el| {
                el.text()
                    .map(str::trim)
                    .filter(|s| !s.is_empty())
                    .map(String::from)
                    .collect()
            })
            .unwrap_or_default();

        let results = document
            .select(&self.selector_team_records)
            .map(|record| {
                let season = select_text(record, &self.selector_team_record_season).ok_or(
                    ParseError::ElementNotFound {
                        selector: SELECTOR_TEAM_RECORD_SEASON,
                        role: "team record season",
                    },
                )?;
                let division = select_text(record, &self.selector_team_record_division)
                    .ok_or(ParseError::ElementNotFound {
                        selector: SELECTOR_TEAM_RECORD_DIVISION,
                        role: "team record division",
                    })?
                    .to_string();
                let result = select_text(record, &self.selector_team_record_result).ok_or(
                    ParseError::ElementNotFound {
                        selector: SELECTOR_TEAM_RECORD_RESULT,
                        role: "team record result",
                    },
                )?;

                let (wins, losses) =
                    result
                        .split_once('-')
                        .ok_or_else(|| ParseError::InvalidText {
                            text: result.to_string(),
                            role: "team record result",
                        })?;

                Ok(Record {
                    season: season.parse().map_err(|_| ParseError::InvalidText {
                        text: season.to_string(),
                        role: "team record season",
                    })?,
                    division,
                    wins: wins.parse().map_err(|_| ParseError::InvalidText {
                        text: wins.to_string(),
                        role: "team record wins",
                    })?,
                    losses: losses.parse().map_err(|_| ParseError::InvalidText {
                        text: losses.to_string(),
                        role: "team record losses",
                    })?,
                })
            })
            .collect::<Result<Vec<_>>>()?;

        let members = document
            .select(&self.selector_team_member_row)
            .map(|row| {
                let link = row.select(&self.selector_team_member_link).next().ok_or(
                    ParseError::ElementNotFound {
                        selector: SELECTOR_TEAM_MEMBER_LINK,
                        role: "team member link",
                    },
                )?;
                let name = link
                    .first_text()
                    .ok_or(ParseError::EmptyText {
                        selector: SELECTOR_TEAM_MEMBER_LINK,
                        role: "team member link",
                    })?
                    .to_string();
                let link = link.attr("href").unwrap_or_default();

                let role = select_text(row, &self.selector_team_member_role)
                    .ok_or(ParseError::ElementNotFound {
                        selector: SELECTOR_TEAM_MEMBER_ROLE,
                        role: "team member role",
                    })?
                    .split('\n')
                    .next()
                    .unwrap();
                let since = select_text(row, &self.selector_team_member_since).ok_or(
                    ParseError::ElementNotFound {
                        selector: SELECTOR_TEAM_MEMBER_SINCE,
                        role: "team member since",
                    },
                )?;
                let role = role.trim().to_string();
                let since = since.trim();
                let since = if since.starts_with('(') {
                    let part = since
                        .split_once('-')
                        .unwrap_or_default()
                        .0
                        .trim()
                        .trim_start_matches('(');
                    let date = Date::parse(part, MEMBER_DATE_ALT_FORMAT).map_err(|_| {
                        ParseError::InvalidDate {
                            role: "member join date (alternate format)",
                            date: since.to_string(),
                        }
                    })?;
                    PrimitiveDateTime::new(date, Time::MIDNIGHT).assume_offset(UtcOffset::UTC)
                } else {
                    PrimitiveDateTime::parse(since, MEMBER_DATE_FORMAT)
                        .map_err(|_| ParseError::InvalidDate {
                            role: "member join date",
                            date: since.to_string(),
                        })?
                        .assume_offset(UtcOffset::from_hms(-5, 0, 0).unwrap())
                };

                Ok(Membership {
                    name,
                    steam_id: steam_id_from_link(link)?,
                    role,
                    since,
                })
            })
            .collect::<Result<Vec<_>>>()?;

        let name_changes = document
            .select(&self.selector_team_name_item)
            .map(|row| {
                let from_tag =
                    select_text(row, &self.selector_team_name_from_tag).unwrap_or_default();
                let from_name = select_text(row, &self.selector_team_name_from_name).ok_or(
                    ParseError::ElementNotFound {
                        selector: SELECTOR_TEAM_NAME_FROM_NAME,
                        role: "team name change from name",
                    },
                )?;
                let to_tag = select_text(row, &self.selector_team_name_to_tag).unwrap_or_default();
                let to_name = select_text(row, &self.selector_team_name_to_name).ok_or(
                    ParseError::ElementNotFound {
                        selector: SELECTOR_TEAM_NAME_TO_NAME,
                        role: "team name change from name",
                    },
                )?;
                let date = select_text(row, &self.selector_team_name_date).ok_or(
                    ParseError::ElementNotFound {
                        selector: SELECTOR_TEAM_NAME_DATE,
                        role: "team name change date",
                    },
                )?;
                let date = Date::parse(date, DATE_FORMAT).map_err(|_| ParseError::InvalidDate {
                    date: date.to_string(),
                    role: "team name change date",
                })?;
                Ok(NameChange {
                    from_tag: from_tag.to_string(),
                    from: from_name.to_string(),
                    to_tag: to_tag.to_string(),
                    to: to_name.to_string(),
                    date,
                })
            })
            .collect::<Result<_>>()?;

        Ok(Team {
            name,
            description,
            division,
            timezone,
            format,
            steam_group,
            image,
            tag,
            titles,
            results,
            members,
            name_changes,
        })
    }
}
