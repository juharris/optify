# Optify Rust Bindings for Python

[![PyPI - Version](https://img.shields.io/pypi/v/optify?color=%23006dad)
](https://pypi.org/project/optify)

⚠️ Development in progress ⚠️\
APIs are not final and will change, for example, interfaces with be used.
This is just meant to be minimal to get started and help build a Python library.

See the [homepage] for details.

## Usage
See the [tests directory](./tests/) for examples.

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
