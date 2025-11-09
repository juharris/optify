# Optify Rust Bindings for Ruby
[![Gem Version](https://badge.fury.io/rb/optify-config.svg?icon=si%3Arubygems&icon_color=%23ec3c3c)](https://badge.fury.io/rb/optify-config)

## Usage

The gem is called `optify-config`, but we would like to call it just `optify`, but that name was taken by a gem that has not been updated since 2012.
So we use the name `optify-config` to avoid conflicts, but the require statement is `optify` and the namespace is `Optify`.

Set up your configuration files as explained in the [root README.md](../../README.md).

```shell
gem install optify-config
```

Define your immutable configuration classes:
```ruby
require 'optify'

class MyObject < Optify::FromHashable
  sig { returns(Integer) }
  attr_reader :number

  sig { returns(String) }
  attr_reader :string
end

class MyConfig < Optify::FromHashable
  sig { returns(String) }
  attr_reader :name

  sig { returns(MyObject) }
  attr_reader :object

  sig { returns(T::Array[MyObject]) }
  attr_reader :objects
end
```

> Note that RBS style comments instead of Sorbet `sig`s are not supported
and may never be supported because RBS is only for static analysis and it is not used at runtime.

Use a provider:
```ruby
require 'optify'

# Create a new OptionsProvider
provider = Optify::OptionsProvider.build('path/to/configs')

# Get the configuration for "myConfig" when the features "feature1" and "feature2" are enabled
config = provider.get_options("myConfig", ['feature1', 'feature2'], MyConfig)

# Use the config
puts config.name
puts config.object.number
```

To watch for changes and automatically reload those changes into the provider when the configuration files change, use `OptionsWatcherBuilder` to create an `OptionsWatcher` which has the same interface as `OptionsProvider`:
```Ruby
provider = Optify::OptionsWatcher.build('path/to/configs')
```

See [optify_test.rb](test/optify_test.rb) for more examples.

## Setup
<!-- Some tips in https://github.com/matsadler/magnus/issues/77 -->

Use Ruby 3.4+., for example, run:
```shell
chruby 3.4.4
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

## Typing
To automatically convert Sorbet style to RBS:
```shell
bundle exec spoom srb sigs translate --from=rbi --to=rbs lib
```

Note that classes that inherit from `Optify:FromHashable` such as `OptionsMetadata` need Sorbet signatures for their attributes for `from_hash` to work.
So some classes will need Sorbet signatures.

<!--
If RBS supports checks **at runtime** and we can support RBS style signatures in comments for configuration objects:
bundle exec spoom srb sigs translate --from=rbi --to=rbs lib test
 -->

To generate the RBS file:
```shell
bundle exec rbs prototype rbi rbi/optify.rbi > sig/optify.rbs
```

See guidance in https://github.com/ruby/rbs/blob/master/docs/gem.md

## Formatting
To automatically change the Rust code, run:
```shell
cargo fmt && cargo clippy --fix --allow-dirty --allow-staged
```

To check for issues in Ruby code, run:
```shell
bundle exec rubocop
bundle exec srb tc
```

To automatically change code and address issues, run:
```shell
bundle exec rubocop --autocorrect
bin/tapioca annotations
bin/tapioca gem
bin/tapioca todo
# Maybe one day:
# spoom bump --from false --to true
# spoom bump --from true --to strict
```

All in one line:
```shell
bundle exec rubocop --autocorrect && bin/tapioca annotations && bin/tapioca gem && bin/tapioca todo
```

Verify the changes with:
```shell
bundle exec srb tc
```

## Publishing
A GitHub Action is setup to publish the gem as needed.
To publish manually, run the following with the correct version and architecture:
```shell
RB_SYS_CARGO_PROFILE='release' RB_SYS_CROSS_COMPILE=true rake native gem
gem push pkg/optify-config-<version>-<architecture>.gem
```

To build a source gem:
```shell
mkdir -p pkg
gem build optify.gemspec --output pkg/optify-config.gem
```

To check metadata for the gem file:
```shell
tar -xf pkg/optify-config-<version>-<architecture>.gem
gzip -d metadata.gz
less metadata
```

To inspect the contents of the gem:
```shell
gem unpack optify-config-<version>-<architecture>.gem
```

To see credentials to get the API key to update a GitHub Action, run:
For Mac:
```shell
cat ~/.local/share/gem/credentials
```
