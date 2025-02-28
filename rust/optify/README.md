# Optify in Rust

[![Crates.io](https://img.shields.io/crates/v/optify)](https://crates.io/crates/optify)
[![docs.rs](https://img.shields.io/docsrs/optify)](https://docs.rs/optify)

The core implementation of Optify in Rust.
Simplifies getting the right configuration options for a process using pre-loaded configurations from files (JSON, YAML, etc.) to manage options for experiments or flights.

See [tests](../../tests/) for examples and tests for different implementations of this format for managing options.

See the root [README.md](../../README.md) for more information and examples.

⚠️ Development in progress ⚠️\
APIs are not final and will change, for example, interfaces with be used.
This is just meant to be minimal to get started and help build Python and Ruby libraries.

## Testing

Run:
```shell
cargo test
```

## Formatting
To automatically change code, run:
```shell
cargo fmt && cargo clippy --fix --allow-dirty --allow-staged
```

## Publishing
```shell
cargo login
cargo publish
```
