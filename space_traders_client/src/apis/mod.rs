use std::error;
use std::fmt;
use std::fmt::Debug;

// #[derive(Debug, Clone)]
// pub struct ResponseContent<T> {
//     pub status: reqwest::StatusCode,
//     pub content: String,
//     pub entity: Option<T>,
// }

#[derive(Debug)]
pub enum Error<T> {
    Reqwest(reqwest::Error),
    ReqwestMiddleware(reqwest_middleware::Error),
    Serde(serde_json::Error),
    Io(std::io::Error),
    ResponseError(ResponseContent<T>),
}

#[derive(Debug)]
pub struct ResponseContent<T> {
    pub status: reqwest::StatusCode,
    pub content: String,
    pub entity: Option<ResponseContentEntity<T>>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct ResponseContentEntityData<T> {
    pub message: String,
    pub code: u32,
    pub data: T,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct ResponseContentEntity<T> {
    pub error: ResponseContentEntityData<T>,
}

impl<T> fmt::Display for Error<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (module, e) = match self {
            Error::Reqwest(e) => ("reqwest", e.to_string()),
            Error::Serde(e) => ("serde", e.to_string()),
            Error::ReqwestMiddleware(e) => ("reqwest-middleware", e.to_string()),
            Error::Io(e) => ("IO", e.to_string()),
            Error::ResponseError(e) => ("response", format!("status code {}", e.status)),
        };
        write!(f, "error in {}: {}", module, e)
    }
}

impl<T: fmt::Debug> error::Error for Error<T> {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        Some(match self {
            Error::Reqwest(e) => e,
            Error::ReqwestMiddleware(e) => e,
            Error::Serde(e) => e,
            Error::Io(e) => e,
            Error::ResponseError(_) => return None,
        })
    }
}

impl<T> From<reqwest::Error> for Error<T> {
    fn from(e: reqwest::Error) -> Self {
        Error::Reqwest(e)
    }
}

impl<T> From<reqwest_middleware::Error> for Error<T> {
    fn from(e: reqwest_middleware::Error) -> Self {
        Error::ReqwestMiddleware(e)
    }
}

impl<T> From<serde_json::Error> for Error<T> {
    fn from(e: serde_json::Error) -> Self {
        Error::Serde(e)
    }
}

impl<T> From<std::io::Error> for Error<T> {
    fn from(e: std::io::Error) -> Self {
        Error::Io(e)
    }
}

#[derive(Debug)]
pub enum ApiError {
    Reqwest(reqwest::Error),
    ReqwestMiddleware(reqwest_middleware::Error),
    Serde(serde_json::Error),
    Io(std::io::Error),
    ResponseError(ResponseContent<Result<serde_json::Value, serde_json::Error>>),
}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (module, e) = match self {
            ApiError::Reqwest(e) => ("reqwest", e.to_string()),
            ApiError::Serde(e) => ("serde", e.to_string()),
            ApiError::ReqwestMiddleware(e) => ("reqwest-middleware", e.to_string()),
            ApiError::Io(e) => ("IO", e.to_string()),
            ApiError::ResponseError(e) => ("response", format!("status code {}", e.status)),
        };
        write!(f, "error in {}: {}", module, e)
    }
}

impl error::Error for ApiError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        Some(match self {
            ApiError::Reqwest(e) => e,
            ApiError::ReqwestMiddleware(e) => e,
            ApiError::Serde(e) => e,
            ApiError::Io(e) => e,
            ApiError::ResponseError(_) => return None,
        })
    }
}

impl From<reqwest::Error> for ApiError {
    fn from(e: reqwest::Error) -> Self {
        ApiError::Reqwest(e)
    }
}

impl From<reqwest_middleware::Error> for ApiError {
    fn from(e: reqwest_middleware::Error) -> Self {
        ApiError::ReqwestMiddleware(e)
    }
}

impl From<serde_json::Error> for ApiError {
    fn from(e: serde_json::Error) -> Self {
        ApiError::Serde(e)
    }
}

impl From<std::io::Error> for ApiError {
    fn from(e: std::io::Error) -> Self {
        ApiError::Io(e)
    }
}

impl<T: serde::Serialize> From<Error<T>> for ApiError {
    fn from(value: Error<T>) -> Self {
        match value {
            Error::Reqwest(e) => ApiError::Reqwest(e),
            Error::ReqwestMiddleware(e) => ApiError::ReqwestMiddleware(e),
            Error::Serde(e) => ApiError::Serde(e),
            Error::Io(e) => ApiError::Io(e),
            Error::ResponseError(e) => ApiError::ResponseError(ResponseContent {
                status: e.status,
                content: e.content,
                entity: e.entity.map(|e| {
                    let serial_data: Result<serde_json::Value, serde_json::Error> =
                        serde_json::to_value(&e.error.data);

                    ResponseContentEntity {
                        error: ResponseContentEntityData {
                            message: e.error.message,
                            code: e.error.code,
                            data: serial_data,
                        },
                    }
                }),
            }),
        }
    }
}

pub fn urlencode<T: AsRef<str>>(s: T) -> String {
    ::url::form_urlencoded::byte_serialize(s.as_ref().as_bytes()).collect()
}

pub fn parse_deep_object(prefix: &str, value: &serde_json::Value) -> Vec<(String, String)> {
    if let serde_json::Value::Object(object) = value {
        let mut params = vec![];

        for (key, value) in object {
            match value {
                serde_json::Value::Object(_) => params.append(&mut parse_deep_object(
                    &format!("{}[{}]", prefix, key),
                    value,
                )),
                serde_json::Value::Array(array) => {
                    for (i, value) in array.iter().enumerate() {
                        params.append(&mut parse_deep_object(
                            &format!("{}[{}][{}]", prefix, key, i),
                            value,
                        ));
                    }
                }
                serde_json::Value::String(s) => {
                    params.push((format!("{}[{}]", prefix, key), s.clone()))
                }
                _ => params.push((format!("{}[{}]", prefix, key), value.to_string())),
            }
        }

        return params;
    }

    unimplemented!("Only objects are supported with style=deepObject")
}

pub mod agents_api;
pub mod contracts_api;
pub mod data_api;
pub mod default_api;
pub mod factions_api;
pub mod fleet_api;
pub mod global_api;
pub mod systems_api;

pub mod configuration;
