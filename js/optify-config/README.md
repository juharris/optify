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
})

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

## Benchmarking

Run:
```shell
rm -rf target config.*.node
yarn build
yarn build:ts
node benchmarks/get_all_options.mjs
```

## Formatting

To automatically change the Rust code, run:
```shell
cargo fmt
```

## Publishing

Use CI.

Some notes:

See https://napi.rs/docs/introduction/getting-started#deep-dive \
See https://napi.rs/docs/introduction/simple-package

[homepage]: https://github.com/juharris/optify
