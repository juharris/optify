# Conditions
Conditions can be used to enable a feature file when it is requested and when constraints are given in the request.
Conditions are meant for temporary experimental features that should only be enabled in some requests.

If no constraints are given, then the conditions in a feature file are are ignored.
Most projects should either always use constraints in every request or never use constraints in order to avoid confusion.

Conditions cannot be used in imported features.
This helps keep retrieving and building configuration options for a list of features fast and more predictable because imports do not need to be re-evaluated.
Instead, keep each feature file as granular and self-contained as possible, then use conditions and import the required granular features in a feature file that defines a common scenario.

The [recommended extensions](../README.md#recommended-extensions) can help you construct and validate conditions in feature files.

## Examples
See this [tests folder](../tests/test_suites/conditions) for more examples.

Suppose that a feature file has the following conditions:
```JSON
{
    "conditions": {
        "or": [
            {
                "jsonPointer": "/clientId",
                "equals": 1234
            },
            {
                "jsonPointer": "/page",
                "matches": "^https://mysite.com/"
            }
        ]
    }
}
```

✅ Then a request to get options with the following constraints will enable the feature file:

```JSON
{
    "page": "https://mysite.com/page",
    "clientId": 9876
}
```

or

```JSON
{
    "clientId": 1234
}
```

❌ A request to get options with the following constraints **will not** enable the feature file:
```JSON
{
    "page": "https://anothersite.com/slug"
}
```

### Matching Mutiple Values

To apply the feature for multiple values of a constraint and achieve something like a "list contains" check,
use either a regex or `or`.
For example, with the following constraints:
```JSON
{
    "page": "https://mysite.com/page",
}
```

Either of these conditions will match:
```JSON
{
    "conditions": {
        "jsonPointer": "/page",
        "matches": "^https://mysite.com/"
    }
}
```

or use `or`:
```JSON
{
    "conditions": {
        "or": [
            {
                "jsonPointer": "/page",
                "equals": "https://mysite.com/page"
            },
            {
                "jsonPointer": "/page",
                "matches": "^https://mysite.com/(about|page)$"
            }
        ]
    }
}
```

## Condition Types

A condition expression can be:
- `{ "and": [ <condition expression>, ...] }`
- `{ "or": [ <condition expression>, ...] }`
- `{ "not": <condition expression> }`
- `{ "jsonPointer": <json pointer>, "equals": <JSON value> }`
- `{ "jsonPointer": <json pointer>, "matches": "<regex>" }`

`"equals"` can use any JSON value, including objects and arrays.

`"matches"` accepts a regular expression as a string.
This can be useful to determine if an array contains a value.
It can match any JSON value, even a object, but be careful because there are no guarantees on the order of the keys in an object.

See the [schema file](../schemas/feature_file.json) for more details.

## Passing Constraints

Constraints are passed when getting options.

For example, in Ruby:
```Ruby
provider = Optify::OptionsProvider.build('path/to/configs')
cache_options = Optify::CacheOptions.new
preferences = Optify::GetOptionsPreferences.new
preferences.constraints = {
    page: "https://mysite.com/page",
    clientId: 9876
}

provider.get_options('myConfig', ['feature_A'], MyConfig, cache_options, preferences)
```
