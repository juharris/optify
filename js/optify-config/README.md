# Optify Bindings for Node.js

[![NPM Version](https://img.shields.io/npm/v/%40optify%2Fconfig?color=bc3433)](https://www.npmjs.com/package/@optify/config)

See the [homepage] for details about how feature files are combined to build the options to process at runtime.

# Usage

```TypeScript
import { z } from 'zod'
import { OptionsProvider } from '@optify/config'

const MyConfigSchema = z.object({
  rootString: z.string(),
  myArray: z.array(z.string()),
  myObject: z.object({
    key: z.string(),
    deeper: z.object({
      new: z.string(),
      num: z.number(),
    }),
  }),
}).readonly()

const provider = OptionsProvider.build('<configs folder path>')
const config = provider.getOptions('myConfig', ['feature_A', 'feature_B'], MyConfigSchema)
// config is typed and validated at runtime
console.log(config.rootString)
console.log(config.myObject.deeper.num)
```

The `getOptions` method accepts any object with a `parse(data: unknown): T` method, making it compatible with [Zod](https://zod.dev/) and other schema validation libraries.
This package does not depend on Zod; it only requires the schema to have a `parse` method.
If desired, install Zod (or your preferred validation library) separately in your project:

```shell
npm install zod
```

### Using `getOptionsJson`

If you don't need schema validation, you can use `getOptionsJson` to get the raw JSON string:

```TypeScript
import { OptionsProvider } from '@optify/config'

const provider = OptionsProvider.build('<configs folder path>')
const options = JSON.parse(provider.getOptionsJson('myConfig', ['feature_A', 'feature_B']))
console.log(JSON.stringify(options, null, 2))
```

Outputs:

```JSON
{
  "myArray": [
    "item 1",
    "item 2"
  ],
  "myObject": {
    "deeper": {
      "new": "new value",
      "num": 3333
    },
    "key": "val"
  },
  "rootString": "root string same"
}
```

### Multiple directories

```TypeScript
import { OptionsProvider } from '@optify/config'

const provider = OptionsProvider.buildFromDirectories(['<configs folder path>', '<another folder path>'])
```

### Scoping to features and overrides

A `ScopedProvider` holds a set of features and overrides so that code which gets options doesn't need to know them.
It works with an `OptionsProvider` or an `OptionsWatcher`.
The features must be canonical feature names: they are not converted from aliases.
Configurable values are enabled by default.

```TypeScript
import { OptionsProvider, ScopedProvider } from '@optify/config'

const provider = OptionsProvider.build('<configs folder path>')
const scoped = new ScopedProvider(provider, ['feature_A', 'feature_B/initial'], { myConfig: { rootString: 'override' } })
const config = scoped.getOptions('myConfig', MyConfigSchema)
const allOptions = scoped.getAllOptions()
```

`getOptions` and `getAllOptions` also accept optional `GetOptionsPreferences` and `CacheOptions`.
The given preferences are not changed.
If the scope has overrides, then they replace any overrides in the given preferences.
When `CacheOptions` are given, the cache of the provider or watcher is used, so the same caching rules apply, including that caching is not supported when there are overrides.
When called without preferences or `CacheOptions`, `getOptions` caches its parsed result by key in the `ScopedProvider`.

The features and overrides are stored once in the Rust implementation, which does the work of getting options, so they are not copied to JavaScript for each call without a cache.

Use `serialize` and `ScopedProvider.deserialize` to transfer or persist a scope.
`overrides` is optional in the serialized value.

```TypeScript
const restored = ScopedProvider.deserialize(provider, scoped.serialize())
```

Constraints are not part of a scope.
Resolve constraint-based filtering with `getFilteredFeatures` before creating the scope.

### Analyzing where a value comes from

To explain which feature (and import chain) set a value, enable analysis and pass a [JSON pointer](https://datatracker.ietf.org/doc/html/rfc6901) to the value.
Analysis retains more data, so it is meant for local development and debugging tools.

```TypeScript
import { BuilderOptions, OptionsProvider } from '@optify/config'

const builderOptions = new BuilderOptions()
builderOptions.setAnalysisEnabled(true)
const provider = OptionsProvider.build('<configs folder path>', builderOptions)
const analysis = provider.analyze('/myConfig/key2', ['grandparent'])
// {
//   origins: [{
//     path: '/myConfig/key2',
//     source: { lineage: ['grandparent', 'parent2', 'base2'], kind: 'feature' },
//     overridden: [{ lineage: ['grandparent', 'parent1'], kind: 'feature' }],
//   }],
// }
```

See [Analysis.md](../../docs/Analysis.md) for details about the result.

# Development

Use Node >= 22.

```Shell
corepack enable
yarn install
yarn build:debug
yarn build:ts
yarn test
```

## Testing

Run:

```shell
yarn build:debug
yarn build:ts
yarn test
```

## Formatting

JavaScript and TypeScript files use tabs as configured in [`.editorconfig`](../../.editorconfig).
Run oxfmt before committing:

```Shell
yarn fmt
```

To check for formatting issues, run:

```Shell
yarn fmt:check
```

## Linting

Run linting before committing:

```Shell
yarn lint:fix
```

To check for issues, run:

```Shell
yarn lint
```

## Rust Formatting

To automatically format the Rust code, run:

```shell
cargo fmt
```

## Benchmarking

Run:

```shell
rm -rf target config.*.node
yarn build
yarn build:ts
node benchmarks/get_all_options.mjs
```

## Publishing

Use CI.

Some notes:

See https://napi.rs/docs/introduction/getting-started#deep-dive \
See https://napi.rs/docs/introduction/simple-package

[homepage]: https://github.com/juharris/optify
