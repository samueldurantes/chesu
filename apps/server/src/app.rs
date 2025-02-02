use crate::http::Env;
use crate::routes;
use aide::{axum::ApiRouter, openapi::OpenApi, transform::TransformOpenApi};
use axum::{
    http::{
        header::{HeaderValue, AUTHORIZATION, CONTENT_TYPE},
        Method,
    },
    Extension, Router,
};
use std::{sync::Arc, time::Duration};
use tower_http::{
    catch_panic::CatchPanicLayer, compression::CompressionLayer, cors::CorsLayer,
    sensitive_headers::SetSensitiveHeadersLayer, timeout::TimeoutLayer, trace::TraceLayer,
};

fn make_app_with_api() -> (Router, OpenApi) {
    Env::init();

    let client_url_parsed = Env::get().client_url.parse::<HeaderValue>().unwrap();

    aide::gen::on_error(|error| {
        println!("{error}");
    });

    aide::gen::extract_schemas(true);

    let mut api = OpenApi::default();
    let app = ApiRouter::new()
        .merge(routes::mount())
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
                .allow_origin(client_url_parsed)
                .allow_credentials(true),
        ));

    (app, api)
}

pub fn make_app() -> Router {
    make_app_with_api().0
}

pub fn make_api() -> OpenApi {
    make_app_with_api().1
}

fn api_docs(api: TransformOpenApi) -> TransformOpenApi {
    api.title("Chesu API")
        .description("Chesu API Documentation")
}
