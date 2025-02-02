use crate::http::Error;
use crate::Env;
use axum::RequestPartsExt;
use axum::{async_trait, extract::FromRequestParts, http::request::Parts};
use axum_extra::{headers::Cookie, TypedHeader};
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub static COOKIE_NAME: &str = "CHESU_TOKEN";

#[derive(aide::OperationIo)]
pub struct AuthUser {
    pub user_id: Uuid,
}

#[derive(Debug, Serialize, Deserialize)]
struct AuthUserClaims {
    user_id: Uuid,
    exp: usize,
}

impl AuthUser {
    pub fn to_jwt(&self) -> String {
        let expiration = chrono::Utc::now()
            .checked_add_signed(chrono::Duration::days(30))
            .expect("Failed to create expiration date")
            .timestamp() as usize;

        let claims = AuthUserClaims {
            user_id: self.user_id,
            exp: expiration,
        };

        encode(
            &Header::new(Algorithm::HS256),
            &claims,
            &EncodingKey::from_secret(Env::get().jwt_secret.as_bytes()),
        )
        .expect("Failed to encode JWT")
    }

    pub fn from_jwt(token: &str) -> Result<Self, Error> {
        match decode::<AuthUserClaims>(
            token,
            &DecodingKey::from_secret(Env::get().jwt_secret.as_bytes()),
            &Validation::new(Algorithm::HS256),
        ) {
            Ok(data) => Ok(Self {
                user_id: data.claims.user_id,
            }),
            Err(_) => Err(Error::Unauthorized {
                message: "Not authorized".to_string(),
            }),
        }
    }
}

#[async_trait]
impl<S> FromRequestParts<S> for AuthUser
where
    S: Send + Sync,
{
    type Rejection = Error;

    async fn from_request_parts(parts: &mut Parts, _: &S) -> Result<Self, Self::Rejection> {
        let cookies =
            parts
                .extract::<TypedHeader<Cookie>>()
                .await
                .map_err(|_| Error::Unauthorized {
                    message: "Not authorized".to_string(),
                })?;

        let token = cookies.get(COOKIE_NAME).ok_or(Error::Unauthorized {
            message: "Not authorized".to_string(),
        })?;

        Self::from_jwt(token)
    }
}
