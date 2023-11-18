use super::Parser;
use crate::data::TeamRef;
use crate::parser::{team_id_from_link, ElementExt};
use crate::{ParseError, Result};
use scraper::{Html, Selector};

const SELECTOR_SELECT: &str = "select[name=\"clan_select\"]";
const SELECTOR_OPTION: &str = "option[value^=\"team_page\"]";

pub struct TeamLookupParser {
    selector_select: Selector,
    selector_option: Selector,
}

impl Default for TeamLookupParser {
    fn default() -> Self {
        Self::new()
    }
}

impl TeamLookupParser {
    pub fn new() -> Self {
        TeamLookupParser {
            selector_select: Selector::parse(SELECTOR_SELECT).unwrap(),
            selector_option: Selector::parse(SELECTOR_OPTION).unwrap(),
        }
    }
}

impl Parser for TeamLookupParser {
    type Output = Vec<TeamRef>;

    fn parse(&self, document: &str) -> Result<Self::Output> {
        let document = Html::parse_document(document);

        let select =
            document
                .select(&self.selector_select)
                .next()
                .ok_or(ParseError::ElementNotFound {
                    selector: SELECTOR_SELECT,
                    role: "team list",
                })?;
        select
            .select(&self.selector_option)
            .map(|option| {
                let link = option.attr("value").ok_or(ParseError::EmptyText {
                    role: "team link",
                    selector: SELECTOR_OPTION,
                })?;
                let text = option.first_text().ok_or(ParseError::EmptyText {
                    role: "team name",
                    selector: SELECTOR_OPTION,
                })?;
                let (_, name) = text.split_once("-").unwrap_or_default();

                let id = team_id_from_link(link)?;
                Ok(TeamRef {
                    id,
                    name: name.trim().to_string(),
                })
            })
            .collect()
    }
}
