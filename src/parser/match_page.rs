use super::Parser;
use crate::data::{MatchInfo, TeamRef};
use crate::parser::{select_last_text, select_text, team_id_from_link, ElementExt};
use crate::{ParseError, Result};
use scraper::{Html, Selector};

const SELECTOR_MATCH_COMMENT_AUTHOR: &str = ".row-fluid .col-md-12 span.text-success";
const SELECTOR_MATCH_COMMENT: &str = ".row-fluid .col-md-12 > .white-row-light-small > p";
const SELECTOR_MATCH_TEAM_LINK: &str = "a[href^=\"team_page\"]:not(.btn-large)";
const SELECTOR_MATCH_RESULT_TEAM: &str =
    ".table.table-condensed.table-bordered tr:nth-child(2) td:nth-child(1)";
const SELECTOR_MATCH_RESULT_SCORE: &str =
    ".table.table-condensed.table-bordered tr:nth-child(2) td:nth-child(2)";

pub struct MatchPageParser {
    selector_author: Selector,
    selector_comment: Selector,
    selector_team_link: Selector,
    selector_result_team: Selector,
    selector_result_score: Selector,
}

impl Default for MatchPageParser {
    fn default() -> Self {
        Self::new()
    }
}

impl MatchPageParser {
    pub fn new() -> Self {
        MatchPageParser {
            selector_author: Selector::parse(SELECTOR_MATCH_COMMENT_AUTHOR).unwrap(),
            selector_comment: Selector::parse(SELECTOR_MATCH_COMMENT).unwrap(),
            selector_team_link: Selector::parse(SELECTOR_MATCH_TEAM_LINK).unwrap(),
            selector_result_team: Selector::parse(SELECTOR_MATCH_RESULT_TEAM).unwrap(),
            selector_result_score: Selector::parse(SELECTOR_MATCH_RESULT_SCORE).unwrap(),
        }
    }
}

impl Parser for MatchPageParser {
    type Output = MatchInfo;

    fn parse(&self, document: &str) -> Result<Self::Output> {
        let document = Html::parse_document(document);

        let author = select_text(document.root_element(), &self.selector_author);
        let comment = select_last_text(document.root_element(), &self.selector_comment);

        let mut team_links = document.select(&self.selector_team_link);
        let team_link_home = team_links.next().ok_or(ParseError::ElementNotFound {
            selector: SELECTOR_MATCH_TEAM_LINK,
            role: "home team link",
        })?;
        let team_link_away = team_links.next().ok_or(ParseError::ElementNotFound {
            selector: SELECTOR_MATCH_TEAM_LINK,
            role: "away team link",
        })?;
        let home_team_id = team_id_from_link(team_link_home.attr("href").unwrap_or_default())?;
        let away_team_id = team_id_from_link(team_link_away.attr("href").unwrap_or_default())?;

        let mut team_names = document.select(&self.selector_result_team);
        let team_name_home = team_names
            .next()
            .ok_or(ParseError::ElementNotFound {
                selector: SELECTOR_MATCH_RESULT_TEAM,
                role: "home team link",
            })?
            .first_text()
            .ok_or(ParseError::EmptyText {
                role: "home team name",
                selector: SELECTOR_MATCH_RESULT_TEAM,
            })?
            .to_string();
        let team_name_away = team_names
            .next()
            .ok_or(ParseError::ElementNotFound {
                selector: SELECTOR_MATCH_RESULT_TEAM,
                role: "away team link",
            })?
            .first_text()
            .ok_or(ParseError::EmptyText {
                role: "away team name",
                selector: SELECTOR_MATCH_RESULT_TEAM,
            })?
            .to_string();

        let mut team_scores = document.select(&self.selector_result_score);

        let team_score_home = team_scores
            .next()
            .ok_or(ParseError::ElementNotFound {
                selector: SELECTOR_MATCH_RESULT_SCORE,
                role: "home team score",
            })?
            .first_text()
            .ok_or(ParseError::EmptyText {
                role: "home team score",
                selector: SELECTOR_MATCH_RESULT_SCORE,
            })?
            .parse()
            .map_err(|_| ParseError::InvalidText {
                role: "away team score",
                text: "dont have this".to_string(),
            })?;
        let team_score_away = team_scores
            .next()
            .ok_or(ParseError::ElementNotFound {
                selector: SELECTOR_MATCH_RESULT_SCORE,
                role: "away team link",
            })?
            .first_text()
            .ok_or(ParseError::EmptyText {
                role: "away team name",
                selector: SELECTOR_MATCH_RESULT_SCORE,
            })?
            .parse()
            .map_err(|_| ParseError::InvalidText {
                role: "home team score",
                text: "dont have this".to_string(),
            })?;

        Ok(MatchInfo {
            comment_author: author.map(String::from),
            comment: comment.map(String::from),
            score_away: team_score_away,
            score_home: team_score_home,
            team_home: TeamRef {
                name: team_name_home.to_string(),
                id: home_team_id,
            },
            team_away: TeamRef {
                name: team_name_away.to_string(),
                id: away_team_id,
            },
        })
    }
}
