# Optify Rust Bindings for Ruby

[![Crates.io](https://img.shields.io/crates/v/optify_ruby)](https://crates.io/crates/optify_ruby)
[![docs.rs](https://img.shields.io/docsrs/optify_ruby)](https://docs.rs/optify_ruby)

⚠️ Development in progress ⚠️\
APIs are not final and will change, for example, interfaces with be used.
This is just meant to be minimal to get started and help build a Ruby library.

## Testing

Run:
```shell
cargo test
```

## Formatting
To automatically change code, run:
```shell
cargo fmt
cargo clippy --fix --allow-dirty --allow-staged
```

## Publishing
```shell
cargo login
cargo publish
```
