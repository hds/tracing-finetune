# finetune-server

A web server API to interact with [`tracing-finetune`]. It provides API support to list callsites
and enable and disable those at runtime.

Currently, `finetune-server` provides an [Axum] router which can be integrated into a web app
to serve the finetune API.

Alternatively, to build an API on a different web server, just the serialization of
[`tracing_finetune::Callsite`] objects could be used.

## Usage

### Finetune API

See the [`axum`] module for documentation on how to integrate the finetune API with an existing
Axum web server.

This requires the `axum-router` feature to be used.

### Serde support

To serialize [`tracing_finetune::Callsite`] objects, use [`serializable_callsite`].

### Example

Run the example with `cargo`:

```sh
cargo run --features axum-router --example example-server-axum
```

The server has a hello world endpoint at `/`:

```
$ curl 127.0.0.1:3007/
Hello, World!
```

The API can be used to get all callsites (the response is JSON):

```sh
curl 127.0.0.1:3007/finetune/api/callsites
```

Individual callsites can have an override created to enable or disable them:

```sh
curl -X PUT 127.0.0.1:3007/finetune/api/callsites/4307429688/enable
curl -X PUT 127.0.0.1:3007/finetune/api/callsites/4307429688/disable
```

The override can be removed too:

```sh
curl -X PUT 127.0.0.1:3007/finetune/api/callsites/4307429688/clear
```

## Supported Rust Versions

`finetune-server` is built against the latest stable release. The minimum supported version is 1.90
([What's New in Rust 1.88 through 1.90]). The current version of `tracing-finetune` is not
guaranteed to build on Rust versions earlier than the minimum supported version.

The Minimum Supported Rust Version (MSRV) policy is that it won't be updated to a version that
hasn't been discussed by Jon and Ben on the [Rustacean Station] podcast.

[What's New in Rust 1.88 through 1.90]: https://rustacean-station.org/episode/rust-1.88-1.89-1.90/
[Rustacean Station]: https://rustacean-station.org

## License

This project is licensed under the [MIT license].

[MIT license]: https://github.com/hds/tracing-finetune/blob/main/LICENSE

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in
`finetune-server` by you, shall be licensed as MIT, without any additional terms or conditions.

[Axum]: https://docs.rs/axum
[`axum`]: https://docs.rs/finetune-server/latest/finetune_server/axum/index.html
[`serializable_callsite`]: https://docs.rs/finetune-server/latest/finetune_server/ser/fn.serializable_callsite.html
[`tracing-finetune`]: https://docs.rs/tracing-finetune
[`tracing_finetune::Callsite`]: https://docs.rs/tracing-finetune/latest/tracing_finetune/struct.Callsite.html
