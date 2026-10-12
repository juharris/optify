# Analysis

When a feature has many imports or several features are applied, it can be hard to tell why an option has a value.
Analysis explains which feature set the value at a path in the options and the chain of imports that brought it in.
It also lists the features whose values at that path lost, so you can see why a value from another feature was not used.

## Enabling Analysis

Analysis retains each feature's own options and its imports, which uses more memory.
It is meant for local development and debugging tools, so it is disabled by default.

Enable it when building a provider:

- Rust: set `BuilderOptions.is_analysis_enabled` to `true`.
- Node.js: call `builderOptions.setAnalysisEnabled(true)`.
- Or set `"isAnalysisEnabled": true` in `.optify/config.json` in a configuration directory.

The [CLI](../rust/optify-cli/) enables analysis automatically for its `analyze` command.

## Paths

The path to analyze is a [JSON pointer](https://datatracker.ietf.org/doc/html/rfc6901), such as `/myConfig/myKey`.
The first segment is the same key that is used to get options.
Escape `~` as `~0` and `/` as `~1` in keys, e.g., `/a~1b` for the key `a/b`.

Arrays are not merged, so a path to an item in an array, such as `/myConfig/myArray/0`, is attributed to the feature that set the whole array.

Analysis explains the options as they are in the feature files, before [configurable values](./ConfigurableStrings.md) are built.

## Results

The result has `origins` and, when the value is blocked, `blockedBy`.

Each origin has:

- `path`: the JSON pointer for the value.
- `source`: where the value came from.
  For a feature, `importChain` lists the canonical feature names from the requested feature to the feature that defined the value.
  For example, `["grandparent", "parent2", "base2"]` means that `grandparent` was requested, it imports `parent2`, and `parent2` imports `base2`, which set the value.
  `{"kind": "overrides"}` means that the value came from the overrides in the preferences.
- `overridden`: the lower-priority sources that also set a value at the path, or at an ancestor on the way to the path, and lost, highest priority first.

When the value at the path came from one source, there is exactly one origin.
When the value is an object assembled from several features, there is one origin per highest-level sub-path that came from one source.

### Example

With the [inheritance test configurations](../tests/test_suites/inheritance/configs):

- `grandparent` imports `parent1` and `parent2`.
- `parent1` imports `base1` and sets `myConfig.key2` to `"parent1 value"`.
- `parent2` imports `base2`, which sets `myConfig.key2` to `"from base2"`.
- `base1` sets `myConfig.key1`.

Analyzing `/myConfig/key2` for `["grandparent"]` gives:

```JSON
{
  "origins": [
    {
      "overridden": [{ "kind": "feature", "importChain": ["grandparent", "parent1"] }],
      "path": "/myConfig/key2",
      "source": { "kind": "feature", "importChain": ["grandparent", "parent2", "base2"] }
    }
  ]
}
```

`parent2` is imported after `parent1`, so its value has a higher priority.

Analyzing `/myConfig` for `["grandparent"]` gives one origin for `/myConfig/key1` from `base1` and one origin for `/myConfig/key2` from `base2`.

## Blocked Values

A feature can set a value at a path that still does not appear in the final options because of a non-object value at an ancestor of the path.
Instead of returning an error, analysis sets `blockedBy` and returns no `origins`:

- `path` is the ancestor.
- `source` is the feature that set the ancestor to a non-object value.
- `overridden` lists the features whose values at the requested path were blocked.

This happens in two ways:

1. A higher-priority feature sets an ancestor to a non-object.
   For example, `x` sets `k.x`, but a higher-priority feature sets `k` to `null`, so `/k/x` does not exist.
2. A middle-priority feature sets an ancestor to a non-object.
   For example, for the features `["x", "b", "high"]`, `x` sets `k.x`, `b` sets `k` to `5`, and `high` sets `k` to `{"h": 1}`.
   The final value of `k` is `{"h": 1}` because `b`'s `5` would have replaced `x`'s whole object at `k`, so `k.x` is not used.

For the second example, analyzing `/k/x` gives:

```JSON
{
  "blockedBy": {
    "overridden": [{ "kind": "feature", "importChain": ["x"] }],
    "path": "/k",
    "source": { "kind": "feature", "importChain": ["b"] }
  },
  "origins": []
}
```

Analysis returns an error when no feature or override sets anything at the path.
