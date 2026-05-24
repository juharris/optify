# Configurable Lists

A list or array can be configured by combining multiple objects across features.
The keys of the objects are ignored and the values are merged into a single array.
This allows features to add or remove entries from the list, and even specify a sorting preference for entries without needing to know the entire list or the position of other entries.

## Goals
- Declare a list/array **across several features**
- **Add** entries
- **Remove** entries
- **Positioning**: or sorting
  **TODO Figure out a good name.**
  Absolute posaitioning would be confusing and tricky, so we can support something like `$order` or `$priority` or `$sort` per entry to get something near the beginning or near the end relative to others.

## Example:
_The entire configurable example is merged for simplicity here, but imagine the keys were across several files._

### List of Objects 

```YAML
options:
  items:
    $type: "Optify.ConfigurableList"
    item_a: "value"
    item_b:
      # Should be early
      $order: -27
      $value:
        prop: "early value"
    item_c:
      prop: "don't care where this goes"
    # Remove an item
    item_d: null
    item_e: 3
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

Issues:
* If another feature just wants to change the order of an entry, should it be required to set `$value`? What if previous features don't set the `$value` and just use an object or primitive directly. If a primitive is used, then setting `$order` using an object would override and the primitive would be lost. Should we make a special case? This makes merge much more complex.
  **Could force to always use `$value`.**

---

Previous brainstorming notes:

What if we want a list of strings instead of objects?
Maybe we can allow for a special key like `$value` to specify the string value?
Kind of awkward because they we need lots of validation to ensure that `$value` isn't adjacent to other properties.
Maybe `$value` overrides add forces it to be used instead of using an object, unless `$value` is an object, but now there's 2 ways to have an object.
Should we just always enforce `$value`?

Could use schemas to validate, but many projects likely won't use a schema by default and schemas might say 2 independent files are correct, but they would not be compatible when merged.

`$item_type: "string"` or `$item_type: "primitive"` or `$item_type: "not object"`

```YAML
options:
  list:
    $type: "Optify.ConfigurableList"
    $item_type: "string"
    item_a:
      $value: "string value"
      $order: 25
    item_b:
      $value: "a string value"
```
becomes:
```JSON
{
  "list": [
    "a string value",
    "string value"
  ]
}
```
