# Optify
Simplifies getting the right configuration options for a process using pre-loaded configurations from files (JSON, YAML, etc.) to manage options for experiments or flights.

See [tests](./tests/) for examples and tests for different implementations of this format for managing options.

Core Features:
* **Each *feature flag* can be represented by a JSON or YAML file** which contains options to override default configuration values when processing feature names or experiment names in a request.
* **Multiple features** can be enabled for the same request to support overlapping or intersecting experiments which are ideally mutually exclusive. Dictionaries are merged with the last feature taking precedence. Key values, including lists are overwritten.
* Supports clear file names and **aliases** for feature names.
* **Caching**: (coming soon)

# .NET
See [github.com/juharris/dotnet-OptionsProvider](https://github.com/juharris/dotnet-OptionsProvider) for an equivalent library with dependency injection support.

# Ruby
Coming soon and it will be built using the Rust implementation.

# Rust
See the [rust](./rust/) folder.
Not intended to be used for other Rust projects as it's mainly made to support building implementations for other languages such as Node.js, Python, and Ruby.
