# Optify Bindings for Node.js
[![NPM Version](https://img.shields.io/npm/v/%40optify%2Fconfig?color=bc3433)](https://www.npmjs.com/package/@optify/config)

# Usage

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
    "key": "val",
  },
  "rootString": "root string same"
}
```

Multiple directories can be used as well:

```TypeScript
import { OptionsProvider } from '@optify/config'

const provider = OptionsProvider.buildFromDirectories(['<configs folder path>', '<another folder path>'])
const options = JSON.parse(provider.getOptionsJson('myConfig', ['feature_A', 'feature_B']))
console.log(JSON.stringify(options, null, 2))
```

# Development

Use Node >= 22.

```Shell
yarn install
yarn build:debug
yarn test
```

## Testing

Run:
```shell
yarn build:debug
yarn test
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
