# Optify
Simplifies getting the right configuration options for a process using pre-loaded configurations from files (JSON, YAML, etc.) to manage options for experiments or flights.

See [tests](./tests/) for examples and tests for different implementations of this format for managing options.

Core Features:
* **Each *feature flag* can be represented by a JSON or YAML file** which contains options to override default configuration values when processing feature names or experiment names in a request.
* **Multiple features** can be enabled for the same request to support overlapping or intersecting experiments which are ideally mutually exclusive. Dictionaries are merged with the last feature taking precedence. Key values, including lists are overwritten.
* Supports clear file names and **aliases** for feature names.
* **Caching**: (coming soon)

# Merging Configuration Files
When merging configurations for features, objects are merged with the last feature taking precedence.
Key values, including lists are overwritten.

More details and examples to come soon.
For now, the details for the .NET implementation are good enough, except that the .NET implementation merges lists, while the implementations in this repository overwrite lists.

# Language Support

## .NET
See [github.com/juharris/dotnet-OptionsProvider](https://github.com/juharris/dotnet-OptionsProvider) for a similar library with dependency injection support.
Configurations are merged using typical .NET standards from `ConfigurationBuilder` when using `IConfiguration`, so lists are merged, unlike the behavior in this repository where lists are overwritten, which is easier to understand.

## Ruby
See the [ruby/optify](./ruby/optify/) folder.
Built using the Rust implementation.

## Rust
See the [rust/optify](./rust/optify/) folder.
Not intended to be used by other Rust projects yet as it's mainly made to support building implementations for other languages such as Node.js, Python, and Ruby.
