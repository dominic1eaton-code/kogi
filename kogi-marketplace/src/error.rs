use thiserror::Error;

#[derive(Debug, Error)]
pub enum MarketplaceError {
    #[error("not found: {0}")]
    NotFound(String),
    #[error("invalid request: {0}")]
    Invalid(String),
    #[error("portfolio error: {0}")]
    Portfolio(#[from] kogi_portfolio::KogiPortfolioError),
}

pub type MarketplaceResult<T> = Result<T, MarketplaceError>;
