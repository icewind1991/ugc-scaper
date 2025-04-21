use super::Parser;
use crate::data::{GameMode, MatchResult, TeamRef, TeamSeason, TeamSeasonMatch};
use crate::parser::{match_id_from_link, select_text, team_id_from_link, ElementExt};
use crate::{ParseError, Result};
use scraper::{Html, Selector};
use std::str::FromStr;
use ugc_scraper_types::{Side, TeamMatches};

const SELECTOR_SEASON_TITLE: &str = ".container table.table.table-condensed.table-striped thead h4";
const SELECTOR_SEASON_SEASON: &str =
    ".container table.table.table-condensed.table-striped thead h4 b";
const SELECTOR_SEASON_MATCHES: &str =
    ".container table.table.table-condensed.table-striped tbody:nth-child(3n)";
const SELECTOR_SEASON_MATCH: &str = "tr:not(:last-child)";
const SELECTOR_SEASON_DIVISION: &str = "td:nth-child(1) small";
const SELECTOR_SEASON_WEEK: &str = "td:nth-child(2) small";
const SELECTOR_SEASON_DATE: &str = "td:nth-child(3) small";
const SELECTOR_SEASON_SIDE: &str = "td:nth-child(4) small";
const SELECTOR_SEASON_OPPONENT: &str = "td:nth-child(6) a";
const SELECTOR_SEASON_MAP: &str = "td:nth-child(7)";
const SELECTOR_SEASON_SCORES: &str = "td:nth-child(8)";
const SELECTOR_SEASON_POINTS: &str = "td:nth-child(9) small";
const SELECTOR_SEASON_POINTS_OPPONENTS: &str = "td:nth-child(10) small";
const SELECTOR_SEASON_MATCH_PAGE: &str = "td a[href^=\"matchpage\"]";

const SELECTOR_TEAM_NAME: &str = r#"div.col-md-9 > h2 > b"#;
const SELECTOR_TEAM_LINK: &str = r#"h2 > span.pull-right > a[href^="team_page.cfm"]"#;

pub struct TeamMatchesParser {
    selector_title: Selector,
    selector_season: Selector,
    selector_matches: Selector,
    selector_match: Selector,
    selector_division: Selector,
    selector_week: Selector,
    selector_date: Selector,
    selector_side: Selector,
    selector_opponent: Selector,
    selector_map: Selector,
    selector_scores: Selector,
    selector_points: Selector,
    selector_points_opponent: Selector,
    selector_match_page: Selector,

    selector_team_name: Selector,
    selector_team_link: Selector,
}

impl Default for TeamMatchesParser {
    fn default() -> Self {
        Self::new()
    }
}

impl TeamMatchesParser {
    pub fn new() -> Self {
        TeamMatchesParser {
            selector_title: Selector::parse(SELECTOR_SEASON_TITLE).unwrap(),
            selector_season: Selector::parse(SELECTOR_SEASON_SEASON).unwrap(),
            selector_matches: Selector::parse(SELECTOR_SEASON_MATCHES).unwrap(),
            selector_match: Selector::parse(SELECTOR_SEASON_MATCH).unwrap(),
            selector_division: Selector::parse(SELECTOR_SEASON_DIVISION).unwrap(),
            selector_week: Selector::parse(SELECTOR_SEASON_WEEK).unwrap(),
            selector_date: Selector::parse(SELECTOR_SEASON_DATE).unwrap(),
            selector_side: Selector::parse(SELECTOR_SEASON_SIDE).unwrap(),
            selector_opponent: Selector::parse(SELECTOR_SEASON_OPPONENT).unwrap(),
            selector_map: Selector::parse(SELECTOR_SEASON_MAP).unwrap(),
            selector_scores: Selector::parse(SELECTOR_SEASON_SCORES).unwrap(),
            selector_points: Selector::parse(SELECTOR_SEASON_POINTS).unwrap(),
            selector_points_opponent: Selector::parse(SELECTOR_SEASON_POINTS_OPPONENTS).unwrap(),
            selector_match_page: Selector::parse(SELECTOR_SEASON_MATCH_PAGE).unwrap(),

            selector_team_name: Selector::parse(SELECTOR_TEAM_NAME).unwrap(),
            selector_team_link: Selector::parse(SELECTOR_TEAM_LINK).unwrap(),
        }
    }
}

impl Parser for TeamMatchesParser {
    type Output = TeamMatches;

    fn parse(&self, document: &str) -> Result<Self::Output> {
        let document = Html::parse_document(document);

        let seasons = document
            .select(&self.selector_title)
            .zip(document.select(&self.selector_season))
            .zip(document.select(&self.selector_matches))
            .map(|((title, season), matches)| {
                let format = title.first_text().ok_or(ParseError::EmptyText {
                    selector: SELECTOR_SEASON_TITLE,
                    role: "season title",
                })?;

                let format = format
                    .split(' ')
                    .find_map(|part| GameMode::from_str(part).ok())
                    .ok_or(ParseError::InvalidText {
                        text: format.into(),
                        role: "season format",
                    })?;

                let season = season.first_text().ok_or(ParseError::EmptyText {
                    selector: SELECTOR_SEASON_SEASON,
                    role: "season title",
                })?;
                let season: u32 = season.trim_start_matches("Season ").parse().map_err(|_| {
                    ParseError::InvalidText {
                        text: season.to_string(),
                        role: "season title",
                    }
                })?;

                let matches = matches
                    .select(&self.selector_match)
                    .map(|game| {
                        let division = select_text(game, &self.selector_division).ok_or(
                            ParseError::ElementNotFound {
                                selector: SELECTOR_SEASON_DIVISION,
                                role: "match division",
                            },
                        )?;
                        let week = select_text(game, &self.selector_week).ok_or(
                            ParseError::ElementNotFound {
                                selector: SELECTOR_SEASON_WEEK,
                                role: "match week",
                            },
                        )?;
                        let week = week.parse().map_err(|_| ParseError::InvalidText {
                            text: week.to_string(),
                            role: "match week",
                        })?;
                        let date = select_text(game, &self.selector_date).ok_or(
                            ParseError::ElementNotFound {
                                selector: SELECTOR_SEASON_DATE,
                                role: "match date",
                            },
                        )?;
                        let side = select_text(game, &self.selector_side).ok_or(
                            ParseError::ElementNotFound {
                                selector: SELECTOR_SEASON_SIDE,
                                role: "match side",
                            },
                        )?;
                        let opponent_link = game.select(&self.selector_opponent).next();
                        let map = select_text(game, &self.selector_map).ok_or(
                            ParseError::ElementNotFound {
                                selector: SELECTOR_SEASON_MAP,
                                role: "match map",
                            },
                        )?;
                        let scores = select_text(game, &self.selector_scores)
                            .ok_or(ParseError::ElementNotFound {
                                selector: SELECTOR_SEASON_SCORES,
                                role: "match scores",
                            })?
                            .trim_start_matches('(')
                            .trim_end_matches(')');
                        let points = select_text(game, &self.selector_points);
                        let points_opponent = select_text(game, &self.selector_points_opponent);
                        let id = game
                            .select(&self.selector_match_page)
                            .next()
                            .and_then(|link| {
                                match_id_from_link(link.attr("href").unwrap_or_default()).ok()
                            });

                        let points = points
                            .map(|points| {
                                points.parse().map_err(|_| ParseError::InvalidText {
                                    text: points.to_string(),
                                    role: "match points",
                                })
                            })
                            .transpose()?;

                        let points_opponent = points_opponent
                            .map(|points| {
                                points.parse().map_err(|_| ParseError::InvalidText {
                                    text: points.to_string(),
                                    role: "match points opponent",
                                })
                            })
                            .transpose()?;

                        let (score, score_opponent) =
                            scores
                                .split_once('-')
                                .ok_or_else(|| ParseError::InvalidText {
                                    text: scores.to_string(),
                                    role: "match scores",
                                })?;
                        let score: u8 = score.trim().parse().unwrap_or_default();
                        let score_opponent: u8 = score_opponent.trim().parse().unwrap_or_default();

                        let opponent = opponent_link
                            .map(|link| {
                                let name = link.first_text().unwrap_or_default();
                                let id = team_id_from_link(link.attr("href").unwrap_or_default())?;
                                Result::<_, ParseError>::Ok(TeamRef {
                                    name: name.to_string(),
                                    id,
                                })
                            })
                            .transpose()?;

                        let result = match (opponent, points, points_opponent, id) {
                            (Some(opponent), Some(point), Some(points_opponent), Some(_)) => {
                                MatchResult::Played {
                                    id: id.ok_or(ParseError::ElementNotFound {
                                        selector: SELECTOR_SEASON_MATCH_PAGE,
                                        role: "match page link",
                                    })?,
                                    opponent,
                                    score,
                                    score_opponent,
                                    match_points: point,
                                    match_points_opponent: points_opponent,
                                }
                            }
                            (Some(opponent), None, None, Some(_)) => MatchResult::Pending {
                                id: id.ok_or(ParseError::ElementNotFound {
                                    selector: SELECTOR_SEASON_MATCH_PAGE,
                                    role: "match page link",
                                })?,
                                opponent,
                                score,
                                score_opponent,
                            },
                            (Some(opponent), _, _, None) => MatchResult::Unknown {
                                opponent,
                                score,
                                score_opponent,
                            },
                            _ => MatchResult::ByeWeek,
                        };
                        Ok(TeamSeasonMatch {
                            week,
                            date: date.to_string(),
                            side: side.parse::<Side>().map_err(|error| {
                                ParseError::InvalidText {
                                    text: error.text,
                                    role: "match side",
                                }
                            })?,
                            map: map.to_string(),
                            division: division.to_string(),
                            result,
                        })
                    })
                    .collect::<Result<_>>()?;

                Ok(TeamSeason {
                    season,
                    matches,
                    format,
                })
            })
            .collect::<Result<Vec<_>>>()?;

        let team_id = document
            .select(&self.selector_team_link)
            .next()
            .and_then(|link| team_id_from_link(link.attr("href").unwrap_or_default()).ok())
            .ok_or(ParseError::ElementNotFound {
                selector: SELECTOR_TEAM_LINK,
                role: "match team link",
            })?;

        let team_name =
            select_text(document.root_element(), &self.selector_team_name).unwrap_or_default();
        let team = TeamRef {
            id: team_id,
            name: team_name.into(),
        };

        Ok(TeamMatches { team, seasons })
    }
}
