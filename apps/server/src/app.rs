use crate::routes;
use crate::AppState;
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

pub fn get_dotenv_path() -> String {
    std::path::Path::new(file!())
        .parent()
        .expect("error on get .env path")
        .parent()
        .expect("error on get .env path")
        .join(".env")
        .to_str()
        .unwrap()
        .to_string()
}

pub fn make_app() -> (Router<AppState>, OpenApi) {
    dotenvy::from_path(get_dotenv_path()).expect(".env file not found");
    let client_url = &std::env::var("CLIENT_URL").expect("CLIENT_URL is void");

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
                .allow_origin(client_url.parse::<HeaderValue>().unwrap())
                .allow_credentials(true),
        ));

    (app, api)
}

fn api_docs(api: TransformOpenApi) -> TransformOpenApi {
    api.title("Chesu API")
        .description("Chesu API Documentation")
}
