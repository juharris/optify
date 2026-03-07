# Optify Rust Bindings for Python

[![PyPI - Version](https://img.shields.io/pypi/v/optify?color=%23006dad)
](https://pypi.org/project/optify)

⚠️ Development in progress ⚠️\
APIs are not final and may change, for example, interfaces may be used in the future or method names may change.
This is just meant to be minimal to get started and help build a Python library.

See the [homepage] for details about how feature files are combined to build the options to process at runtime.

## Usage
```Python
import json
from optify import OptionsProvider

provider = OptionsProvider.build('path/to/configs')
config = provider.get_options_json('key', ['feature_A', 'feature_B'])
print(config)
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

See the [tests directory](./tests/) for more examples.

### Getting Typed Objects

The JSON output can be deserialized into typed objects using different libraries such as [dacite](https://github.com/konradhalas/dacite) (for dataclasses) or [Pydantic](https://docs.pydantic.dev/).

#### Using `dacite` with `dataclasses`

Install with `pip install dacite`

```Python
import json
from dataclasses import dataclass

import dacite

from optify import OptionsProvider


@dataclass
class Deeper:
    newValue: str
    num: int


@dataclass
class MyObject:
    key: str
    deeper: Deeper


@dataclass
class MyConfig:
    rootString: str
    myArray: list[str]
    myObject: MyObject


provider = OptionsProvider.build('path/to/configs')
options_json = provider.get_options_json('myConfig', ['feature_A', 'feature_B'])
config = dacite.from_dict(data_class=MyConfig, data=json.loads(options_json))

print(config.rootString)
print(config.myObject.deeper.newValue)
```

#### Using Pydantic

Install with `pip install pydantic`

```Python
import json

from pydantic import BaseModel

from optify import OptionsProvider


class Deeper(BaseModel):
    newValue: str
    num: int


class MyObject(BaseModel):
    key: str
    deeper: Deeper


class MyConfig(BaseModel):
    rootString: str
    myArray: list[str]
    myObject: MyObject


provider = OptionsProvider.build('path/to/configs')
options_json = provider.get_options_json('myConfig', ['feature_A', 'feature_B'])
config = MyConfig.model_validate_json(options_json)

print(config.rootString)
print(config.myObject.deeper.newValue)
```

### Watching for Changes

Use `OptionsWatcher` instead of `OptionsProvider` during development to have it automatically reload the options it stores when the files change:

```Python
from optify import OptionsWatcher

watcher = OptionsWatcher.build('path/to/configs')
```

## Development

### Setup

```shell
pyenv virtualenv optify-dev
pyenv local optify-dev
pyenv activate optify-dev

pip install -e '.[dev]'
```

### Build

```shell
maturin develop
```

### Tests

```shell
pytest
```

### Formatting
To automatically change the Rust code, run:
```shell
cargo fmt && cargo clippy --fix --allow-dirty --allow-staged
```

# Publishing
A GitHub Action will automatically publish new versions: https://github.com/juharris/optify/actions/workflows/python_publish.yml

[homepage]: https://github.com/juharris/optify
