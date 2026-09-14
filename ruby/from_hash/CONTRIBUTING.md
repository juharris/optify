# Development Notes

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
bundle exec tapioca gem --verify
bundle exec rubocop --cache true
bundle exec srb tc
```

To automatically change code and address issues, run:

```shell
bundle exec rubocop --autocorrect --cache true
bin/tapioca annotations
bundle exec tapioca gem
bin/tapioca todo
# Maybe one day:
# spoom bump --from false --to true
# spoom bump --from true --to strict
```

All in one line:

```shell
bundle exec rubocop --autocorrect --cache true && bin/tapioca annotations && bundle exec tapioca gem && bin/tapioca todo
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

Note that classes that inherit from `Optify:FromHashable` need Sorbet signatures for their attributes for `from_hash` to work.
So some classes will need Sorbet signatures.
We may investigate supporting RBS in the future.

## Benchmarks

Run:

```shell
ruby benchmarks/load.rb
```

## Publishing

A GitHub Action is setup to publish the gem as needed.
