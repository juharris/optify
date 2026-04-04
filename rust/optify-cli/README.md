# Optify CLI

A command-line tool for inspecting and querying [Optify](https://github.com/juharris/optify) configuration options.

## Installation

```shell
cargo install optify-cli
```

Or build from source:

```shell
cargo build --release
```

The binary will be at `target/release/optify`.

## Usage

```
optify --dir <DIR> [--dir <DIR>...] [--schema <PATH>] <COMMAND>
```

### Global Options

| Option | Description |
|--------|-------------|
| `-d, --dir <DIR>` | Path to a configuration directory. Repeat to load from multiple directories. |
| `--schema <PATH>` | Optional path to a JSON schema file for validating configurations. |

---

### `list-features`

List all available canonical feature names.

```shell
optify --dir ./configs list-features
```

Add `--include-aliases` to also show aliases:

```shell
optify --dir ./configs list-features --include-aliases
```

---

### `get-options <KEY>`

Get the options for a specific configuration key, merged across the given features (last feature wins).

```shell
optify --dir ./configs get-options myConfig --features A B
```

Use `--compact` for single-line JSON output:

```shell
optify --dir ./configs get-options myConfig -f A B --compact
```

---

### `get-all-options`

Get the full merged configuration for the given features.

```shell
optify --dir ./configs get-all-options --features A B
```

---

## Examples

```shell
# List features in a config directory
optify --dir tests/test_suites/simple/configs list-features

# Get options for the "myConfig" key with feature A active
optify --dir tests/test_suites/simple/configs get-options myConfig -f A

# Get options for the "myConfig" key with features A and B active (B overrides A)
optify --dir tests/test_suites/simple/configs get-options myConfig -f A B

# Get the full merged configuration for features A and B
optify --dir tests/test_suites/simple/configs get-all-options -f A B

# Load from multiple directories
optify --dir ./base-configs --dir ./override-configs get-options myConfig -f A
```

## Testing

```shell
cargo test
```

## Formatting

```shell
cargo fmt && cargo clippy --fix --allow-dirty --allow-staged
```
