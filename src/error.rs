use thiserror::Error;
use ugc_scraper_types::MallFormedTransaction;

#[derive(Debug, Error)]
pub enum ScrapeError {
    #[error("Failed to request data: {0:#}")]
    Request(#[from] reqwest::Error),
    #[error(transparent)]
    Parse(#[from] ParseError),
    #[error("Player or team doesn't exist")]
    NotFound,
}

#[derive(Debug, Error, Clone)]
pub enum ParseError {
    #[error("Couldn't find expected element '{selector}' for {role}")]
    ElementNotFound {
        selector: &'static str,
        role: &'static str,
    },
    #[error("Element '{selector}' does contain text for {role}")]
    EmptyText {
        selector: &'static str,
        role: &'static str,
    },
    #[error("Invalid text for {role}: {text}")]
    InvalidText { text: String, role: &'static str },
    #[error("Invalid link for {role}: {link}")]
    InvalidLink { link: String, role: &'static str },
    #[error("Invalid date for {role}: {date}")]
    InvalidDate { date: String, role: &'static str },
}

impl From<MallFormedTransaction> for ParseError {
    fn from(transaction: MallFormedTransaction) -> Self {
        ParseError::InvalidText {
            role: "transaction",
            text: transaction.text,
        }
    }
}

impl From<MallFormedTransaction> for ScrapeError {
    fn from(transaction: MallFormedTransaction) -> Self {
        ScrapeError::Parse(ParseError::from(transaction))
    }
}
