# Tracing Finetune

A [`tracing`] filter which can be finetuned to enable or disable specific callsites.

This crate provides [`FinetuneFilter`] which implements the [`Filter`] trait from
[`tracing-subscriber`]. It wraps another filter, but can override whether or not specific
callsites (places where events or spans are created) are recorded. This finetuning can be
performed at runtime.

[`FinetuneFilter`]: https://docs.rs/tracing-finetune/latest/tracing_finetune/struct.FinetuneFilter.html
[`Filter`]: https://docs.rs/tracing-subscriber/latest/tracing_subscriber/layer/trait.Filter.html
[`tracing`]: https://docs.rs/tracing
[`tracing-subscriber`]: https://docs.rs/tracing-subscriber

## Usage

A finetune filter can wrap any other filter and then be added to filter a layer.

```rust
use tracing_subscriber::prelude::*;
use tracing_finetune::FinetuneFilter;

let filter = tracing_subscriber::filter::LevelFilter::WARN;
let finetune = FinetuneFilter::new(filter);
let finetune_handle = finetune.handle();
let subscriber = tracing_subscriber::registry()
    .with(tracing_subscriber::fmt::layer().with_filter(finetune))
    .init();

fn info_event() {
    tracing::info!("hi there!");
}

fn warn_event() {
    tracing::warn!("important!");
}

info_event(); // Doesn't match WARN filter: not emitted
warn_event(); // Matches WARN filter: emitted

let callsites = finetune_handle.callsites();

assert_eq!(callsites.len(), 2_usize);
// Callsites are returned in the order they are encounted during execution
let info_callsite = &callsites[0];
let warn_callsite = &callsites[1];

// Here we explicitly enable the info callsite
finetune_handle.enable_callsite(info_callsite.id());
// and explicitly disable the warn callsite
finetune_handle.disable_callsite(warn_callsite.id());


info_event(); // Explictly enabled: emitted
warn_event(); // Explicitly disabled: not emitted
```

## Supported Rust Versions

`tracing-finetune` is built against the latest stable release. The minimum supported version is 1.90
([What's New in Rust 1.88 through 1.90]). The current version of `tracing-finetune` is not
guaranteed to build on Rust versions earlier than the minimum supported version.

The Minimum Supported Rust Version (MSRV) policy is that it won't be updated to a version that
hasn't been discussed by Jon and Ben on the [Rustacean Station] podcast.

[What's New in Rust 1.88 through 1.90]: https://rustacean-station.org/episode/rust-1.88-1.89-1.90/
[Rustacean Station]: https://rustacean-station.org

## License

This project is licensed under the [MIT license].

[MIT license]: https://github.com/hds/ari_subscriber/blob/main/LICENSE

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in
`tracing-finetune` by you, shall be licensed as MIT, without any additional terms or conditions.
