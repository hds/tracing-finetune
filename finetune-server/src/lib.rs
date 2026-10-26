//! A web server API to interact with [`tracing-finetune`]. It provides API support to list
//! callsites and enable and disable those at runtime.
//!
//! [`tracing-finetune`]: https://docs.rs/tracing-finetune
//!
//! Currently, `finetune-server` provides an [Axum] router which can be integrated into a web app
//! to serve the finetune API.
//!
//! Alternatively, to build an API on a different web server, just the serialization of
//! [`tracing_finetune::Callsite`] objects could be used.
//!
//! ## Usage
//!
//! ### Finetune API
//!
//! See the [`axum`] module for documentation on how to integrate the finetune API with an existing
//! Axum web server.
//!
//! This requires the `axum-router` feature to be used.
//!
//! ### Serde support
//!
//! To serialize [`tracing_finetune::Callsite`] objects, use [`serializable_callsite`].
//!
//! ## Supported Rust Versions
//!
//! `finetune-server` is built against the latest stable release. The minimum supported version is 1.90
//! ([What's New in Rust 1.88 through 1.90]). The current version of `tracing-finetune` is not
//! guaranteed to build on Rust versions earlier than the minimum supported version.
//!
//! The Minimum Supported Rust Version (MSRV) policy is that it won't be updated to a version that
//! hasn't been discussed by Jon and Ben on the [Rustacean Station] podcast.
//!
//! [What's New in Rust 1.88 through 1.90]: https://rustacean-station.org/episode/rust-1.88-1.89-1.90/
//! [Rustacean Station]: https://rustacean-station.org
//!
//! ## License
//!
//! This project is licensed under the [MIT license].
//!
//! [MIT license]: https://github.com/hds/ari_subscriber/blob/main/LICENSE
//!
//! ### Contribution
//!
//! Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in
//! `finetune-server` by you, shall be licensed as MIT, without any additional terms or conditions.
//!
//! [`axum`]: crate::axum
//! [Axum]: https://docs.rs/axum
//! [`serializable_callsite`]: ser::serializable_callsite
#![cfg_attr(docsrs, feature(doc_cfg), deny(rustdoc::broken_intra_doc_links))]

#[cfg(feature = "axum-router")]
pub mod axum;

pub mod ser;
