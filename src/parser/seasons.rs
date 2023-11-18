use super::Parser;
use crate::data::{Season, Seasons};
use crate::parser::{select_text, ElementExt};
use crate::{ParseError, Result};
use scraper::{Html, Selector};

const SELECTOR_MENU: &str = ".sub-menu";
const SELECTOR_NAME: &str = ".mega-menu-sub-title";
const SELECTOR_SEASON_LINK: &str = "ul[id$=\"seasons\"] a[href^=\"rankings_\"]";

pub struct SeasonsParser {
    selector_menu: Selector,
    selector_name: Selector,
    selector_link: Selector,
}

impl Default for SeasonsParser {
    fn default() -> Self {
        Self::new()
    }
}

impl SeasonsParser {
    pub fn new() -> Self {
        SeasonsParser {
            selector_menu: Selector::parse(SELECTOR_MENU).unwrap(),
            selector_name: Selector::parse(SELECTOR_NAME).unwrap(),
            selector_link: Selector::parse(SELECTOR_SEASON_LINK).unwrap(),
        }
    }
}

impl Parser for SeasonsParser {
    type Output = Vec<Seasons>;

    fn parse(&self, document: &str) -> Result<Self::Output> {
        let document = Html::parse_document(document);

        document
            .select(&self.selector_menu)
            .filter(|item| item.select(&self.selector_link).next().is_some())
            .filter(|item| item.select(&self.selector_name).next().is_some())
            .map(|item| {
                let name = select_text(item, &self.selector_name)
                    .ok_or(ParseError::EmptyText {
                        role: "game mode name",
                        selector: SELECTOR_NAME,
                    })?
                    .trim_end_matches(" Menu");

                let seasons = item
                    .select(&self.selector_link)
                    .map(|link| {
                        let text = link
                            .first_text()
                            .ok_or(ParseError::EmptyText {
                                role: "season name",
                                selector: SELECTOR_SEASON_LINK,
                            })?
                            .trim_end_matches(" Final Standings")
                            .trim_end_matches(" Final Rank")
                            .trim_end_matches(" Final Ranks");
                        let link = link.attr("href").ok_or(ParseError::EmptyText {
                            role: "season link",
                            selector: SELECTOR_SEASON_LINK,
                        })?;
                        let id = link
                            .trim_start_matches("rankings_")
                            .trim_end_matches(".cfm");
                        Ok(Season {
                            name: text.to_string(),
                            id: id.to_string(),
                        })
                    })
                    .collect::<Result<_>>()?;

                Ok(Seasons {
                    mode: name.to_string(),
                    seasons,
                })
            })
            .collect::<Result<Vec<_>>>()
    }
}
