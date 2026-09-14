# Optify Utilities for Converting Hashes

[![Gem Version](https://badge.fury.io/rb/optify-from_hash.svg?icon=si%3Arubygems&icon_color=%23ec3c3c)](https://badge.fury.io/rb/optify-from_hash)

Helps convert hashes to immutable objects.

## Usage

```shell
gem install optify-from_hash
```

Define your immutable classes:

```ruby
require 'optify-from_hash'

class MyObject < Optify::FromHashable
  sig { returns(Integer) }
  attr_reader :number

  sig { returns(String) }
  attr_reader :string
end

class MyConfig < Optify::FromHashable
  sig { returns(String) }
  attr_reader :name

  sig { returns(T::Array[MyObject]) }
  attr_reader :objects
end

config = MyConfig.from_hash(
  name: 'My Config',
  objects: [
    {
      number: 1,
      string: 'My String'
    },
    {
      number: 2,
      string: 'My String 2'
    }
  ]
)

puts config.name # "My Config"
puts config.objects[1].number # 2
```

> Note that RBS style comments instead of Sorbet `sig`s are not supported
> and may never be supported because RBS is only for static analysis and it is not used at runtime.

Symbols and and classes that define a `deserialize` method can be used for strings in JSON.
For example, if your project uses the [enummify gem][enummify] for immutable, RBS comment-friendly, typed Ruby enums,
then values in JSON can be automatically deserialized into the corresponding enum instances,
as well as keys in hashes.

```Ruby
require 'enummify'
require 'optify-from_hash'

class Service < Enummify::Enum
  DATABASE = new #: Service
  SEARCH = new #: Service
end

class Action < Enummify::Enum
  START = new #: Action
  STOP = new #: Action
end

class Status < Enummify::Enum
  PENDING = new #: Status
  RUNNING = new #: Status
  SUCCEEDED = new #: Status
end

class Response < Optify::FromHashable
  sig { returns(Service) }
  attr_reader :service

  sig { returns(T::Hash[Action, Status]) }
  attr_reader :statuses
end
```

# Development Notes

See [CONTRIBUTING.md](CONTRIBUTING.md) for development notes and instructions.

[enummify](https://rubygems.org/gems/enummify)
