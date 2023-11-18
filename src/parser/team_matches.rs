use super::Parser;
use crate::data::{MatchResult, TeamRef, TeamSeason, TeamSeasonMatch};
use crate::parser::{select_text, team_id_from_link, ElementExt};
use crate::{ParseError, Result};
use scraper::{Html, Selector};

const SELECTOR_SEASON_TITLE: &str =
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

pub struct TeamMatchesParser {
    selector_title: Selector,
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
        }
    }
}

impl Parser for TeamMatchesParser {
    type Output = Vec<TeamSeason>;

    fn parse(&self, document: &str) -> Result<Self::Output> {
        let document = Html::parse_document(document);

        document
            .select(&self.selector_title)
            .zip(document.select(&self.selector_matches))
            .map(|(title, matches)| {
                let title = title.first_text().ok_or(ParseError::EmptyText {
                    selector: SELECTOR_SEASON_TITLE,
                    role: "season title",
                })?;
                let season: u32 = title.trim_start_matches("Season ").parse().map_err(|_| {
                    ParseError::InvalidText {
                        text: title.to_string(),
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
                                .split_once(" -\n")
                                .ok_or_else(|| ParseError::InvalidText {
                                    text: scores.to_string(),
                                    role: "match scores",
                                })?;
                        let score = score.parse().map_err(|_| ParseError::InvalidText {
                            text: scores.to_string(),
                            role: "match scores",
                        });
                        let score_opponent =
                            score_opponent.parse().map_err(|_| ParseError::InvalidText {
                                text: scores.to_string(),
                                role: "match scores",
                            });

                        let opponent = opponent_link
                            .map(|link| {
                                let name = link.first_text().ok_or(ParseError::EmptyText {
                                    selector: SELECTOR_SEASON_OPPONENT,
                                    role: "match opponent",
                                })?;
                                let id = team_id_from_link(link.attr("href").unwrap_or_default())?;
                                Result::<_, ParseError>::Ok(TeamRef {
                                    name: name.to_string(),
                                    id,
                                })
                            })
                            .transpose()?;

                        let result = match (opponent, points, points_opponent) {
                            (Some(opponent), Some(point), Some(points_opponent)) => {
                                MatchResult::Played {
                                    opponent,
                                    score: score?,
                                    score_opponent: score_opponent?,
                                    match_points: point,
                                    match_points_opponent: points_opponent,
                                }
                            }
                            (Some(opponent), None, None) => MatchResult::Pending {
                                opponent,
                                score: score?,
                                score_opponent: score_opponent?,
                            },
                            _ => MatchResult::ByeWeek,
                        };
                        Ok(TeamSeasonMatch {
                            week,
                            date: date.to_string(),
                            side: side.to_string(),
                            map: map.to_string(),
                            division: division.to_string(),
                            result,
                        })
                    })
                    .collect::<Result<_>>()?;

                Ok(TeamSeason { season, matches })
            })
            .collect::<Result<Vec<_>>>()
    }
}
