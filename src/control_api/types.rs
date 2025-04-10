use database::Agent;
use ship::MyShip;

#[derive(Debug, Clone, serde::Serialize)]
pub struct WsObject {
    pub data: WsData,
}

#[derive(Debug, Clone, serde::Serialize)]
#[serde(tag = "type", content = "data")]
pub enum WsData {
    RustShip(MyShip),
    MyAgent(Agent),
}

pub struct MyReceiver<T: Clone>(pub tokio::sync::broadcast::Receiver<T>);

impl<T: Clone> Clone for MyReceiver<T> {
    fn clone(&self) -> Self {
        MyReceiver(self.0.resubscribe())
    }
}

pub type Result<T> = std::result::Result<T, warp::Rejection>;

#[derive(thiserror::Error, Debug)]
pub enum ServerError {
    #[error("Database error: {0}")]
    Database(#[from] database::Error),
    #[error("Not found")]
    NotFound,
    #[error("Server error: {0}")]
    Server(String),
    #[error("Invalid request: {0}")]
    BadRequest(String),
    #[error("API error: {status} {message}")]
    APIError { status: u16, message: String },
}

impl From<crate::error::Error> for ServerError {
    fn from(value: crate::error::Error) -> Self {
        match value {
            crate::error::Error::Database(error) => ServerError::Database(error),
            crate::error::Error::NotEnoughFunds {
                remaining_funds,
                required_funds,
            } => ServerError::Server(format!(
                "Not enough funds: {} {}",
                remaining_funds, required_funds
            )),
            crate::error::Error::General(_) => ServerError::Server(value.to_string()),
            crate::error::Error::Api(api_error) => match api_error {
                space_traders_client::apis::ApiError::ResponseError(response_content) => {
                    ServerError::APIError {
                        status: response_content.status.as_u16(),
                        message: response_content.content
                            + response_content
                                .entity
                                .as_ref()
                                .map(|e| {
                                    format!(
                                        "\n{}: {}\n{:?}",
                                        e.error.code, e.error.message, e.error.data
                                    )
                                })
                                .unwrap_or("No Response".to_string())
                                .as_str(),
                    }
                }

                space_traders_client::apis::ApiError::Reqwest(error) => ServerError::APIError {
                    status: error.status().map(|s| s.as_u16()).unwrap_or(500),
                    message: error.to_string(),
                },
                space_traders_client::apis::ApiError::ReqwestMiddleware(error) => {
                    ServerError::APIError {
                        status: error.status().map(|s| s.as_u16()).unwrap_or(500),
                        message: error.to_string(),
                    }
                }
                space_traders_client::apis::ApiError::Serde(error) => {
                    ServerError::Server(error.to_string())
                }
                space_traders_client::apis::ApiError::Io(error) => {
                    ServerError::Server(error.to_string())
                }
            },
            crate::error::Error::Ship(error) => match error {
                ship::Error::Database(error) => ServerError::Database(error.into()),
                ship::Error::Api(api_error) => {
                    ServerError::from(Into::<crate::error::Error>::into(api_error))
                }
                ship::Error::General(message) => ServerError::Server(message),
            },
        }
    }
}

impl<T: Clone> From<space_traders_client::apis::Error<T>> for ServerError {
    fn from(value: space_traders_client::apis::Error<T>) -> Self {
        match value {
            space_traders_client::apis::Error::Reqwest(error) => ServerError::APIError {
                status: error.status().map(|s| s.as_u16()).unwrap_or(500),
                message: error.to_string(),
            },
            space_traders_client::apis::Error::ReqwestMiddleware(error) => ServerError::APIError {
                status: error.status().map(|s| s.as_u16()).unwrap_or(500),
                message: error.to_string(),
            },
            space_traders_client::apis::Error::Serde(error) => {
                ServerError::Server(error.to_string())
            }
            space_traders_client::apis::Error::Io(error) => ServerError::Server(error.to_string()),
            space_traders_client::apis::Error::ResponseError(response_content) => {
                ServerError::APIError {
                    status: response_content.status.as_u16(),
                    message: response_content.content,
                }
            }
        }
    }
}

impl warp::reject::Reject for ServerError {}
