use crate::{ParseError, Result};
use scraper::{ElementRef, Selector};
use steamid_ng::SteamID;
use time::format_description::FormatItem;
use time::macros::format_description;

mod player;
mod player_details;
mod seasons;
mod team;
mod team_lookup;
mod team_matches;
mod team_roster_history;

pub use player::*;
pub use player_details::*;
pub use seasons::*;
pub use team::*;
pub use team_lookup::*;
pub use team_matches::*;
pub use team_roster_history::*;

pub trait Parser {
    type Output;
    fn parse(&self, document: &str) -> Result<Self::Output>;
}

trait ElementExt<'a> {
    fn first_text(&'a self) -> Option<&'a str>;
    fn nth_text(&'a self, n: usize) -> Option<&'a str>;
}

impl<'a> ElementExt<'a> for ElementRef<'a> {
    fn first_text(&self) -> Option<&'a str> {
        self.text().map(str::trim).find(|s| !s.is_empty())
    }
    fn nth_text(&self, n: usize) -> Option<&'a str> {
        self.text()
            .filter(|s| !s.trim().is_empty())
            .nth(n - 1)
            .map(str::trim)
    }
}

fn select_text<'a>(el: ElementRef<'a>, selector: &Selector) -> Option<&'a str> {
    el.select(selector)
        .next()
        .and_then(|item| item.text().find(|s| !s.trim().is_empty()))
        .map(str::trim)
}

fn select_last_text<'a>(el: ElementRef<'a>, selector: &Selector) -> Option<&'a str> {
    el.select(selector)
        .next()
        .and_then(|item| item.text().last())
        .map(str::trim)
}

const DATE_FORMAT: &[FormatItem<'static>] =
    format_description!("[month padding:none]/[day padding:none]/[year]");
const MEMBER_DATE_FORMAT: &[FormatItem<'static>] = format_description!(
    "[month repr:short] [day padding:none], [year]\n/\n[hour padding:none]:[minute] [period]\n(ET)"
);
const ROSTER_HISTORY_DATE_FORMAT: &[FormatItem<'static>] =
    format_description!("[month repr:short] [day padding:none], [year]");

fn team_id_from_link(link: &str) -> Result<u32, ParseError> {
    link.rsplit_once('=')
        .and_then(|part| part.1.parse().ok())
        .ok_or_else(|| ParseError::InvalidLink {
            link: link.to_string(),
            role: "team id",
        })
}

fn steam_id_from_link(link: &str) -> Result<SteamID, ParseError> {
    link.rsplit_once('=')
        .and_then(|part| part.1.parse::<u64>().ok())
        .ok_or_else(|| ParseError::InvalidLink {
            link: link.to_string(),
            role: "user id",
        })
        .map(SteamID::from)
}
