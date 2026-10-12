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

Output is compact single-line JSON, except for `analyze --format text`.
Use [`jq`](https://jqlang.org) to pretty-print or further process the output:

```shell
optify --dir ./configs get-options -k myConfig -f A | jq
```

### Global Options

| Option            | Description                                                                  |
| ----------------- | ---------------------------------------------------------------------------- |
| `-d, --dir <DIR>` | Path to a configuration directory; repeat to load from multiple directories. |
| `--schema <PATH>` | Optional path to a JSON schema file for validating configurations.           |

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

### `analyze`

Explain which feature (and import chain) set the value at a path in the options.
This helps when a feature has many imports or several features are applied and you need to know why an option has a value.

The path (`-p, --path`) is a [JSON pointer](https://datatracker.ietf.org/doc/html/rfc6901) such as `/myConfig/myKey`.
Escape `~` as `~0` and `/` as `~1` in keys.

```shell
optify --dir ./configs analyze -p /myConfig/key2 -f grandparent
# {"origins":[{"overridden":[{"kind":"feature","lineage":["grandparent","parent1"]}],"path":"/myConfig/key2","source":{"kind":"feature","lineage":["grandparent","parent2","base2"]}}]}
```

Use `--format text` for human-readable output:

```shell
optify --dir ./configs analyze -p /myConfig -f grandparent --format text
# /myConfig/key1: base1 (grandparent -> parent1 -> base1)
# /myConfig/key2: base2 (grandparent -> parent2 -> base2)
#   overrode: parent1 (grandparent -> parent1)
```

When features set a value at the path, but a higher-priority feature set an ancestor of the path to a non-object, the output explains what blocked the value:

```shell
optify --dir ./configs analyze -p /k/x -f x b high --format text
# /k/x is blocked: /k was set to a non-object by b (b)
#   blocked: x (x)
```

See [Analysis.md](../../docs/Analysis.md) for details about the output.

---

### `--preferences` / `--prefs`

`analyze`, `get-options`, and `get-all-options` accept a `--preferences` (or `--prefs`) flag with a JSON value.
This maps directly to the `GetOptionsPreferences` struct, so any new fields added to the struct are automatically available.

```shell
# Enable configurable string resolution
optify --dir ./configs get-options -k myConfig -f A --prefs '{"are_configurable_strings_enabled": true}'

# Apply overrides on top of the merged configuration
optify --dir ./configs get-all-options -f A --prefs '{"overrides": {"myConfig": {"key": "value"}}}'

# Filter features by constraints
optify --dir ./configs get-options -k myConfig -f A B --prefs '{"constraints": {"constraints": {"client": "Android"}}}'
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

# Explain which feature set a value
optify --dir tests/test_suites/inheritance/configs analyze -p /myConfig/key2 -f grandparent --format text
```

## Testing

```shell
cargo test
```

## Formatting

```shell
cargo fmt && cargo clippy --fix --allow-dirty --allow-staged
```
