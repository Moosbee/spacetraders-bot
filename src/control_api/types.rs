use crate::{ship::MyShip, sql::Agent};

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
    Database(#[from] sqlx::Error),
    #[error("Server error: {0}")]
    Server(String),
    #[error("Invalid request: {0}")]
    BadRequest(String),
    #[error("API error: {status} {message}")]
    APIError { status: u16, message: String },
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
