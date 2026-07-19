# AI Example

This is an example of how an application to use an LLM can be configured to easily customize the system instructions and enabled tools using Optify.

See [Configurable Lists](./ConfigurableLists.md) and [Configurable Strings](./ConfigurableStrings.md) for more details on how to dynamically build lists and strings.

Other properties such as the tool input schema can also be customized adjacent to the tool's description.

Here is a simple configuration.
Imagine it was the result of merging multiple configuration files based on several features.

```JSONC
{
    "options": {
        "model": {
            "name": "GPT-5",
            "parameters": {
                "max_tokens": 1500,
                "temperature": 0.7
            }
        },
        "chat": {
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
                    "$value": {
                        "name": "code_executor",
                        "description": {
                            "$type": "Optify.ConfigurableString",
                            "base": "Executes code snippets in a secure environment."
                        }
                    }
                },
                // Map to `null` to disable a tool.
                "math": null,
                "web_search": {
                    "$value": {
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
        "chat": {
            "system_instructions": "You are a helpful assistant for MyApp.\n\nYou are nice.",
            "tools": [
                {
                    "name": "code_executor",
                    "description": "Executes code snippets in a secure environment."
                },
                {
                    "name": "web_search",
                    "description": "Search the web using Bing Search."
                }
            ]
        }
    }
}
```

## Schema

A custom schema at `.optify/schema.json` can specify the value type for `tools` while continuing to use Optify's standard definitions:

```JSON
{
    "$schema": "https://json-schema.org/draft/2020-12/schema",
    "type": "object",
    "additionalProperties": false,
    "minProperties": 1,
    "properties": {
        "conditions": {
            "$ref": "https://raw.githubusercontent.com/juharris/optify/refs/heads/main/schemas/feature_file.json#/definitions/conditions"
        },
        "imports": {
            "$ref": "https://raw.githubusercontent.com/juharris/optify/refs/heads/main/schemas/feature_file.json#/definitions/imports"
        },
        "metadata": {
            "$ref": "https://raw.githubusercontent.com/juharris/optify/refs/heads/main/schemas/feature_file.json#/definitions/metadata"
        },
        "options": {
            "type": "object",
            "additionalProperties": false,
            "minProperties": 1,
            "properties": {
                "model": {
                    "type": "object",
                    "additionalProperties": false,
                    "properties": {
                        "name": {
                            "type": "string"
                        },
                        "parameters": {
                            "type": "object"
                        }
                    }
                },
                "chat": {
                    "type": "object",
                    "additionalProperties": false,
                    "properties": {
                        "system_instructions": {
                            "$ref": "https://raw.githubusercontent.com/juharris/optify/refs/heads/main/schemas/feature_file.json#/definitions/configurableString"
                        },
                        "tools": {
                            "allOf": [
                                {
                                    "$ref": "https://raw.githubusercontent.com/juharris/optify/refs/heads/main/schemas/feature_file.json#/definitions/configurableList"
                                }
                            ],
                            "items": {
                                "type": "object",
                                "additionalProperties": false,
                                "properties": {
                                    "description": {
                                        "$ref": "https://raw.githubusercontent.com/juharris/optify/refs/heads/main/schemas/feature_file.json#/definitions/configurableString"
                                    },
                                    "name": {
                                        "type": "string"
                                    }
                                }
                            },
                            "additionalProperties": {
                                "properties": {
                                    "$value": {
                                        "$ref": "#/properties/options/properties/tools/items"
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
```

This validates both literal tool arrays and configurable tool entries while providing editor suggestions for `"name"`, `"description"`, and the nested ConfigurableString properties.
The tool schema is defined once under `"items"` and reused for each configurable entry's `"$value"`.
`"name"` and `"description"` intentionally remain optional because each feature file is validated before features are merged, and a feature may only set one property.
