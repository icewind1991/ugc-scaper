use thiserror::Error;
use miette::Diagnostic;

#[derive(Debug, Error, Diagnostic)]
pub enum ScrapeError {
    #[error("Failed to request data: {0:#}")]
    Request(#[from] reqwest::Error),
    #[error(transparent)]
    #[diagnostic(transparent)]
    Parse(#[from] ParseError)
}

#[derive(Debug, Error, Diagnostic)]
pub enum ParseError {
    #[error("Couldn't find expected element '{selector}' for {role}")]
    ElementNotFound {
        selector: &'static str,
        role: &'static str
    }
}