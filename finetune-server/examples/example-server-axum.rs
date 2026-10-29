use axum::{
    Router,
    extract::Request,
    http::{Response, StatusCode, header},
    response::IntoResponse,
    routing::get,
};
use tower_http::trace::TraceLayer;
use tracing_finetune::{FinetuneFilter, FinetuneHandle};
use tracing_subscriber::{fmt::format::FmtSpan, prelude::*};

use finetune_server::axum::api_router;

#[tokio::main]
async fn main() {
    let filter = tracing_subscriber::filter::LevelFilter::WARN;
    let finetune = FinetuneFilter::new(filter);
    let finetune_handle = finetune.handle();
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::fmt::layer()
                .with_span_events(FmtSpan::CLOSE)
                .with_filter(finetune),
        )
        .init();

    let app = Router::new()
        .route(
            "/",
            get(|| async {
                {
                    let _span_guard =
                        tracing::info_span!("work-span", work_type = "busy").entered();
                    tracing::debug!("Do some work");
                }

                tracing::info!("Hello, World!");

                "Hello, World!\n"
            }),
        )
        .route("/health", get(|| async { "OK\n" }))
        .nest("/finetune", webapp_router(finetune_handle))
        .layer(TraceLayer::new_for_http())
        .fallback(default_fallback);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3007").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn default_fallback(request: Request) -> impl IntoResponse {
    tracing::warn!(uri = %request.uri(), "Not Found");

    (StatusCode::NOT_FOUND, "Not Found\n")
}

fn webapp_router(finetune_handle: FinetuneHandle) -> Router {
    api_router(finetune_handle)
        .route("/", get(index_html))
        .route("/finetune.css", get(finetune_css))
        .route("/finetune.js", get(finetune_js))
}

async fn index_html() -> impl IntoResponse {
    const INDEX_HTML: &'static str = include_str!("static/index.html");

    Response::builder()
        .header(header::CONTENT_TYPE, "text/html")
        .body(INDEX_HTML.to_string())
        .expect("known content should not fail")
}

async fn finetune_css() -> impl IntoResponse {
    const FINETUNE_CSS: &'static str = include_str!("static/finetune.css");

    Response::builder()
        .header(header::CONTENT_TYPE, "text/css")
        .body(FINETUNE_CSS.to_string())
        .expect("known content should not fail")
}

async fn finetune_js() -> impl IntoResponse {
    const FINETUNE_JS: &'static str = include_str!("static/finetune.js");

    Response::builder()
        .header(header::CONTENT_TYPE, "text/javascript")
        .body(FINETUNE_JS.to_string())
        .expect("known content should not fail")
}
