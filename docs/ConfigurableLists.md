# Configurable Lists

A list or array can be configured by combining multiple objects across features.
The keys of objects with `$type: "Optify.ConfigurableList"` are ignored and the values are merged into a single array.
This allows features to add or remove entries from the list, and even specify a sorting preference for entries without needing to know the entire list or the position of other entries.

## Capabilities

- Declare a list or array **across several features**
- **Add** entries
- **Remove** entries
- **Sort** entries:
  Absolute positioning would be confusing.
  Use an `$order` property with a numerical value (float) to specify a sorting preference for entries, where higher values are sorted later in the list.
  This allows features to specify that certain entries should be earlier or later in the list without needing to know the entire list or the position of other entries.

## Enabling Configurable Lists

In `.optify/config.json` for the directory where you want to enable configurable values:

```JSON
{
  "$schema": "https://raw.githubusercontent.com/juharris/optify/refs/heads/main/schemas/optify_config.json",
  "areConfigurableValuesEnabled": true
}
```

Configurable values must also be enabled in the preferences given when getting options from an `OptionsProvider`.

## List Entries

| Property                                    | Description                                                                                   |
| ------------------------------------------- | --------------------------------------------------------------------------------------------- |
| `$value`                                    | **Required** The value of the entry.<br/> A `$value: null` yields a `null` entry in the list. |
| `$order`                                    | (Optional) Sorting preference for the entry.                                                  |
| Higher values are sorted later in the list. |
| Default is `0`.                             |

`$value` is required because if another feature just wants to change the order of an entry, then setting `$order` using an object would override a value that is not an object and special custom logic would be needed to handle merging such special cases across features.

**Delete**: A key with a `null` value means to remove the entry from the list.

## Example:

_The entire configurable example is merged for simplicity here, but imagine the keys were across several files._

### List of Objects

```YAML
options:
  items:
    $type: "Optify.ConfigurableList"
    item_a:
      $value: "value"
    item_b:
      # Should be early
      $order: -27.2
      $value:
        prop: "early value"
    item_c:
      $value:
        prop: "don't care where this goes"
    # Remove an item
    item_d: null
    item_e:
      $value: 3
    item_f:
      # Should be later
      $order: 999
      $value:
        prop: "late value"
    # `null` entry
    item_g:
      $value: null
```

`item_d: null` is ignored because a value of `null` means remove the entry.
Use `$value: null` to yield a `null` entry in the array.

becomes:

```JSONC
{
  "items": [
    // item_b
    {
      "prop": "early value"
    },
    // item_a
    "value",
    // item_c
    {
      "prop": "don't care where this goes"
    },
    // item_e
    3,
    // item_g: from `$value: null`
    null,
    // item_f
    {
      "prop": "late value"
    }
  ]
}
```

## Validating List Value Types

The standard Optify schema allows any JSON value in a configurable list.
A custom schema can compose the standard `configurableList` definition with its own `items` schema.

For example, this schema gives `options.scores` numeric values and `options.books` object values with `author` and `title` properties:

```JSONC
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "type": "object",
  // Other root schema properties...
  "properties": {
    "options": {
      "type": "object",
      "properties": {
        "scores": {
          "allOf": [
            {
              "$ref": "https://raw.githubusercontent.com/juharris/optify/refs/heads/main/schemas/feature_file.json#/definitions/configurableList"
            }
          ],
          "items": {
            "type": "number"
          },
          "additionalProperties": {
            "properties": {
              "$value": {
                "$ref": "#/properties/options/properties/scores/items"
              }
            }
          }
        },
        "books": {
          "allOf": [
            {
              "$ref": "https://raw.githubusercontent.com/juharris/optify/refs/heads/main/schemas/feature_file.json#/definitions/configurableList"
            }
          ],
          "items": {
            "type": "object",
            "additionalProperties": false,
            "properties": {
              "author": {
                "type": "string"
              },
              "title": {
                "type": "string"
              }
            }
          },
          "additionalProperties": {
            "properties": {
              "$value": {
                "$ref": "#/properties/options/properties/books/items"
              }
            }
          }
        }
      }
    }
  }
}
```

`allOf` applies the standard configurable-list structure.
`items` validates literal array elements, and `additionalProperties` applies that same item schema to each configurable list entry's `$value`.
Each property has its own `items` schema, so a schema and configuration file can contain any number of independently typed configurable lists.
