# Optify for Elixir

An Elixir library for [Optify](../../README.md) powered by Rust NIFs via [Rustler](https://github.com/rusterlium/rustler).

## Requirements

- Elixir >= 1.18
- Erlang/OTP >= 27
- Rust toolchain (for compiling the NIF)

## Installation

*Coming soon*

## Usage

```Elixir
# Build a provider from a directory of configuration files
provider = Optify.OptionsProvider.build!("path/to/configs")

# Get all options for enabled features
options = Optify.OptionsProvider.get_all_options(provider, ["feature_A", "feature_B"])
# %{
#   "myConfig" => %{
#     "myArray" => ["example item 1"],
#     "myObject" => %{
#       "deeper" => %{"list" => [1, 2], "wtv" => 3},
#       "one" => 1,
#       "string" => "string",
#       "two" => 2
#     },
#     "rootString" => "root string same",
#     "rootString2" => "overridden"
#   }
# }

# Get options for a specific key
options = Optify.OptionsProvider.get_options(provider, "myConfig", ["feature_A", "feature_B"])
# %{
#   "myArray" => ["example item 1"],
#   "myObject" => %{...},
#   "rootString" => "root string same",
#   "rootString2" => "overridden"
# }

# Use preferences for configurable strings and constraints
prefs = Optify.GetOptionsPreferences.new()
prefs = Optify.GetOptionsPreferences.enable_configurable_strings(prefs)
options = Optify.OptionsProvider.get_all_options(provider, ["feature_A"], prefs)

# Set overrides using a native Elixir map
prefs = Optify.GetOptionsPreferences.set_overrides(prefs, %{"myConfig" => %{"rootString" => "new value"}})

# Or using a JSON string
prefs = Optify.GetOptionsPreferences.set_overrides_json(prefs, ~s({"myConfig": {"rootString": "new value"}}))

# Use the builder for multiple directories
builder = Optify.OptionsProviderBuilder.new()
builder |> Optify.OptionsProviderBuilder.add_directory!("path/to/configs1")
builder |> Optify.OptionsProviderBuilder.add_directory!("path/to/configs2")
provider = Optify.OptionsProviderBuilder.build!(builder)
```

## Development

### Prerequisites

Install Erlang and Elixir using [asdf](https://asdf-vm.com/):

```shell
asdf install
```

### Running Tests

```shell
mix deps.get
mix test
```

### Formatting

Elixir:
```shell
mix format
mix format --check-formatted
```

Rust (NIF code):
```shell
cd native/optify_nif
cargo fmt
cargo fmt -- --check
```

### Linting

```shell
cd native/optify_nif
cargo clippy --no-deps -- -D warnings
```
