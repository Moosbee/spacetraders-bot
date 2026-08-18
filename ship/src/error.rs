#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Database(#[from] database::Error),

    #[error(transparent)]
    Api(#[from] space_traders_client::apis::ApiError),

    #[error("General error: {0}")]
    General(String),

    #[error("No route found for {symbol} from {from} to {to} with nav stats {nav_stats:?}")]
    NoRouteFound {
        symbol: String,
        from: String,
        to: String,
        nav_stats: crate::autopilot::ShipNavStats,
    },
}

impl From<&str> for Error {
    fn from(value: &str) -> Self {
        Error::General(value.to_string())
    }
}

impl<T: serde::Serialize> From<space_traders_client::apis::Error<T>> for Error {
    fn from(value: space_traders_client::apis::Error<T>) -> Self {
        let err = value.into();
        Error::Api(err)
    }
}

pub type Result<T> = core::result::Result<T, Error>;
