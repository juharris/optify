# Optify Rust Bindings for Ruby

⚠️ Development in progress ⚠️\
APIs are not final and will change, for example, interfaces with be used.
This is just meant to be minimal to get started and help build a Ruby library.

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
