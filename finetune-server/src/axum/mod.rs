//! Tools to serve the Finetune API on an Axum web server.
//!
//! Use [`api_router`] to obtain an [`axum::Router`] which can be integrated in any Axum web app.
//! for further details on the structure of the API, check the documentation on the [`api_router`]
//! function.
//!
//! ```
//! #use axum::Router;
//! #use tracing_subscriber::prelude::*;
//! #use tracing_finetune::FinetuneFilter;
//!
//! // Set up a tracing layer with a finetune filter and add it to the `registry`.
//! #let filter = tracing_subscriber::filter::LevelFilter::WARN;
//! #let finetune = FinetuneFilter::new(filter);
//! let finetune_handle = finetune.handle();
//! #let subscriber = tracing_subscriber::registry()
//! #    .with(tracing_subscriber::fmt::layer().with_filter(finetune))
//! #    .init();
//!
//! // Create an [`axum::Router`] attached to the finetune handle and then
//! // nest it in your own app.
//! let finetune_api = finetune_server::axum::api_router(finetune_handle);
//! let app = Router::new()
//!     // Add all your normal routes here.
//! #    .route("/", get(|| async { "Hello, World!\n" }))
//!     .nest("/finetune", finetune_api);
//! ```
use std::sync::Arc;

use ::axum::routing::{get, put};

mod handler;

/// An Axum router that provides the Finetune server API.
///
/// The API provides the following endpoints:
/// - `/api/callsites` - fetch all recorded callsites
/// - `/api/callsites/<callsite_id>/enable` - add an override to explicitly enable the callsite
///   with `callsite_id`
/// - `/api/callsites/<callsite_id>/disable` - add an override to explicitly disable the callsite
///   with `callsite_id`
/// - `/api/callsites/<callsite_id>/clear` - clear any override present for the callsite with
///   `callsite_id`
///
/// # Examples
///
/// ```
/// use axum::Router;
/// use tracing_subscriber::prelude::*;
/// use tracing_finetune::FinetuneFilter;
///
/// // Set up a tracing layer with a finetune filter.
/// let filter = tracing_subscriber::filter::LevelFilter::WARN;
/// let finetune = FinetuneFilter::new(filter);
/// let finetune_handle = finetune.handle();
/// let subscriber = tracing_subscriber::registry()
///     .with(tracing_subscriber::fmt::layer().with_filter(finetune))
///     .init();
///
/// // Create an [`axum::Router`] attached to the finetune handle and then
/// // nest it in your own app.
/// let finetune_api = finetune_server::axum::api_router(finetune_handle);
/// let app = Router::new()
///     .route(
///         "/",
///         get(|| async { "Hello, World!\n" }),
///     )
///     .nest("/finetune", finetune_api);
/// ```
pub fn api_router(finetune_handle: tracing_finetune::FinetuneHandle) -> ::axum::Router {
    use ::axum::Router;
    let callsites_api = Router::new()
        .route("/", get(handler::callsites))
        .route("/{callsite_id}/enable", put(handler::enable_callsite))
        .route("/{callsite_id}/disable", put(handler::disable_callsite))
        .route(
            "/{callsite_id}/clear",
            put(handler::clear_callsite_override),
        )
        .with_state(Arc::new(finetune_handle));

    Router::new().nest("/api/callsites", callsites_api)
}
