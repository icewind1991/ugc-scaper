use crate::Result;

mod player;

pub use player::*;

pub trait Parser {
    type Output;
    fn parse(&self, document: &str) -> Result<Self::Output>;
}