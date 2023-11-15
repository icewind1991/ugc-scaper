use crate::Result;
use scraper::ElementRef;

mod player;

pub use player::*;

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
        self.text().filter(|s| !s.trim().is_empty()).next()
    }
    fn nth_text(&self, n: usize) -> Option<&'a str> {
        self.text()
            .filter(|s| !s.trim().is_empty())
            .skip(n - 1)
            .next()
            .map(|s| s.trim())
    }
}
