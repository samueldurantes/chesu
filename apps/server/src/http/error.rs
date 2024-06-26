use axum::http::header::WWW_AUTHENTICATE;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use schemars::JsonSchema;
use serde_json::json;
use sqlx::error::DatabaseError;
use std::borrow::Cow;
use std::collections::HashMap;

#[derive(JsonSchema)]
pub struct GenericError {
    pub message: String,
}

#[derive(thiserror::Error, Debug, aide::OperationIo)]
pub enum Error {
    /// Return `400 Bad Request`
    #[error("bad request")]
    BadRequest { message: String },

    /// Return `401 Unauthorized`
    #[error("authentication required")]
    Unauthorized { message: String },

    /// Return `403 Forbidden`
    #[error("user may not perform that action")]
    Forbidden,

    /// Return `404 Not Found`
    #[error("request path not found")]
    NotFound { message: String },

    /// Return `422 Unprocessable Entity`
    #[error("error in the request body")]
    UnprocessableEntity {
        errors: HashMap<Cow<'static, str>, Vec<Cow<'static, str>>>,
    },

    #[error("an error occurred with the database")]
    Sqlx(#[from] sqlx::Error),

    #[error("an internal server error occurred")]
    Anyhow(#[from] anyhow::Error),
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
            Self::UnprocessableEntity { .. } => StatusCode::UNPROCESSABLE_ENTITY,
            Self::Sqlx(_) | Self::Anyhow(_) => StatusCode::INTERNAL_SERVER_ERROR,
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

            Self::Sqlx(e) => {
                tracing::error!("SQLx error: {:?}", e);
            }

            Self::Anyhow(e) => {
                tracing::error!("Generic error: {:?}", e);
            }

            // Other errors get mapped normally.
            _ => (),
        }

        (self.status_code(), self.to_string()).into_response()
    }
}

pub trait ResultExt<T> {
    fn on_constraint(
        self,
        name: &str,
        f: impl FnOnce(Box<dyn DatabaseError>) -> Error,
    ) -> Result<T, Error>;
}

impl<T, E> ResultExt<T> for Result<T, E>
where
    E: Into<Error>,
{
    fn on_constraint(
        self,
        name: &str,
        map_err: impl FnOnce(Box<dyn DatabaseError>) -> Error,
    ) -> Result<T, Error> {
        self.map_err(|e| match e.into() {
            Error::Sqlx(sqlx::Error::Database(dbe)) if dbe.constraint() == Some(name) => {
                map_err(dbe)
            }
            e => e,
        })
    }
}
