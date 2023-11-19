use super::Parser;
use crate::data::{TeamRef, Transaction};
use crate::parser::{
    select_last_text, select_text, steam_id_from_link, team_id_from_link, ElementExt,
};
use crate::{ParseError, Result};
use scraper::{Html, Selector};

const SELECTOR_TRANSACTION_ROW: &str = "table.table.table-condensed.table-striped tr";
const SELECTOR_TRANSACTION_PLAYER_LINK: &str = "a[href^=\"players_page\"][title^=\"Roster\"]";
const SELECTOR_TRANSACTION_ACTION: &str = "td:nth-child(4) span b";
const SELECTOR_TRANSACTION_TEAM_LINK: &str = "a[href^=\"team_page\"]";
const SELECTOR_TRANSACTION_TEAM_NAME: &str = "td:nth-child(5)";

pub struct TransactionParser {
    selector_row: Selector,
    selector_player: Selector,
    selector_action: Selector,
    selector_team_link: Selector,
    selector_team_name: Selector,
}

impl Default for TransactionParser {
    fn default() -> Self {
        Self::new()
    }
}

impl TransactionParser {
    pub fn new() -> Self {
        TransactionParser {
            selector_row: Selector::parse(SELECTOR_TRANSACTION_ROW).unwrap(),
            selector_player: Selector::parse(SELECTOR_TRANSACTION_PLAYER_LINK).unwrap(),
            selector_action: Selector::parse(SELECTOR_TRANSACTION_ACTION).unwrap(),
            selector_team_link: Selector::parse(SELECTOR_TRANSACTION_TEAM_LINK).unwrap(),
            selector_team_name: Selector::parse(SELECTOR_TRANSACTION_TEAM_NAME).unwrap(),
        }
    }
}

impl Parser for TransactionParser {
    type Output = Vec<Transaction>;

    fn parse(&self, document: &str) -> Result<Self::Output> {
        let document = Html::parse_document(document);

        document
            .select(&self.selector_row)
            .filter(|row| row.select(&self.selector_player).next().is_some())
            .map(|row| {
                let player_link = row.select(&self.selector_player).next().ok_or(
                    ParseError::ElementNotFound {
                        selector: SELECTOR_TRANSACTION_PLAYER_LINK,
                        role: "player link",
                    },
                )?;
                let name = player_link.first_text().ok_or(ParseError::EmptyText {
                    selector: SELECTOR_TRANSACTION_PLAYER_LINK,
                    role: "player name",
                })?;
                let steam_id = steam_id_from_link(player_link.attr("href").unwrap_or_default())?;

                let action = select_text(row, &self.selector_action)
                    .ok_or(ParseError::ElementNotFound {
                        selector: SELECTOR_TRANSACTION_ACTION,
                        role: "transaction action",
                    })?
                    .parse()?;

                let team_link = row.select(&self.selector_team_link).next().ok_or(
                    ParseError::ElementNotFound {
                        selector: SELECTOR_TRANSACTION_TEAM_LINK,
                        role: "team link",
                    },
                )?;
                let team_id = team_id_from_link(team_link.attr("href").unwrap_or_default())?;
                let team_name = select_last_text(row, &self.selector_team_name).ok_or(
                    ParseError::EmptyText {
                        selector: SELECTOR_TRANSACTION_TEAM_LINK,
                        role: "team link",
                    },
                )?;

                Ok(Transaction {
                    name: name.to_string(),
                    steam_id,
                    action,
                    team: TeamRef {
                        id: team_id,
                        name: team_name.to_string(),
                    },
                })
            })
            .collect()
    }
}
