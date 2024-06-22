use crate::{http::error::Error, AppState};
use axum::{
    async_trait,
    extract::{FromRef, FromRequestParts},
    http::{header::AUTHORIZATION, request::Parts},
};
use jwt_simple::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(aide::OperationIo)]
pub struct AuthUser {
    pub user_id: Uuid,
}

#[derive(Serialize, Deserialize)]
pub struct AuthUserClaims {
    pub user_id: Uuid,
    // exp: i64,
}

impl AuthUser {
    pub fn to_jwt(&self) -> String {
        let key = HS256Key::from_bytes(std::env::var("JWT_SECRET").unwrap().as_bytes());

        let custom_claims = Claims::with_custom_claims(
            AuthUserClaims {
                user_id: self.user_id,
            },
            Duration::from_days(30),
        );

        key.authenticate(custom_claims).unwrap()
    }

    pub fn from_jwt(token: &str) -> Result<Self, Error> {
        let key = HS256Key::from_bytes(std::env::var("JWT_SECRET").unwrap().as_bytes());

        match key.verify_token::<AuthUserClaims>(&token, None) {
            Ok(claims) => Ok(Self {
                user_id: claims.custom.user_id,
            }),
            Err(_) => Err(Error::Unauthorized {
                error: "Not authorized".to_string(),
            }),
        }
    }
}

#[async_trait]
impl<S> FromRequestParts<S> for AuthUser
where
    S: Send + Sync,
    AppState: FromRef<S>,
{
    type Rejection = Error;

    async fn from_request_parts(parts: &mut Parts, _: &S) -> Result<Self, Self::Rejection> {
        let auth_header = parts
            .headers
            .get(AUTHORIZATION)
            .ok_or(Error::Unauthorized {
                error: "Not authorized".to_string(),
            })?;

        let token = auth_header.to_str().map_err(|_| Error::Unauthorized {
            error: "Not authorized".to_string(),
        })?;

        Self::from_jwt(token)
    }
}
