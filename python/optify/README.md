# Optify Rust Bindings for Python

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
