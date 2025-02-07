# Optify
Simplifies getting the right configuration options for a process or request using pre-loaded configurations from files (JSON, YAML, etc.) to manage options for experiments or flights.
Configurations for different experiments or feature flags are mergeable to support multiple experiments or feature flags for the same request.

[![Crates.io](https://img.shields.io/crates/v/optify)](https://crates.io/crates/optify)
[![Gem Version](https://badge.fury.io/rb/optify-config.svg?icon=si%3Arubygems&icon_color=%23ec3c3c)](https://badge.fury.io/rb/optify-config)

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

# File Formats
| Format | Good For | Caveats |
| --- | --- | --- |
| JSON  | **Long files** where built in parentheses checking is important because multiple people may edit the same file. This should normally be avoided by making smaller more granular files that are combined at runtime by giving each as a feature. | **No comments** because comments are not part of the JSON standard. Comments can be given as properties: `"_comment": "blah blah"`. Multiline strings or strings with escaping are head to read. |
| YAML | Short and simple files that are not edited often. Good support for strings with newlines. | Getting the indentation wrong can mean that properties are ignored. |
| JSON5 | Good mix of features from JSON and YAML. | Your IDE may require an extension to help with validation. |

Other types are supported as the [config](https://crates.io/crates/config) Rust crate is used to back this project, but those other types are not as well-known and not as nice for working with deep objects so they are not recommended.
In most cases, JSON should be preferred to help with some basic static structural validation at load time.
Standard JSON validation will easily catch issues such as a bad merge conflict resolution, whereas it is easy to have valid YAML, but would not work as expected at runtime because of incorrect indentation.

# Language Support

## .NET
See [github.com/juharris/dotnet-OptionsProvider](https://github.com/juharris/dotnet-OptionsProvider) for a similar library with dependency injection support.
Configurations are merged using typical .NET standards from `ConfigurationBuilder` when using `IConfiguration`, so lists are merged, unlike the behavior in this repository where lists are overwritten, which is easier to understand.

## Ruby
[![Gem Version](https://badge.fury.io/rb/optify-config.svg?icon=si%3Arubygems&icon_color=%23ec3c3c)](https://badge.fury.io/rb/optify-config)

See the [ruby/optify](./ruby/optify/) folder.
Built using the Rust implementation.

## Rust
[![Crates.io](https://img.shields.io/crates/v/optify)](https://crates.io/crates/optify)

See the [rust/optify](./rust/optify/) folder.
Not intended to be used by other Rust projects yet as it's mainly made to support building implementations for other languages such as Node.js, Python, and Ruby.
