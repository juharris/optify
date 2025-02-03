# Optify Rust Bindings for Ruby
[![Gem Version](https://badge.fury.io/rb/optify-config.svg?icon=si%3Arubygems&icon_color=%23ec3c3c)](https://badge.fury.io/rb/optify-config)

⚠️ Development in progress ⚠️\
APIs are not final and will change, for example, interfaces with be used.
This is just meant to be minimal to get started and help build a Ruby library.

## Usage

The gem is called `optify-config`, but we would like to call it just `optify`, but that name was taken by a gem that has not been updated since 2012.
So we use the name `optify-config` to avoid conflicts, but the require statement is `optify` and the namespace is `Optify`.

```shell
gem install optify-config
```

```ruby
require 'optify'

# Create a new OptionsProviderBuilder
provider = Optify::OptionsProviderBuilder.new
    .add_directory('configs')
    .build

# Get the configuration for "myConfig" when the features "feature1" and "feature2" are enabled
config = provider.get_options("myConfig", ['feature1', 'feature2'])
```

## Setup
<!-- Some tips in https://github.com/matsadler/magnus/issues/77 -->

Use Ruby 3.3+., for example, run:
```shell
chruby 3.3.1
```

Run:
```shell
bundle install
```

## Building
Run:
```shell
rake compile
```

To make a release build, run:
```shell
RB_SYS_CARGO_PROFILE='release' rake compile
```

## Testing

Run:
```shell
rake test
```

## Formatting
To check for issues, run:
```shell
bundle exec srb tc
```

To automatically change code and address issues, run:
```shell
bin/tapioca annotations
bin/tapioca gem
bin/tapioca todo
# Maybe one day:
# spoom bump --from false --to true
# spoom bump --from true --to strict
```

All in one line:
```shell
bin/tapioca annotations && bin/tapioca gem && bin/tapioca todo
```

Verify the changes with:
```shell
bundle exec srb tc
```

## Publishing
A GitHub Action is setup to publish the gem as needed.
To publish manually, run the following with the correct version and architecture:
```shell
rake native gem
gem push pkg/optify-<version>-<architecture>.gem
```

To see credentials to get the API key to update a GitHub Action, run:
For Mac:
```shell
cat ~/.local/share/gem/credentials
```
