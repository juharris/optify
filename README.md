<img src="./assets/logo.png" alt="logo" width="64" style="margin-left: 40%"/>

# Optify

Simplifies **configuration driven development**: getting the right configuration options for a process or request using pre-loaded configurations from files (JSON, YAML, etc.) to manage options for feature flags, experiments, or flights.
Configurations for different experiments or feature flags are mergeable to support multiple experiments or feature flags for the same request.

[![Crates.io](https://img.shields.io/crates/v/optify?logo=Rust)](https://crates.io/crates/optify)
[![Gem Version](https://badge.fury.io/rb/optify-config.svg?icon=si%3Arubygems&icon_color=%23ec3c3c)](https://badge.fury.io/rb/optify-config)
[![NPM Version](https://img.shields.io/npm/v/%40optify%2Fconfig?color=bc3433&logo=TypeScript)](https://www.npmjs.com/package/@optify/config)
[![NuGet Version](https://img.shields.io/nuget/v/OptionsProvider?logo=NuGet)](https://www.nuget.org/packages/OptionsProvider)
[![PyPI - Version](https://img.shields.io/pypi/v/optify?color=%23006dad&logo=Python)
](https://pypi.org/project/optify)
[![VS Code Extension](https://img.shields.io/vscode-marketplace/v/optify-config.optify?color=blue&logo=data:image/svg%2bxml;base64,PHN2ZyB2aWV3Qm94PSIwIDAgMTAwIDEwMCIgZmlsbD0ibm9uZSIgeG1sbnM9Imh0dHA6Ly93d3cudzMub3JnLzIwMDAvc3ZnIj4KPG1hc2sgaWQ9Im1hc2swIiBtYXNrLXR5cGU9ImFscGhhIiBtYXNrVW5pdHM9InVzZXJTcGFjZU9uVXNlIiB4PSIwIiB5PSIwIiB3aWR0aD0iMTAwIiBoZWlnaHQ9IjEwMCI+CjxwYXRoIGZpbGwtcnVsZT0iZXZlbm9kZCIgY2xpcC1ydWxlPSJldmVub2RkIiBkPSJNNzAuOTExOSA5OS4zMTcxQzcyLjQ4NjkgOTkuOTMwNyA3NC4yODI4IDk5Ljg5MTQgNzUuODcyNSA5OS4xMjY0TDk2LjQ2MDggODkuMjE5N0M5OC42MjQyIDg4LjE3ODcgMTAwIDg1Ljk4OTIgMTAwIDgzLjU4NzJWMTYuNDEzM0MxMDAgMTQuMDExMyA5OC42MjQzIDExLjgyMTggOTYuNDYwOSAxMC43ODA4TDc1Ljg3MjUgMC44NzM3NTZDNzMuNzg2MiAtMC4xMzAxMjkgNzEuMzQ0NiAwLjExNTc2IDY5LjUxMzUgMS40NDY5NUM2OS4yNTIgMS42MzcxMSA2OS4wMDI4IDEuODQ5NDMgNjguNzY5IDIuMDgzNDFMMjkuMzU1MSAzOC4wNDE1TDEyLjE4NzIgMjUuMDA5NkMxMC41ODkgMjMuNzk2NSA4LjM1MzYzIDIzLjg5NTkgNi44NjkzMyAyNS4yNDYxTDEuMzYzMDMgMzAuMjU0OUMtMC40NTI1NTIgMzEuOTA2NCAtMC40NTQ2MzMgMzQuNzYyNyAxLjM1ODUzIDM2LjQxN0wxNi4yNDcxIDUwLjAwMDFMMS4zNTg1MyA2My41ODMyQy0wLjQ1NDYzMyA2NS4yMzc0IC0wLjQ1MjU1MiA2OC4wOTM4IDEuMzYzMDMgNjkuNzQ1M0w2Ljg2OTMzIDc0Ljc1NDFDOC4zNTM2MyA3Ni4xMDQzIDEwLjU4OSA3Ni4yMDM3IDEyLjE4NzIgNzQuOTkwNUwyOS4zNTUxIDYxLjk1ODdMNjguNzY5IDk3LjkxNjdDNjkuMzkyNSA5OC41NDA2IDcwLjEyNDYgOTkuMDEwNCA3MC45MTE5IDk5LjMxNzFaTTc1LjAxNTIgMjcuMjk4OUw0NS4xMDkxIDUwLjAwMDFMNzUuMDE1MiA3Mi43MDEyVjI3LjI5ODlaIiBmaWxsPSJ3aGl0ZSIvPgo8L21hc2s+CjxnIG1hc2s9InVybCgjbWFzazApIj4KPHBhdGggZD0iTTk2LjQ2MTQgMTAuNzk2Mkw3NS44NTY5IDAuODc1NTQyQzczLjQ3MTkgLTAuMjcyNzczIDcwLjYyMTcgMC4yMTE2MTEgNjguNzUgMi4wODMzM0wxLjI5ODU4IDYzLjU4MzJDLTAuNTE1NjkzIDY1LjIzNzMgLTAuNTEzNjA3IDY4LjA5MzcgMS4zMDMwOCA2OS43NDUyTDYuODEyNzIgNzQuNzU0QzguMjk3OTMgNzYuMTA0MiAxMC41MzQ3IDc2LjIwMzYgMTIuMTMzOCA3NC45OTA1TDkzLjM2MDkgMTMuMzY5OUM5Ni4wODYgMTEuMzAyNiAxMDAgMTMuMjQ2MiAxMDAgMTYuNjY2N1YxNi40Mjc1QzEwMCAxNC4wMjY1IDk4LjYyNDYgMTEuODM3OCA5Ni40NjE0IDEwLjc5NjJaIiBmaWxsPSIjMDA2NUE5Ii8+CjxnIGZpbHRlcj0idXJsKCNmaWx0ZXIwX2QpIj4KPHBhdGggZD0iTTk2LjQ2MTQgODkuMjAzOEw3NS44NTY5IDk5LjEyNDVDNzMuNDcxOSAxMDAuMjczIDcwLjYyMTcgOTkuNzg4NCA2OC43NSA5Ny45MTY3TDEuMjk4NTggMzYuNDE2OUMtMC41MTU2OTMgMzQuNzYyNyAtMC41MTM2MDcgMzEuOTA2MyAxLjMwMzA4IDMwLjI1NDhMNi44MTI3MiAyNS4yNDZDOC4yOTc5MyAyMy44OTU4IDEwLjUzNDcgMjMuNzk2NCAxMi4xMzM4IDI1LjAwOTVMOTMuMzYwOSA4Ni42MzAxQzk2LjA4NiA4OC42OTc0IDEwMCA4Ni43NTM4IDEwMCA4My4zMzM0VjgzLjU3MjZDMTAwIDg1Ljk3MzUgOTguNjI0NiA4OC4xNjIyIDk2LjQ2MTQgODkuMjAzOFoiIGZpbGw9IiMwMDdBQ0MiLz4KPC9nPgo8ZyBmaWx0ZXI9InVybCgjZmlsdGVyMV9kKSI+CjxwYXRoIGQ9Ik03NS44NTc4IDk5LjEyNjNDNzMuNDcyMSAxMDAuMjc0IDcwLjYyMTkgOTkuNzg4NSA2OC43NSA5Ny45MTY2QzcxLjA1NjQgMTAwLjIyMyA3NSA5OC41ODk1IDc1IDk1LjMyNzhWNC42NzIxM0M3NSAxLjQxMDM5IDcxLjA1NjQgLTAuMjIzMTA2IDY4Ljc1IDIuMDgzMjlDNzAuNjIxOSAwLjIxMTQwMiA3My40NzIxIC0wLjI3MzY2NiA3NS44NTc4IDAuODczNjMzTDk2LjQ1ODcgMTAuNzgwN0M5OC42MjM0IDExLjgyMTcgMTAwIDE0LjAxMTIgMTAwIDE2LjQxMzJWODMuNTg3MUMxMDAgODUuOTg5MSA5OC42MjM0IDg4LjE3ODYgOTYuNDU4NiA4OS4yMTk2TDc1Ljg1NzggOTkuMTI2M1oiIGZpbGw9IiMxRjlDRjAiLz4KPC9nPgo8ZyBzdHlsZT0ibWl4LWJsZW5kLW1vZGU6b3ZlcmxheSIgb3BhY2l0eT0iMC4yNSI+CjxwYXRoIGZpbGwtcnVsZT0iZXZlbm9kZCIgY2xpcC1ydWxlPSJldmVub2RkIiBkPSJNNzAuODUxMSA5OS4zMTcxQzcyLjQyNjEgOTkuOTMwNiA3NC4yMjIxIDk5Ljg5MTMgNzUuODExNyA5OS4xMjY0TDk2LjQgODkuMjE5N0M5OC41NjM0IDg4LjE3ODcgOTkuOTM5MiA4NS45ODkyIDk5LjkzOTIgODMuNTg3MVYxNi40MTMzQzk5LjkzOTIgMTQuMDExMiA5OC41NjM1IDExLjgyMTcgOTYuNDAwMSAxMC43ODA3TDc1LjgxMTcgMC44NzM2OTVDNzMuNzI1NSAtMC4xMzAxOSA3MS4yODM4IDAuMTE1Njk5IDY5LjQ1MjcgMS40NDY4OEM2OS4xOTEyIDEuNjM3MDUgNjguOTQyIDEuODQ5MzcgNjguNzA4MiAyLjA4MzM1TDI5LjI5NDMgMzguMDQxNEwxMi4xMjY0IDI1LjAwOTZDMTAuNTI4MyAyMy43OTY0IDguMjkyODUgMjMuODk1OSA2LjgwODU1IDI1LjI0NkwxLjMwMjI1IDMwLjI1NDhDLTAuNTEzMzM0IDMxLjkwNjQgLTAuNTE1NDE1IDM0Ljc2MjcgMS4yOTc3NSAzNi40MTY5TDE2LjE4NjMgNTBMMS4yOTc3NSA2My41ODMyQy0wLjUxNTQxNSA2NS4yMzc0IC0wLjUxMzMzNCA2OC4wOTM3IDEuMzAyMjUgNjkuNzQ1Mkw2LjgwODU1IDc0Ljc1NEM4LjI5Mjg1IDc2LjEwNDIgMTAuNTI4MyA3Ni4yMDM2IDEyLjEyNjQgNzQuOTkwNUwyOS4yOTQzIDYxLjk1ODZMNjguNzA4MiA5Ny45MTY3QzY5LjMzMTcgOTguNTQwNSA3MC4wNjM4IDk5LjAxMDQgNzAuODUxMSA5OS4zMTcxWk03NC45NTQ0IDI3LjI5ODlMNDUuMDQ4MyA1MEw3NC45NTQ0IDcyLjcwMTJWMjcuMjk4OVoiIGZpbGw9InVybCgjcGFpbnQwX2xpbmVhcikiLz4KPC9nPgo8L2c+CjxkZWZzPgo8ZmlsdGVyIGlkPSJmaWx0ZXIwX2QiIHg9Ii04LjM5NDExIiB5PSIxNS44MjkxIiB3aWR0aD0iMTE2LjcyNyIgaGVpZ2h0PSI5Mi4yNDU2IiBmaWx0ZXJVbml0cz0idXNlclNwYWNlT25Vc2UiIGNvbG9yLWludGVycG9sYXRpb24tZmlsdGVycz0ic1JHQiI+CjxmZUZsb29kIGZsb29kLW9wYWNpdHk9IjAiIHJlc3VsdD0iQmFja2dyb3VuZEltYWdlRml4Ii8+CjxmZUNvbG9yTWF0cml4IGluPSJTb3VyY2VBbHBoYSIgdHlwZT0ibWF0cml4IiB2YWx1ZXM9IjAgMCAwIDAgMCAwIDAgMCAwIDAgMCAwIDAgMCAwIDAgMCAwIDEyNyAwIi8+CjxmZU9mZnNldC8+CjxmZUdhdXNzaWFuQmx1ciBzdGREZXZpYXRpb249IjQuMTY2NjciLz4KPGZlQ29sb3JNYXRyaXggdHlwZT0ibWF0cml4IiB2YWx1ZXM9IjAgMCAwIDAgMCAwIDAgMCAwIDAgMCAwIDAgMCAwIDAgMCAwIDAuMjUgMCIvPgo8ZmVCbGVuZCBtb2RlPSJvdmVybGF5IiBpbjI9IkJhY2tncm91bmRJbWFnZUZpeCIgcmVzdWx0PSJlZmZlY3QxX2Ryb3BTaGFkb3ciLz4KPGZlQmxlbmQgbW9kZT0ibm9ybWFsIiBpbj0iU291cmNlR3JhcGhpYyIgaW4yPSJlZmZlY3QxX2Ryb3BTaGFkb3ciIHJlc3VsdD0ic2hhcGUiLz4KPC9maWx0ZXI+CjxmaWx0ZXIgaWQ9ImZpbHRlcjFfZCIgeD0iNjAuNDE2NyIgeT0iLTguMDc1NTgiIHdpZHRoPSI0Ny45MTY3IiBoZWlnaHQ9IjExNi4xNTEiIGZpbHRlclVuaXRzPSJ1c2VyU3BhY2VPblVzZSIgY29sb3ItaW50ZXJwb2xhdGlvbi1maWx0ZXJzPSJzUkdCIj4KPGZlRmxvb2QgZmxvb2Qtb3BhY2l0eT0iMCIgcmVzdWx0PSJCYWNrZ3JvdW5kSW1hZ2VGaXgiLz4KPGZlQ29sb3JNYXRyaXggaW49IlNvdXJjZUFscGhhIiB0eXBlPSJtYXRyaXgiIHZhbHVlcz0iMCAwIDAgMCAwIDAgMCAwIDAgMCAwIDAgMCAwIDAgMCAwIDAgMTI3IDAiLz4KPGZlT2Zmc2V0Lz4KPGZlR2F1c3NpYW5CbHVyIHN0ZERldmlhdGlvbj0iNC4xNjY2NyIvPgo8ZmVDb2xvck1hdHJpeCB0eXBlPSJtYXRyaXgiIHZhbHVlcz0iMCAwIDAgMCAwIDAgMCAwIDAgMCAwIDAgMCAwIDAgMCAwIDAgMC4yNSAwIi8+CjxmZUJsZW5kIG1vZGU9Im92ZXJsYXkiIGluMj0iQmFja2dyb3VuZEltYWdlRml4IiByZXN1bHQ9ImVmZmVjdDFfZHJvcFNoYWRvdyIvPgo8ZmVCbGVuZCBtb2RlPSJub3JtYWwiIGluPSJTb3VyY2VHcmFwaGljIiBpbjI9ImVmZmVjdDFfZHJvcFNoYWRvdyIgcmVzdWx0PSJzaGFwZSIvPgo8L2ZpbHRlcj4KPGxpbmVhckdyYWRpZW50IGlkPSJwYWludDBfbGluZWFyIiB4MT0iNDkuOTM5MiIgeTE9IjAuMjU3ODEyIiB4Mj0iNDkuOTM5MiIgeTI9Ijk5Ljc0MjMiIGdyYWRpZW50VW5pdHM9InVzZXJTcGFjZU9uVXNlIj4KPHN0b3Agc3RvcC1jb2xvcj0id2hpdGUiLz4KPHN0b3Agb2Zmc2V0PSIxIiBzdG9wLWNvbG9yPSJ3aGl0ZSIgc3RvcC1vcGFjaXR5PSIwIi8+CjwvbGluZWFyR3JhZGllbnQ+CjwvZGVmcz4KPC9zdmc+Cg==)][vsc-extension]

> The configuration should declare **what** to do, but **not how** to do it.

This project helps improve the scalability and maintainability of code.
We should determine the right configuration for a request or process when it starts by passing the enabled features to an `OptionsProvider`.
The returned options would be used throughout the request or process to change business logic.
Supporting deep configurations with many types of properties instead of simple enabled/disabled feature flags is important to help avoid conditional statements (`if` statements) and thus improve the scalability of our code and make it easier to maintain our code as explained in [this article][cond-article].

Instead of working with feature flags:
```Python
if Settings.is_feature_A_enabled:
    handle_A(params)
elif Settings.is_feature_B_enabled:
    handle_B(params)
elif Settings.is_feature_C_enabled:
    handle_C(params)
else:
    raise FeatureError
```

You can ensure your code effortlessly scales to new scenarios working with a list of enabled features:
```Python
handler_options = provider.get_options('handler', features)
handler = handlers[handler_options.handler_name]
handler.handle(params)
```

It's fine to use systems that only support enabled/disabled feature flags, but we'll inevitably need to support more sophisticated configurations than on/off or `true`/`false`.
This project facilitates using deep configurations to be the backing for simple feature flags, thus keeping API contracts clean and facilitating the refactoring of code that uses the configurations.
Allowing clients to know about and pass in deep configurations for specific components is hard to maintain and makes it difficult to change the structure of the configurations.
Instead we can convert each enabled flag to a string and then build the merged configuration for the list of string "features".

See [tests](./tests/) for examples and tests for different variations of this paradigm for managing options.

Core Features:
* **Each *feature flag* can be represented by a JSON or YAML file** which contains options to override default configuration values when processing feature names or experiment names in a request.
* Each file is a granular **partial** representation of the overall configuration.
  Features are intended to be combined to build the final configuration.
* **Multiple features** can be enabled for the same request to support overlapping or intersecting experiments which are ideally mutually exclusive. Dictionaries are merged with the last feature taking precedence. Key values, including lists are overwritten.
* Supports clear file names and **aliases** for feature names.
* Feature names are **case insensitive**. `"feature_A"` and `"FeaTurE_a"` are the same.
* **Reads files in parallel** when loading your configurations.
* **Caching**: Configurations for a key and enabled features are cached to avoid rebuilding objects.
  Caching is only implemented in Ruby for now.
* Files are only read once when the `OptionsProvider` is built.
  This should be done when your application starts to ensure that files are only read once and issues are found early.
* **Inheritance**: Features can import or depend on other features.
  This keeps your list of enabled features smaller at runtime by allowing you to group related configurations while keeping most files small, focused, and like granular building blocks.

# Ethos
The main idea behind Optify is **configuration driven development**.

> The configuration should declare **what** to do, but **not how** to do it.

Engineers are responsible for writing robust business logic in code to interpret the configuration and execute the desired behavior.
With clear configurations, it's easy to change or refactor the implementation of the business logic because the configuration declares the desired behavior.
Team members that are not familiar with the details of the business logic or language of the code - such as perhaps, new team members, managers, the product team, data scientists, etc. - can focus on working with and understanding configuration files instead of diving deep into the code which involves understand that particular programming language.
This enables more team members to be involved in experimentation without worrying about the details of the implementation.
A typical example of this in programming is how SQL tells the database the desired output and structure, but is usually agnostic about how to execute a query and what data structures or optimizations can be used to implement the search.

This also makes many changes easier to make and review because we don't have to scrutinize specific code as much.
For example, many changes will be less intimidating to make and review because they're just adding or modifying a YAML file instead of changing Ruby files or adding custom conditional logic.

This project encourages using **features backed by configurations in files with your source code** because that's the most clear way for developers to see what values are supported for different configurable options.
The .NET version of this library happens to support configurations in the cloud because it uses standard .NET interfaces for configuration providers,
but this is not the main focus of these projects because configurations in the cloud are hard to maintain and easy to break in a backwards incompatible way.
Configurations in the cloud are fine for temporary experiments, but make the daily development experience less stable and unclear because it's not obvious what values are possible for different options which make refactoring difficult.
The main point is to keep the configurations private and internal to your codebase while feature flags names are part of your external API.

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

Create a **new** folder for configurations files, for this example, we'll call it `configurations` and add some files to it.
All `*.json`, `*.yaml`, and `*.yml` files in `configurations` and any of its subdirectories will be loaded into memory.
Markdown files (ending in `.md`) are ignored.

Create `configurations/feature_A.json`:
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

Create `configurations/feature_B/initial.yaml`:
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

You'll load the `configurations` folder using an `OptionsProviderBuilder` and then get an `OptionsProvider` from that builder.
Some languages also have an `OptionsWatcherBuilder` which can be used to watch for changes in the configuration files and automatically reload changes into the `OptionsProvider`.
The exact class names and methods may vary slightly depending on the language you are using.
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

# Configuration Classes
There are several aspects to consider when defining the classes that are used to represent the built configuration of combined features.

## Immutability
Classes will often be shared and passed around throughout your codebase.
In particular, configuration classes can be cached, so they can be shared across requests so no request should change the state of a configuration class.
Classes should be reusable without side effects.

## Documentation
Having clear documentation for classes and properties helps developers understand how to use the class and its properties when reading the codebase and writing business logic.

## Scope of a Configuration Class
Each configuration class should be scoped to a specific component or aspect of the system.
Consider the [Single Responsibility Principle](https://wikipedia.org/wiki/Single-responsibility_principle) and [Law of Demeter](https://wikipedia.org/wiki/Law_of_Demeter).
Using minimal classes helps developers understand the logic because as soon as they see the class, they know what behavior might change.
If a class is too large with too many properties, then it's difficult to understand what properties may be used by the component and the behavior that might change.
Large classes that are passed around too much quickly evolve to [God Objects](https://wikipedia.org/wiki/God_object) as they become a convenient place to dump new properties.
Passing around these God Objects throughout your codebase makes it difficult to understand what a component may do because that component has convenient access to too many properties.

## Nullability
Summary: **no guidance** because it's a case by case decision.

In theory, every property should be nullable because it's possible to use a combination of features that omits a property or sets that property to `null`.
Remember, any combination of features is permitted.
In practice, we don't have to worry about `null`s much if we make our files properly and use the configuration in the right places.
If someone sends a request that uses a strange combination of features, then it's up to them to understand the consequences and test appropriately.
"Garbage in; garbage out".

Eventually we can facilitate validation after a configuration is built.

# File Formats
| Format | Good For | Caveats |
| --- | --- | --- |
| JSON  | **Long files** where built in parentheses checking is important because multiple people may edit the same file. This should normally be avoided by making smaller more granular files that are combined at runtime by giving each as a feature. | **No comments** because comments are not part of the JSON standard. Comments can be given as properties: `"_comment": "blah blah"`.<br/> Multiline strings or strings with escaping are hard to read. |
| YAML | Short and simple files that are not edited often.<br/> Good support for strings with newlines. <br/> Since JSON is valid YAML, JSON with comments can be used. | Getting the indentation wrong can mean that properties are ignored.<br/> If you try to use JSON, your editor may automatically convert the JSON to simpler YAML depending on your settings or your project might have certain style checks enabled for YAML files. |
| JSON5 | Good mix of features from JSON and YAML. | Your IDE may require an extension to help with validation. |

Other types are supported as the [config](https://crates.io/crates/config) Rust crate is used to back this project, but those other types are not as well-known and not as nice for working with deep objects so they are not recommended.
In most cases, JSON should be preferred to help with some basic static structural validation at load time.
Standard JSON validation will easily catch issues such as a bad merge conflict resolution, whereas it is easy to have valid YAML, but would not work as expected at runtime because of incorrect indentation.

## Schema Help

In VS Code and editors derived from VS Code,
there are a few ways to get hints and see documentation for the properties such as `"metadata"`, `"imports"`, and `"options"`.

It is also recommended to install the [extension][vsc-extension] to get help with the JSON and YAML files.

### Recommended Extensions

In `./vscode/extensions.json`, add:
```JSON
{
    "recommendations": [
        "optify-config.optify",
        "redhat.vscode-yaml"
    ]
}
```

Then install those recommended extensions.

### VS Code Settings

To get help with many files, add the following to your `.vscode/settings.json` file:

```JSON
{
    "json.validate.enable": true,
    "json.schemaDownload.enable": true,
    "json.schemas": [
        {
            "fileMatch": [
                "path/**/configs/**/*.json"
            ],
            "url": "https://raw.githubusercontent.com/juharris/optify/refs/heads/main/schemas/feature_file.json"
        }
    ],
    "yaml.schemas": {
        "https://raw.githubusercontent.com/juharris/optify/refs/heads/main/schemas/feature_file.json": [
            "path/to/configs/**/*.{yaml,yml}"
        ]
    }
}
```

### Directly in JSON

To only enable help in one JSON file:

```JSON
{
    "$schema": "https://raw.githubusercontent.com/juharris/optify/refs/heads/main/schemas/feature_file.json",
    "metadata": {
        ...
    },
    "options": {
        ...
    }
} 
```

### Directly in YAML

To only enable help in one YAML file:

```YAML
# yaml-language-server: $schema=https://raw.githubusercontent.com/juharris/optify/refs/heads/main/schemas/feature_file.json
metadata:
    ...
options:
    ...
```


# Inheritance
Feature files can list ordered dependencies to declare other files to eagerly import.

This allows grouping related configurations while keeping most files small, focused, and like granular building blocks.
This also helps keep lists of enabled features smaller at runtime for typical feature that are used together.

Imports are resolved at build time, when `OptionsProviderBuilder::build` is called so that getting to right configuration from an `OptionsProvider` is as fast as possible, but sacrificing some extra memory overhead to store redundant options because the options will also be stored in each parent.

Each import must be a canonical feature name, i.e., derived from path to a file in order to keep dependencies clear and to help with navigating through files.

For example, if we have:

`configurations/feature_A.json`:
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

`configurations/feature_B.yaml`:
```yaml
options:
    myConfig:
        myArray:
            - "feature B item 1"
        myObject:
            one: 11
            three: 33
```

And `configurations/feature_C.yaml`:
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

See [tests](./tests/) for more examples.

# Language Support
This repository is mainly for the Rust implementation and that implementation that build off of that Rust implementations.
Below are implementations for a few languages.

## .NET
[![NuGet Version](https://img.shields.io/nuget/v/OptionsProvider?logo=NuGet)](https://www.nuget.org/packages/OptionsProvider)

See [github.com/juharris/dotnet-OptionsProvider][dotnet-OptionsProvider] for a similar library with dependency injection support.
Configurations are merged using typical .NET standards from `ConfigurationBuilder` when using `IConfiguration`, so lists are merged, unlike the behavior in this repository where lists are overwritten, which is easier to understand.

## Node.js
[![NPM Version](https://img.shields.io/npm/v/%40optify%2Fconfig?color=bc3433&logo=TypeScript)](https://www.npmjs.com/package/@optify/config)

See the [js/optify-config](./js/optify-config/) folder.
Built using the Rust implementation.

## Python
[![PyPI - Version](https://img.shields.io/pypi/v/optify?color=%23006dad&logo=Python)
](https://pypi.org/project/optify)

See the [python/optify](./python/optify/) folder.
Built using the Rust implementation.

## Ruby
[![Gem Version](https://badge.fury.io/rb/optify-config.svg?icon=si%3Arubygems&icon_color=%23ec3c3c)](https://badge.fury.io/rb/optify-config)

See the [ruby/optify](./ruby/optify/) folder.
Built using the Rust implementation.

## Rust
[![Crates.io](https://img.shields.io/crates/v/optify?logo=Rust)](https://crates.io/crates/optify)

See the [rust/optify](./rust/optify/) folder.
Not intended to be used by other Rust projects yet as it's mainly made to support building implementations for other languages such as Node.js, Python, and Ruby.
The API may change slightly until version 1.0 is released.

[cond-article]: https://medium.com/@justindharris/conditioning-code-craft-clear-and-concise-conditional-code-f4f328c43c2b
[dotnet-OptionsProvider]: https://github.com/juharris/dotnet-OptionsProvider
[vsc-extension]: https://marketplace.visualstudio.com/items?itemName=optify-config.optify
