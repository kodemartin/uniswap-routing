#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("max number of retries reached")]
    GetPoolsMaxRetries,
    #[error(transparent)]
    Reqwest(#[from] reqwest::Error),
}

pub type Result<T> = std::result::Result<T, Error>;
