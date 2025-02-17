# Optify
Simplifies getting the right configuration options for a process or request using pre-loaded configurations from files (JSON, YAML, etc.) to manage options for experiments or flights.
Configurations for different experiments or feature flags are mergeable to support multiple experiments or feature flags for the same request.

![NuGet Version](https://img.shields.io/nuget/v/OptionsProvider)
[![Crates.io](https://img.shields.io/crates/v/optify)](https://crates.io/crates/optify)
[![Gem Version](https://badge.fury.io/rb/optify-config.svg?icon=si%3Arubygems&icon_color=%23ec3c3c)](https://badge.fury.io/rb/optify-config)

This project helps code scale better and be easier to maintain.
We should determine the right configuration for a request or process when it starts by passing the enabled features to an `OptionsProvider`.
The returned options would be used throughout the request or process to change business logic.
Supporting deep configurations with many types of properties instead of simple enabled/disabled feature flags is important to help avoid conditional statements (`if` statements) and thus help code scale and be more maintainable as explained in [this article][cond-blog].

It's fine to use systems that support enabled/disabled feature flags, but we'll inevitably need to support more sophisticated configurations.
This project facilitates using deep configurations to be the backing for simple feature flags, thus keeping API contracts clean and facilitating the refactoring of code that uses the configurations.
Allowing clients to know about and pass in deep configurations for specific components is hard to maintain and makes it difficult to change the structure of the configurations.

See [tests](./tests/) for examples and tests for different variations of this paradigm for managing options.

Core Features:
* **Each *feature flag* can be represented by a JSON or YAML file** which contains options to override default configuration values when processing feature names or experiment names in a request.
* Each file is a granular **partial** representation of the overall configuration.
  Features are intended to be combined to build the final configuration.
* **Multiple features** can be enabled for the same request to support overlapping or intersecting experiments which are ideally mutually exclusive. Dictionaries are merged with the last feature taking precedence. Key values, including lists are overwritten.
* Supports clear file names and **aliases** for feature names.
* **Caching**: Configurations for a key and enabled features are cached to avoid rebuilding objects.
  Caching is only implemented in Ruby for now.
* Files are only read once when the `OptionsProvider` is built.
  This should be done when your application starts to ensure that files are only read once and issues are found early.
* **Inheritance**: Features can import or depend on other features.
  This keeps your list of enabled features smaller at runtime by allowing you to group related configurations while keeping most files small, focused, and like granular building blocks.

# Merging Configuration Files
When merging configurations for features, objects are merged with the last feature taking precedence.
Key values, including lists are overwritten.

As explained below, the .NET version works a little differently that the versions in this repository which are backed by the Rust implementation.

## Example
Suppose you have a class that you want to use to configure your logic at runtime:
```csharp
class MyConfiguration
{
    string[]? myArray
    MyObject? myObject
}
```

Now you want to start experimenting with different values deep within `MyConfiguration`.

Create a **new** folder for configurations files, for this example, we'll call it `Configurations` and add some files to it.
All `*.json`, `*.yaml`, and `*.yml` files in `Configurations` and any of its subdirectories will be loaded into memory.
Markdown files (ending in `.md`) are ignored.

Create `Configurations/feature_A.json`:
```json
{
    "metadata": {
        "aliases": [ "A" ],
        "owners": "a-team@company.com"
    },
    "options": {
        "myConfig": {
            "myArray": [
                "example item 1"
            ],
            "myObject": {
                "one": 1,
                "two": 2
            }
        }
    }
}
```

Create `Configurations/feature_B/initial.yaml`:
```yaml
metadata:
    aliases:
        - "B"
    owners: "team-b@company.com"
options:
    myConfig:
        myArray:
            - "different item 1"
            - "item 2"
        myObject:
            one: 11
            three: 33
```

You'll load the `Configurations` folder using an `OptionsProviderBuild` and then get an `OptionsProvider` from that builder.
How that works depend on the language you are using.
See below for links to implementations in different languages.

The result of using features: `["A", "B"]` will be:
```json
{
  // "myArray" from B overrides "myArray" from A.
  "myArray": [
    "different item 1",
    "item 2"
  ],
  // Values in "myObject" in B override values in "myObject" in A.
  "myObject": {
    "one": 11,
    "two": 2,
    "three": 3
  }
}
```

The result of using features: `["B", "A"]` will be:
```json
{
  // "myArray" from A overrides "myArray" from B.
  "myArray": [
    "example item 1"
  ],
  // Values in "myObject" in A override values in "myObject" in B.
  "myObject": {
    "one": 1,
    "two": 2,
    "three": 3
  }
}
```

# File Formats
| Format | Good For | Caveats |
| --- | --- | --- |
| JSON  | **Long files** where built in parentheses checking is important because multiple people may edit the same file. This should normally be avoided by making smaller more granular files that are combined at runtime by giving each as a feature. | **No comments** because comments are not part of the JSON standard. Comments can be given as properties: `"_comment": "blah blah"`.<br/> Multiline strings or strings with escaping are hard to read. |
| YAML | Short and simple files that are not edited often.<br/> Good support for strings with newlines. <br/> Since JSON is valid YAML, JSON with comments can be used. | Getting the indentation wrong can mean that properties are ignored.<br/> If you try to use JSON, your editor may automatically convert the JSON to simpler YAML depending on your settings or your project might have certain style checks enabled for YAML files. |
| JSON5 | Good mix of features from JSON and YAML. | Your IDE may require an extension to help with validation. |

Other types are supported as the [config](https://crates.io/crates/config) Rust crate is used to back this project, but those other types are not as well-known and not as nice for working with deep objects so they are not recommended.
In most cases, JSON should be preferred to help with some basic static structural validation at load time.
Standard JSON validation will easily catch issues such as a bad merge conflict resolution, whereas it is easy to have valid YAML, but would not work as expected at runtime because of incorrect indentation.

# Inheritance
Feature files can list ordered dependencies to declare other files to eagerly import.

This allows grouping related configurations while keeping most files small, focused, and like granular building blocks.
This also helps keep lists of enabled features smaller at runtime for typical feature that are used together.

Imports are resolved at build time, when `OptionsProviderBuilder::build` is called so that getting to right configuration from an `OptionsProvider` is as fast as possible, but sacrificing some extra memory overhead to store redundant options in each parent.

Each import must be a canonical feature name, i.e., derived from path to a file in order to keep dependencies clear and to help with navigating through files.

For example, if we have:

`Configurations/feature_A.json`:
```json
{
    "options": {
        "myConfig": {
            "myArray": [
                "example item 1",
                "example item 2"
            ],
            "myObject": {
                "one": 1,
                "two": 2
            }
        }
    }
}
```

`Configurations/feature_B.yaml`:
```yaml
options:
    myConfig:
        myArray:
            - "feature B item 1"
        myObject:
            one: 11
            three: 33
```

And `Configurations/feature_C.yaml`:
```yaml
imports:
    - "feature_A"
    - "feature_B"
options:
    myConfig:
        myObject:
            three: 3
```

The resulting options for `feature_C` will be as if we included the features in the order `["feature_A", "feature_B", "feature_C"]`:
```json
{
    "myConfig":{
        // The values from feature_B as feature_B is listed after feature_A so it overrides it.
        "myArray": [
            "feature B item 1"
        ],
        // Applying feature_A, then feature_B, then feature_C.
        "myObject": {
            "one": 11,
            "two": 2,
            "three": 3
        }
    }
}
```

There is no limit on the depth for imports; imports can import other features that import other features.

Circular imports are not allowed and will result in an error at build time.

See [tests](./tests/) more examples.

# Language Support
This repository is mainly for the Rust implementation and that implementation that build off of that Rust implementations.
Below are implementations for a few languages.

## .NET
![NuGet Version](https://img.shields.io/nuget/v/OptionsProvider)

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
The API may change slightly until version 1.0 is released.

[cond-blog]: https://medium.com/@justindharris/conditioning-code-craft-clear-and-concise-conditional-code-f4f328c43c2b
