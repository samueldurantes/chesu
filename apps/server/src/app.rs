use std::{sync::Arc, time::Duration};

use crate::{http, State};
use aide::{axum::ApiRouter, openapi::OpenApi, transform::TransformOpenApi};
use axum::{
    http::{
        header::{AUTHORIZATION, CONTENT_TYPE},
        Method,
    },
    Extension, Router,
};
use tower_http::{
    catch_panic::CatchPanicLayer,
    compression::CompressionLayer,
    cors::{Any, CorsLayer},
    sensitive_headers::SetSensitiveHeadersLayer,
    timeout::TimeoutLayer,
    trace::TraceLayer,
};

pub fn make_app() -> (Router<State>, OpenApi) {
    aide::gen::on_error(|error| {
        println!("{error}");
    });

    aide::gen::extract_schemas(true);

    let mut api = OpenApi::default();
    let app = ApiRouter::new()
        // Auth routes
        .merge(http::auth::router())
        // Docs routes
        .merge(http::docs::router())
        // Game routes
        .merge(http::game::router())
        // User routes
        .merge(http::user::router())
        .finish_api_with(&mut api, api_docs)
        .layer((
            SetSensitiveHeadersLayer::new([AUTHORIZATION]),
            CompressionLayer::new(),
            TraceLayer::new_for_http().on_failure(()),
            TimeoutLayer::new(Duration::from_secs(30)),
            CatchPanicLayer::new(),
            Extension(Arc::new(api.clone())),
            CorsLayer::new()
                .allow_methods([Method::GET, Method::POST])
                .allow_headers([CONTENT_TYPE])
                .allow_origin(Any),
        ));

    (app, api)
}

fn api_docs(api: TransformOpenApi) -> TransformOpenApi {
    api.title("Chesu API")
        .description("Chesu API Documentation")
}
