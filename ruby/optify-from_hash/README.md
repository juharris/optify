# Optify Utilities for Converting Hashes

[![Gem Version](https://badge.fury.io/rb/optify-from_hash.svg?icon=si%3Arubygems&icon_color=%23ec3c3c)](https://badge.fury.io/rb/optify-from_hash)

Helps convert hashes to immutable objects.

## Usage

```shell
gem install optify-from_hash
```

Define your immutable classes:
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

## Setup
Use Ruby 3.4+., for example, run:
```shell
chruby 3.4.4
```

Run:
```shell
bundle install
```

## Testing

Run:
```shell
bundle exec rake test
```

## Style
To check for issues, run:
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

## Typing
To automatically convert Sorbet style to RBS:
```shell
bundle exec spoom srb sigs translate --from=rbi --to=rbs lib
```

Note that classes that inherit from `Optify:FromHashable` Sorbet signatures for their attributes for `from_hash` to work.
So some classes will need Sorbet signatures.
When it is possible to convert an RBS signature, then this library will try to support it.

<!--
If RBS supports checks **at runtime** and we can support RBS style signatures in comments for configuration objects:
bundle exec spoom srb sigs translate --from=rbi --to=rbs lib test
 -->

To generate the RBS file:
```shell
bundle exec rbs prototype rbi rbi/optify_from_hash.rbi > sig/optify_from_hash.rbs
```

See guidance in https://github.com/ruby/rbs/blob/master/docs/gem.md

## Publishing
A GitHub Action is setup to publish the gem as needed.
To publish manually, run the following with the correct version and architecture:
```shell
rake gem
gem push pkg/optify-from_hash-<version>-<architecture>.gem
```

To check metadata for the gem file:
```shell
tar -xf pkg/optify-from_hash-<version>-<architecture>.gem
gzip -d metadata.gz
less metadata
```

To inspect the contents of the gem:
```shell
gem unpack optify-from_hash-<version>-<architecture>.gem
```

To see credentials to get the API key to update a GitHub Action, run:
For Mac:
```shell
cat ~/.local/share/gem/credentials
```
