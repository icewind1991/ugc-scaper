use super::Parser;
use crate::data::{CurrentSeasonMap, CurrentSeasonMapList, MapHistory};
use crate::parser::select_text;
use crate::{ParseError, Result};
use scraper::{Html, Selector};

const SELECTOR_CURRENT_ROW: &str = "table.table.table-condensed.table-responsive tbody tr";
const SELECTOR_CURRENT_SEASON: &str = "div.row > div > div.white-row-small > h5:nth-child(2), div.row-fluid > div > div.white-row-small > h4:first-child+h5";
const SELECTOR_CURRENT_WEEK: &str = "td:nth-child(1)";
const SELECTOR_CURRENT_MAP: &str = "td:nth-child(2)";
const SELECTOR_CURRENT_DATE: &str = "td:nth-child(4) small";
const SELECTOR_CURRENT_DATE_ALT: &str = "td:nth-child(5) small";

const SELECTOR_PREVIOUS: &str = "table.table.table-condensed.table-bordered";
const SELECTOR_PREVIOUS_SEASON: &str = "tr.top-bar td h3.text-info";

pub struct MapHistoryParser {
    selector_current_row: Selector,
    selector_current_season: Selector,
    selector_current_week: Selector,
    selector_current_map: Selector,
    selector_current_date: Selector,
    selector_current_date_alt: Selector,
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

        Ok(MapHistory {
            current: CurrentSeasonMapList {
                season,
                maps: current_weeks,
            },
            previous: Vec::new(),
        })
    }
}
