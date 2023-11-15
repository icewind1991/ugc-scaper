use scraper::{Html, Selector};
use super::Parser;
use crate::{ParseError, Result};
use crate::data::Player;

const SELECTOR_PLAYER_NAME: &str = ".col-md-4 > h3 > b";


pub struct PlayerParser {
    selector_name: Selector,
}

impl PlayerParser {
    pub fn new() -> Self {
        PlayerParser {
            selector_name: Selector::parse(SELECTOR_PLAYER_NAME).unwrap(),
        }
    }
}

impl Parser for PlayerParser {
    type Output = Player;

    fn parse(&self, document: &str) -> Result<Self::Output> {
        let document = Html::parse_document(&document);

        let name = document.select(&self.selector_name).next().ok_or(ParseError::ElementNotFound {
            selector: SELECTOR_PLAYER_NAME,
            role: "player name",
        })?.text().next().unwrap_or_default().to_string();
        Ok(Player {
            name
        })
    }
}
