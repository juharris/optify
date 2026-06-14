# Optify in Rust

[![Crates.io](https://img.shields.io/crates/v/optify)](https://crates.io/crates/optify)
[![docs.rs](https://img.shields.io/docsrs/optify)](https://docs.rs/optify)

The core implementation of Optify in Rust.
Simplifies getting the right configuration options for a process using pre-loaded configurations from files (JSON, YAML, etc.) to manage options for experiments or flights.

## Usage

Set up your configuration files as explained in the [homepage].

Load the configuration directory once when your application starts.
Use the enabled features for a request or process to get the merged options for a key:

```rust
use optify::provider::OptionsRegistry;
use optify::OptionsProvider;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct MyConfig {
    my_array: Vec<String>,
    my_object: MyObject,
    root_string: String,
}

#[derive(Debug, Deserialize)]
struct MyObject {
    deeper: Deeper,
}

#[derive(Debug, Deserialize)]
struct Deeper {
    #[serde(rename = "new")]
    new_value: String,
}

fn main() -> Result<(), String> {
    let provider = OptionsProvider::build("path/to/configs")?;
    let options = provider.get_options("myConfig", &["feature_A", "feature_B/initial"])?;
    let config: MyConfig = serde_json::from_value(options).map_err(|error| error.to_string())?;

    println!("{}", config.root_string);
    println!("{}", config.my_array.join(", "));
    println!("{}", config.my_object.deeper.new_value);

    Ok(())
}
```

Use `get_all_options` when you want the full merged options object instead of one key:

```rust
let options = provider.get_all_options(&["feature_A", "feature_B/initial"], None, None)?;
```

See [tests] for examples and tests for different implementations of this format for managing options.

## How It Works

The [`config`][config] crate (library) is used to help load configuration files.
This allows us to load many different types of files, including JSON, JSON5, YAML, and more.
We no longer use the `config` crate to combine configuration files because it was slower to merge them and deserialize the result than our custom merging logic since we know that we want to use `serde_json::Value`s.

We merge configurations starting with the first one given and thus the final feature overrides the previous ones.
We may try to optimize further in the future, but this is fine now when there are just a few features or imports and when keys are mostly unique.

Optionally, when working locally, there is support to watch for changes to the configuration files and folders using the [`notify-debouncer-full`][notify-debouncer-full] crate (library).

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

## Benchmarking
Run:
```shell
cargo build --release
cargo bench

# Run one specific benchmark, example:
cargo bench --bench get_options_benchmark
cargo bench --bench get_options_benchmark -- 'get_options/many features'

# Open one of the reports
open target/criterion/*/report/index.html
```

## Publishing
```shell
cargo login
cargo publish
```

[config]: https://crates.io/crates/config
[homepage]: https://github.com/juharris/optify
[tests]: https://github.com/juharris/optify/tree/main/tests
[notify-debouncer-full]: https://crates.io/crates/notify-debouncer-full
