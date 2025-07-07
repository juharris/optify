# Conditions
Conditions can be used to enable a feature file when it is requested and when constraints are given in the request.
Conditions are meant for temporary experimental features that should only be enabled in some requests.

If no constraints are given, then the conditions in a feature file are are ignored.
Most projects should either always use constraints in every request or never use constraints in order to avoid confusion.

Conditions cannot be used in imported features.
This helps keep retrieving and building configuration options for a list of features fast and more predictable because imports do not need to be re-evaluated.
Instead, keep each feature file as granular and self-contained as possible, then use conditions and import the required granular features in a feature file that defines a common scenario.

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

## Condition Types

A condition expression can be:
- `{ "and": [ <condition expression>, ...] }`
- `{ "or": [ <condition expression>, ...] }`
- `{ "not": <condition expression> }`
- `{ "jsonPointer": <json pointer>, "equals": <JSON value> }`
- `{ "jsonPointer": <json pointer>, "matches": <regex> }`

`"equals"` can use any JSON value, including objects and arrays.

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

Some versions might only accept constraints as a JSON string for now.