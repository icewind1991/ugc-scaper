use super::Parser;
use crate::data::{
    CurrentSeasonMap, CurrentSeasonMapList, MapHistory, PreviousSeasonMap, PreviousSeasonMapList,
};
use crate::parser::{select_text, ElementExt};
use crate::{ParseError, Result};
use scraper::{Html, Selector};
use time::{Date, Month};

const SELECTOR_CURRENT_ROW: &str = "table.table.table-condensed.table-responsive tbody tr";
const SELECTOR_CURRENT_SEASON: &str = "div.row > div > div.white-row-small > h5:nth-child(2), div.row-fluid > div > div.white-row-small > h4:first-child+h5";
const SELECTOR_CURRENT_WEEK: &str = "td:nth-child(1)";
const SELECTOR_CURRENT_MAP: &str = "td:nth-child(2)";
const SELECTOR_CURRENT_DATE: &str = "td:nth-child(4) small";
const SELECTOR_CURRENT_DATE_ALT: &str = "td:nth-child(5) small";

const SELECTOR_PREVIOUS: &str =
    "table.table.table-condensed.table-bordered tbody tr:not(:first-child)";
const SELECTOR_PREVIOUS_WEEK: &str = "td:nth-child(1)";
const SELECTOR_PREVIOUS_DATE: &str = "td:nth-child(2)";
const SELECTOR_PREVIOUS_MAP: &str = "td:nth-child(3)";

pub struct MapHistoryParser {
    selector_current_row: Selector,
    selector_current_season: Selector,
    selector_current_week: Selector,
    selector_current_map: Selector,
    selector_current_date: Selector,
    selector_current_date_alt: Selector,

    selector_previous: Selector,
    selector_previous_week: Selector,
    selector_previous_date: Selector,
    selector_previous_map: Selector,
}

impl Default for MapHistoryParser {
    fn default() -> Self {
        Self::new()
    }
}

impl MapHistoryParser {
    pub fn new() -> Self {
        MapHistoryParser {
            selector_current_row: Selector::parse(SELECTOR_CURRENT_ROW).unwrap(),
            selector_current_season: Selector::parse(SELECTOR_CURRENT_SEASON).unwrap(),
            selector_current_week: Selector::parse(SELECTOR_CURRENT_WEEK).unwrap(),
            selector_current_map: Selector::parse(SELECTOR_CURRENT_MAP).unwrap(),
            selector_current_date: Selector::parse(SELECTOR_CURRENT_DATE).unwrap(),
            selector_current_date_alt: Selector::parse(SELECTOR_CURRENT_DATE_ALT).unwrap(),

            selector_previous: Selector::parse(SELECTOR_PREVIOUS).unwrap(),
            selector_previous_week: Selector::parse(SELECTOR_PREVIOUS_WEEK).unwrap(),
            selector_previous_date: Selector::parse(SELECTOR_PREVIOUS_DATE).unwrap(),
            selector_previous_map: Selector::parse(SELECTOR_PREVIOUS_MAP).unwrap(),
        }
    }
}

impl Parser for MapHistoryParser {
    type Output = MapHistory;

    fn parse(&self, document: &str) -> Result<Self::Output> {
        let document = Html::parse_document(document);

        let season = select_text(document.root_element(), &self.selector_current_season)
            .ok_or(ParseError::ElementNotFound {
                selector: SELECTOR_CURRENT_SEASON,
                role: "current season number",
            })?
            .trim_start_matches("Season")
            .trim();
        let season: u8 = season.parse().map_err(|_| ParseError::InvalidText {
            role: "current season number",
            text: season.to_string(),
        })?;

        let current_weeks = document
            .select(&self.selector_current_row)
            .map(|row| {
                let week = select_text(row, &self.selector_current_week).ok_or(
                    ParseError::ElementNotFound {
                        selector: SELECTOR_CURRENT_WEEK,
                        role: "current season week number",
                    },
                )?;
                let week: u8 = week.parse().map_err(|_| ParseError::InvalidText {
                    role: "current season week number",
                    text: season.to_string(),
                })?;
                let map = select_text(row, &self.selector_current_map)
                    .ok_or(ParseError::ElementNotFound {
                        selector: SELECTOR_CURRENT_MAP,
                        role: "current season map",
                    })?
                    .to_string();
                let mut date = select_text(row, &self.selector_current_date)
                    .ok_or(ParseError::ElementNotFound {
                        selector: SELECTOR_CURRENT_MAP,
                        role: "current season map",
                    })?
                    .to_string();
                let mut alt_date =
                    select_text(row, &self.selector_current_date_alt).map(String::from);
                if let Some(global_date) = alt_date.take() {
                    alt_date = Some(date);
                    date = global_date;
                }

                Ok(CurrentSeasonMap {
                    week,
                    map,
                    date,
                    na_date: alt_date,
                })
            })
            .collect::<Result<_>>()?;

        let mut previous = Vec::with_capacity(8);
        let mut prev_season = None;
        for row in document.select(&self.selector_previous) {
            if row.attr("class") == Some("top-bar") {
                if let Some(season) = prev_season.take() {
                    previous.push(season);
                }
                let season = row
                    .first_text()
                    .unwrap_or_default()
                    .trim_start_matches("Season ");
                dbg!(season);
                let season = season.parse().map_err(|_| ParseError::InvalidText {
                    role: "previous season number",
                    text: season.to_string(),
                })?;
                prev_season = Some(PreviousSeasonMapList {
                    season,
                    maps: Vec::with_capacity(8),
                });
            } else if row
                .children()
                .filter(|child| child.value().is_element())
                .count()
                == 3
            {
                if let Some(season) = prev_season.as_mut() {
                    let week = select_text(row, &self.selector_previous_week).ok_or(
                        ParseError::ElementNotFound {
                            selector: SELECTOR_PREVIOUS_WEEK,
                            role: "previous season week number",
                        },
                    )?;
                    if week != "Week" {
                        let week = week.parse().map_err(|_| ParseError::InvalidText {
                            role: "previous season week number",
                            text: week.to_string(),
                        })?;
                        let date = select_text(row, &self.selector_previous_date).ok_or(
                            ParseError::ElementNotFound {
                                selector: SELECTOR_PREVIOUS_DATE,
                                role: "previous season week number",
                            },
                        )?;
                        let date = parse_date(date)?;
                        let map = select_text(row, &self.selector_previous_map)
                            .ok_or(ParseError::ElementNotFound {
                                selector: SELECTOR_PREVIOUS_MAP,
                                role: "previous season map",
                            })?
                            .to_string();
                        season.maps.push(PreviousSeasonMap { week, date, map })
                    }
                }
            }
        }
        if let Some(season) = prev_season {
            previous.push(season);
        }

        Ok(MapHistory {
            current: CurrentSeasonMapList {
                season,
                maps: current_weeks,
            },
            previous,
        })
    }
}

fn parse_date(date: &str) -> Result<Date> {
    let err = || ParseError::InvalidDate {
        date: date.to_string(),
        role: "previous season date",
    };
    let mut parts = date.split('/');
    let month: u8 = parts.next().ok_or_else(err)?.parse().map_err(|_| err())?;
    let month = Month::try_from(month).map_err(|_| err())?;
    let date: u8 = parts.next().ok_or_else(err)?.parse().map_err(|_| err())?;
    let year: i32 = parts.next().ok_or_else(err)?.parse().map_err(|_| err())?;
    let year = 2000 + year;
    Ok(Date::from_calendar_date(year, month, date).map_err(|_| err())?)
}
