#![allow(dead_code)]
use anyhow;
use axum::http::header::WWW_AUTHENTICATE;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use schemars::JsonSchema;
use serde_json::json;
use std::borrow::Cow;
use std::collections::HashMap;

#[derive(JsonSchema)]
pub struct GenericError {
    pub message: String,
}

#[derive(thiserror::Error, Debug, aide::OperationIo, Default)]
pub enum Error {
    /// Return `400 Bad Request`
    #[error("{message}")]
    BadRequest { message: String },

    /// Return `401 Unauthorized`
    #[error("authentication required: {message}")]
    Unauthorized { message: String },

    /// Return `403 Forbidden`
    #[error("user may not perform that action")]
    Forbidden,

    /// Return `404 Not Found`
    #[error("{message}")]
    NotFound { message: String },

    /// Return `409 Conflict`
    #[error("request conflicts with the current state: {message}")]
    Conflict { message: String },

    /// Return `422 Unprocessable Entity`
    #[error("error in the request body")]
    UnprocessableEntity {
        errors: HashMap<Cow<'static, str>, Vec<Cow<'static, str>>>,
    },

    /// Return `500 Internal Server Error`
    #[default]
    #[error("an internal server error occurred")]
    InternalServerError,
}

#[macro_export]
macro_rules! bad_req {
    ($msg:expr) => {
        Err(Error::BadRequest {
            message: $msg.into(),
        })
    };
}

#[macro_export]
macro_rules! internal_error {
    () => {
        Error::InternalServerError
    };
}

impl Error {
    pub fn unprocessable_entity<K, V>(errors: impl IntoIterator<Item = (K, V)>) -> Self
    where
        K: Into<Cow<'static, str>>,
        V: Into<Cow<'static, str>>,
    {
        let mut error_map = HashMap::new();

        for (key, val) in errors {
            error_map
                .entry(key.into())
                .or_insert_with(Vec::new)
                .push(val.into());
        }

        Self::UnprocessableEntity { errors: error_map }
    }

    fn status_code(&self) -> StatusCode {
        match self {
            Self::BadRequest { .. } => StatusCode::BAD_REQUEST,
            Self::Unauthorized { .. } => StatusCode::UNAUTHORIZED,
            Self::Forbidden => StatusCode::FORBIDDEN,
            Self::NotFound { .. } => StatusCode::NOT_FOUND,
            Self::Conflict { .. } => StatusCode::CONFLICT,
            Self::UnprocessableEntity { .. } => StatusCode::UNPROCESSABLE_ENTITY,
            Self::InternalServerError => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        match &self {
            Self::BadRequest { message } => {
                return (self.status_code(), Json(json!({ "message": message }))).into_response()
            }

            Self::NotFound { message } => {
                return (self.status_code(), Json(json!({ "message": message }))).into_response()
            }

            Self::Conflict { message } => {
                return (self.status_code(), Json(json!({ "message": message }))).into_response()
            }

            Self::UnprocessableEntity { errors } => {
                #[derive(serde::Serialize)]
                struct Errors {
                    errors: HashMap<Cow<'static, str>, Vec<Cow<'static, str>>>,
                }

                return (
                    StatusCode::UNPROCESSABLE_ENTITY,
                    Json(Errors {
                        errors: errors.to_owned(),
                    }),
                )
                    .into_response();
            }

            Self::Unauthorized { message } => {
                return (
                    self.status_code(),
                    [(WWW_AUTHENTICATE, "Token")],
                    Json(json!({ "message": message })),
                )
                    .into_response();
            }

            Self::InternalServerError => {
                tracing::error!("Internal Server Error");
            }

            // Other errors get mapped normally.
            _ => (),
        }

        (self.status_code(), self.to_string()).into_response()
    }
}

impl From<Error> for String {
    fn from(error: Error) -> Self {
        error.to_string()
    }
}

impl From<anyhow::Error> for Error {
    fn from(error: anyhow::Error) -> Self {
        match error {
            _ => Self::InternalServerError,
        }
    }
}

impl From<sqlx::Error> for Error {
    fn from(error: sqlx::Error) -> Self {
        match error {
            sqlx::Error::RowNotFound => Self::NotFound {
                message: String::from("Item not found!"),
            },
            _ => Self::InternalServerError,
        }
    }
}
