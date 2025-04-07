#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Database(#[from] database::Error),
    #[error(transparent)]
    Reqwest(#[from] reqwest::Error),
    #[error(transparent)]
    ReqwestMiddleware(#[from] reqwest_middleware::Error),
    #[error(transparent)]
    Serde(#[from] serde_json::Error),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error("API error {status}: {msg} {code:?} {message:?}")]
    Api {
        status: reqwest::StatusCode,
        msg: String,
        code: Option<u32>,
        message: Option<String>,
    },
    #[error("Not enough funds: {remaining_funds} < {required_funds}")]
    NotEnoughFunds {
        remaining_funds: i64,
        required_funds: i64,
    },
    #[error("General error: {0}")]
    General(String),
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

impl<T: Clone> From<space_traders_client::apis::Error<T>> for Error {
    fn from(value: space_traders_client::apis::Error<T>) -> Self {
        match value {
            space_traders_client::apis::Error::Reqwest(error) => Error::Reqwest(error),
            space_traders_client::apis::Error::ReqwestMiddleware(error) => {
                Error::ReqwestMiddleware(error)
            }
            space_traders_client::apis::Error::Serde(error) => Error::Serde(error),
            space_traders_client::apis::Error::Io(error) => Error::Io(error),
            space_traders_client::apis::Error::ResponseError(response_content) => {
                let msg = response_content.content;
                let status = response_content.status;
                let (code, message) = response_content
                    .entity
                    .map(|entity| (entity.error.code, entity.error.message))
                    .unzip();
                Error::Api {
                    status,
                    msg,
                    code,
                    message,
                }
            }
        }
    }
}

pub type Result<T> = std::result::Result<T, Error>;
