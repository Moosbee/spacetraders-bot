#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Database(#[from] database::Error),
    #[error(transparent)]
    ArcDatabase(#[from] std::sync::Arc<database::Error>),
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
    #[error("Reservation not found: {reservation_id}")]
    ReservationNotFound { reservation_id: i64 },

    #[error("Fleet not found: fleet_id={fleet_id}, assignment_id={assignment_id:?}")]
    FleetNotFound {
        fleet_id: i32,
        assignment_id: Option<i64>,
    },

    #[error(transparent)]
    ArcError(#[from] std::sync::Arc<Error>),
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
