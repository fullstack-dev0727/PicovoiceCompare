use axum::{
    body::Body,
    Router,
    routing::get,
    routing::post,
};

use std::{net::SocketAddr};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use tower::ServiceBuilder;
use tower_http::{
    set_header::SetResponseHeaderLayer,
    trace::TraceLayer,
    cors::{CorsLayer, any},
};
use http::{header, HeaderValue};//, StatusCode};
mod routes;

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "example_templates=debug".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();
    let cors = CorsLayer::new()
    .allow_methods(any())
    .allow_origin(any())
    .allow_credentials(true)
    .allow_headers(any());

    let cors_middleware = ServiceBuilder::new()
        .layer(TraceLayer::new_for_http())
        .layer(SetResponseHeaderLayer::<_, Body>::if_not_present(
            header::ACCESS_CONTROL_ALLOW_ORIGIN,
            HeaderValue::from_static("*"),
        ))
        .layer(SetResponseHeaderLayer::<_, Body>::if_not_present(
            header::ACCESS_CONTROL_ALLOW_METHODS,
            HeaderValue::from_static("*"),
        ))
        .layer(SetResponseHeaderLayer::<_, Body>::if_not_present(
            header::ACCESS_CONTROL_ALLOW_HEADERS,
            HeaderValue::from_static("*"),
        ))
        .layer(cors)
        .into_inner();

    // build our application with some routes
    let app = Router::new()
        .route("/file-picovoice", get(routes::picovoice_route::fire))
        .route("/file-cheetah", post(routes::cheetah_route::save))
        .route("/file-leopard", post(routes::leopard_route::save))
        .layer(cors_middleware);

    // run it
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::debug!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}