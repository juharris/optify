# Configurable Strings

Strings can be configured with a `"base"` starting root template and `"arguments"` to fill the template.
This encourages reusing common substrings and sharing them amongst features while allowing features to override parts of the string.

`"base"` and values in `"arguments"` can be strings, or objects that reference a file or a [Liquid][Liquid] template.

Simple example:
```JSON
{
  "options": {
    "greeting": {
      "$type": "Optify.ConfigurableString",
      "base": {
        "liquid": "Hello, {{name}}!"
      },
      "arguments": {
        "name": "World"
      }
    }
  }
}
```

Result: `{ "greeting": "Hello, World!" }`

**Strings are built when the configuration is retrieved.**
Meaning that they will be built eagerly in Rust and cached when requested from a provider,
speeding up your runtime for subsequent requests.

## Overview

Configurable strings provide:
- **Template-based string generation** using Liquid syntax
- **Variable substitution** with configurable arguments in Liquid templates
- **File-based templates** for longer or reusable content in the root or arguments
- **Inheritance and override** capabilities across features, like any other configurable object

## Enabling Configurable Strings

In `.optify/config.json` for the directory where you want to enable configurable strings:

```JSON
{
  "$schema": "https://raw.githubusercontent.com/juharris/optify/refs/heads/main/schemas/optify_config.json",
  "areConfigurableStringsEnabled": true
}
```

## Basic Structure

A configurable string is defined using the special type `Optify.ConfigurableString` with two main components:
1. **root**: The template string or source
2. **arguments**: Optional variables to substitute in the template

## Syntax Options

### 1. Simple String

The simplest form is a plain string without any variables:

```JSON
{
  "greeting": {
    "$type": "Optify.ConfigurableString",
    "base": "Hello, World!"
  }
}
```

Result: `"Hello, World!"`

### 2. Liquid Template with Variables

Use Liquid syntax for variable substitution:

```JSON
{
  "message": {
    "$type": "Optify.ConfigurableString",
    "base": {
      "liquid": "Welcome to {{ name }}!"
    },
    "arguments": {
      "name": "Optify"
    }
  }
}
```

Result: `"Welcome to Optify!"`

### 3. File-based Templates

Load templates from external files:

```JSON
{
  "greeting_from_file": {
    "$type": "Optify.ConfigurableString",
    "base": {
      "file": "templates/greeting.txt"
    }
  }
}
```

Where `templates/greeting.txt` contains:
```
Hello from template file!
```

Result: `"Hello from template file!"`

### 4. File-based Liquid Templates

Combine file loading with Liquid templates:

```JSON
{
  "message_from_liquid_file": {
    "$type": "Optify.ConfigurableString",
    "base": {
      "file": "templates/message.liquid"
    },
    "arguments": {
      "app_name": "Optify",
      "message_from_file": {
        "file": "templates/message.txt"
      }
    }
  }
}
```

Where `templates/message.liquid` contains:
```liquid
Welcome to {{ app_name }}! {{ message_from_file }}
```

Where `templates/message.txt` contains:
```
This message is from a file.
```

Result: `"Welcome to Optify! This message is from a file."`

## Overriding Arguments

One of the most powerful features of configurable strings is the ability to override arguments in other feature files while keeping the template intact.

### Example: Base Configuration

`feature_A.json`:
```JSON
{
  "options": {
    "welcome_message": {
      "$type": "Optify.ConfigurableString",
      "base": {
        "liquid": "Hello, {{audience}}!"
      },
      "arguments": {
        "audience": "World"
      }
    }
  }
}
```

### Example: Override Arguments

`feature_B.yaml`:
```YAML
options:
  welcome_message:
    arguments:
      audience: "Justin"
```

When features `["feature_A", "feature_B"]` are applied, the result is:
```JSON
{
  "welcome_message": "Hello, Justin!"
}
```

Notice how only the `arguments` were overridden, not the template itself.

## Nested Arguments

Arguments can reference other arguments using Liquid templates:

```JSON
{
  "complex_message": {
    "$type": "Optify.ConfigurableString",
    "base": {
      "liquid": "{{ greeting }}, {{ name }}! {{ closing }}"
    },
    "arguments": {
      "greeting": {
        "liquid": "Welcome to {{ location }}"
      },
      "location": "Optify",
      "name": "Developer",
      "closing": "Enjoy your stay!"
    }
  }
}
```

Result: `"Welcome to Optify, Developer! Enjoy your stay!"`

## Use Cases

### 1. Environment-Specific Messages
Different messages for different environments without changing code:

```JSON
{
  "api_endpoint": {
    "$type": "Optify.ConfigurableString",
    "base": {
      "liquid": "https://{{ subdomain }}.example.com/api/{{ version }}"
    },
    "arguments": {
      "subdomain": "dev",
      "version": "v1"
    }
  }
}
```

### 2. Dynamic Error Messages
Provide context-aware error messages:

```JSON
{
  "error_message": {
    "$type": "Optify.ConfigurableString",
    "base": {
      "liquid": "Failed to {{ action }} {{ resource }}: {{ reason }}"
    },
    "arguments": {
      "action": "load",
      "resource": "user profile",
      "reason": "network timeout"
    }
  }
}
```

## Best Practices

1. **Keep templates focused**: Each configurable string should represent a single, cohesive message or value
2. **Use descriptive argument names**: Make it clear what each argument represents
3. **Document complex templates**: Add comments or documentation for templates with multiple variables
4. **Prefer file-based templates for long content**: Keep configuration files clean by moving longer templates to separate files
5. **Test argument overrides**: Ensure that overriding arguments produces the expected results

## File Organization

When using file-based templates, organize them logically:

```
configurations/
├── feature_a.json
├── feature_b.yaml
└── templates/
    ├── emails/
    │   ├── welcome.liquid
    │   └── notification.liquid
    ├── ui/
    │   ├── error.liquid
    │   └── success.liquid
    └── api/
        └── responses.liquid
```

## Liquid Template Features

Configurable strings support standard Liquid template features:

- **Variables**: `{{ variable_name }}`
- **Filters**: `{{ name | upcase }}`, `{{ price | round: 2 }}`

For full Liquid syntax documentation, see the [Liquid template language documentation](https://shopify.github.io/liquid/).

## Examples and Tests

For more comprehensive examples and test cases, see the [test suite for configurable values](../tests/test_suites/configurable_values/).

The test suite includes:
- Simple string configurations
- Template overrides
- File-based templates
- Complex nested arguments
- Various data type handling

[Liquid]: https://shopify.github.io/liquid
