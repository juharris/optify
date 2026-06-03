# AI Example

This is an example of how an application to use an LLM can be configured to easily customize the system instructions and enabled tools using Optify.

See [Configurable Lists](./ConfigurableLists.md) and [Configurable Strings](./ConfigurableStrings.md) for more details on how to dynamically build lists and strings.

Other properties such as the tool input schema can also be customized adjacent to the tool's description.

Here is a simple configuration.
Imagine it was the result of merging multiple configuration files based on several features.

```JSON
{
    "options": {
        "model": {
            "name": "GPT-5",
            "parameters": {
                "max_tokens": 1500,
                "temperature": 0.7
            }
        },
        "system_instructions": {
            "$type": "Optify.ConfigurableString",
            "base": {
                "file": "templates/system_instructions.liquid"
            },
            "arguments": {
                "product_name": "MyApp",
                "personality": {
                    "file": "shared/personality.txt"
                }
            }
        },
        "tools": {
            "$type": "Optify.ConfigurableList",
            "code_executor": {
                "$value":{
                    "name": "code_executor",
                    "description": {
                        "$type": "Optify.ConfigurableString",
                        "base": "Excutes code snippets in a secure environment."
                    }
                }
            },
            // Map to `null` to disable a tool.
            "math": null,
            "web_search": {
                "$value":{
                    "name": "web_search",
                    "description": {
                        "$type": "Optify.ConfigurableString",
                        "base": {
                            "liquid": "Search the web using {{ provider }}."
                        },
                        "arguments": {
                            "provider": "Bing Search"
                        }
                    }
                }
            }
        }
    }
}
```

Here is how your code would see the configuration when using an `OptionsProvider` with configurable values enabled:

```JSON
{
    "options": {
        "model": {
            "name": "GPT-5",
            "parameters": {
                "max_tokens": 1500,
                "temperature": 0.7
            }
        },
        "system_instructions": "You are a helpful assistant for MyApp.\n\nYou are nice.",
        "tools": [
            {
                "name": "code_executor",
                "description": "Excutes code snippets in a secure environment."
            },
            {
                "name": "web_search",
                "description": "Search the web using Bing Search."
            }
        ]
    }
}
```