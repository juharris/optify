# Configurable Lists

A list or array can be configured by combining multiple objects across features.
The keys of objects with `$type: "Optify.ConfigurableList"` are ignored and the values are merged into a single array.
This allows features to add or remove entries from the list, and even specify a sorting preference for entries without needing to know the entire list or the position of other entries.

## Goals
- Declare a list/array **across several features**
- **Add** entries
- **Remove** entries
- **Positioning**: or sorting
  **TODO Figure out a good name.**
  Absolute posaitioning would be confusing and tricky, so we can support something like `$order` or `$priority` or `$sort` per entry to get something near the beginning or near the end relative to others.

Each entry must be an object with a `$value` key to specify the value of the entry, and an optional `$order` key to specify the sorting preference.
`$value` is required because if another feature just wants to change the order of an entry, then setting `$order` using an object would override a value that is not an object and special custom logic would be needed to handle merging such special cases.

## Naming

TODO Need to figure out:

* "Optify.ConfigurableList" vs. "Optify.ConfigurableArray":
  Justin likes list better because it's shorter and easier to say.
  Maybe it really is more like a list since we're talking about adding or inserting entries across features.
  Arrays are typically more fixed in size.
* `$value`
* `$order` vs. `$priority` vs. `$sort`:
  Maybe `$priority` is better because it implies that entries with higher priority should come later in the list, which is more intuitive for sorting.
  `$sort` could also work, but it might be confused with a sorting function or method.

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
      $order: -27
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
