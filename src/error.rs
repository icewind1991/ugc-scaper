use miette::Diagnostic;
use thiserror::Error;

#[derive(Debug, Error, Diagnostic)]
pub enum ScrapeError {
    #[error("Failed to request data: {0:#}")]
    Request(#[from] reqwest::Error),
    #[error(transparent)]
    #[diagnostic(transparent)]
    Parse(#[from] ParseError),
}

#[derive(Debug, Error, Diagnostic, Clone)]
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
    #[error("Invalid link for {role}: {link}")]
    InvalidLink { link: String, role: &'static str },
    #[error("Invalid date for {role}: {date}")]
    InvalidDate { date: String, role: &'static str },
}
