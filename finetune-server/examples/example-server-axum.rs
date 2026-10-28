use axum::{Router, extract::Request, response::IntoResponse, routing::get};
use tower_http::{services::ServeDir, trace::TraceLayer};
use tracing_finetune::FinetuneFilter;
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
        .nest("/finetune", api_router(finetune_handle))
        .nest_service("/finetune", ServeDir::new("static"))
        .layer(TraceLayer::new_for_http())
        .fallback(default_fallback);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3007").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn default_fallback(request: Request) -> impl IntoResponse {
    tracing::warn!(uri = %request.uri(), "Not Found");

    "Not Found\n"
}
