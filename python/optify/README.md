# Optify Rust Bindings for Python

⚠️ Development in progress ⚠️\
APIs are not final and will change, for example, interfaces with be used.
This is just meant to be minimal to get started and help build a Python library.

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

# Publishing
A GitHub Action will automatically publish new versions: https://github.com/juharris/optify/actions/workflows/python_publish.yml
