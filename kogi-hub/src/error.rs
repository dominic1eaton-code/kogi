use thiserror::Error;

#[derive(Debug, Error)]
pub enum HubError {
    #[error("not found: {0}")]
    NotFound(String),
    #[error("invalid request: {0}")]
    Invalid(String),
    #[error("portfolio error: {0}")]
    Portfolio(#[from] kogi_portfolio::KogiPortfolioError),
}

pub type HubResult<T> = Result<T, HubError>;
