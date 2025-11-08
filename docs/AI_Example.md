# AI Example

This is an example of how an application to use an LLM can be configured to easily customize the system instructions and enabled tools using Optify.

See [Configurable Strings](./ConfigurableStrings.md) for more details on how configurable strings work.

Map a tool to `null` to disable it so that other configurations that override the base can disable tools.

Other properties such as the tool input schema can also be customized adjacent to the tool's description.

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
            "code_executor": {
                "description": {
                    "$type": "Optify.ConfigurableString",
                    "base": "Excutes code snippets in a secure environment."
                }
            },
            // Map to `null` to disable a tool.
            "math": null,
            "web_search": {
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
```