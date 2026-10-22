# finetune-server

A web server API to interact with [`tracing-finetune`]. It provides API support to list callsites
and enable and disable those at runtime.

At present, this crate only provides an example showing how to set up an API using [Axum].

[`tracing-finetune`]: https://docs.rs/tracing-finetune
[Axum]: https://docs.rs/axum

# Usage

Run the example with `cargo`:

```sh
cargo run --example server
```

The server has a hello world endpoint at `/`:

```
$ curl 127.0.0.1:3007/
Hello, World!
```

The API can be used to get all callsites (the response is JSON):

```sh
curl 127.0.0.1:3007/api/callsites
```

Individual callsites can have an override created to enable or disable them:

```sh
curl -X PUT 127.0.0.1:3007/api/callsites/4307429688/enable
curl -X PUT 127.0.0.1:3007/api/callsites/4307429688/disable
```

The override can be removed too:

```sh
curl -X PUT 127.0.0.1:3007/api/callsites/4307429688/clear
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

[MIT license]: https://github.com/hds/ari_subscriber/blob/main/LICENSE

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in
`finetune-server` by you, shall be licensed as MIT, without any additional terms or conditions.
