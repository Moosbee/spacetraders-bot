#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Database(#[from] database::Error),
    #[error(transparent)]
    Api(#[from] space_traders_client::apis::ApiError),
    #[error("Not enough funds: {remaining_funds} < {required_funds}")]
    NotEnoughFunds {
        remaining_funds: i64,
        required_funds: i64,
    },

    #[error("General error: {0}")]
    General(String),
    #[error(transparent)]
    Ship(#[from] ship::Error),
}

impl From<&str> for Error {
    fn from(value: &str) -> Self {
        Error::General(value.to_string())
    }
}

impl From<String> for Error {
    fn from(value: String) -> Self {
        Error::General(value)
    }
}

impl<T: serde::Serialize> From<space_traders_client::apis::Error<T>> for Error {
    fn from(value: space_traders_client::apis::Error<T>) -> Self {
        let err = value.into();
        Error::Api(err)
    }
}

pub type Result<T> = std::result::Result<T, Error>;
