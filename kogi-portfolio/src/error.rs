use thiserror::Error;

#[derive(Debug, Error)]
pub enum KogiPortfolioError {
    #[error("apapo error: {0}")]
    Apapo(#[from] apapo::ApapoError),
    #[error("hypergrid error: {0}")]
    Hypergrid(#[from] hypergrid::error::HypergridError),
    #[error("portfolio error: {0}")]
    Portfolio(String),
}

pub type KogiPortfolioResult<T> = Result<T, KogiPortfolioError>;
