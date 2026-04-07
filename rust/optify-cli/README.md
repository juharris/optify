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

Output is always compact single-line JSON.
Use [`jq`](https://jqlang.org) to pretty-print or further process the output:

```shell
optify --dir ./configs get-options -k myConfig -f A | jq
```

### Global Options

| Option | Description |
|--------|-------------|
| `-d, --dir <DIR>` | Path to a configuration directory. Repeat to load from multiple directories. |
| `--schema <PATH>` | Optional path to a JSON schema file for validating configurations. |

---

### `list-features`

List all features with their metadata as a JSON array.

```shell
optify --dir ./configs list-features
# [{"aliases":["short"],"details":...,"name":"featureName","owners":"team@co.com",...}, ...]

optify --dir ./configs list-features | jq '.[].name'
# "featureA"
# "featureB"
```

---

### `get-options`

Get the options for a specific configuration key, merged across the given features (last feature wins).

```shell
optify --dir ./configs get-options --key myConfig --features A B
```

Feature names with spaces must be quoted:

```shell
optify --dir ./configs get-options -k myConfig -f "feature with spaces" A
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
# List features with metadata in a config directory
optify --dir tests/test_suites/simple/configs list-features | jq

# Get options for the "myConfig" key with feature A active
optify --dir tests/test_suites/simple/configs get-options -k myConfig -f A

# Get options for the "myConfig" key with features A and B active (B overrides A)
optify --dir tests/test_suites/simple/configs get-options -k myConfig -f A B

# Pretty-print with jq
optify --dir tests/test_suites/simple/configs get-options -k myConfig -f A | jq

# Get the full merged configuration for features A and B
optify --dir tests/test_suites/simple/configs get-all-options -f A B

# Load from multiple directories
optify --dir ./base-configs --dir ./override-configs get-options -k myConfig -f A
```

## Testing

```shell
cargo test
```

## Formatting

```shell
cargo fmt && cargo clippy --fix --allow-dirty --allow-staged
```


