# typed: strict
# frozen_string_literal: true

require 'benchmark'
require_relative '../lib/optify-from_hash'

puts "PID: #{Process.pid}"

N = 1_000
WARMUP_COUNT = 10

# Four-level-deep class hierarchy for the benchmark.

class Level4Config < Optify::FromHashable
  sig { returns(String) }
  attr_reader :name

  sig { returns(Integer) }
  attr_reader :count

  sig { returns(Float) }
  attr_reader :ratio

  sig { returns(T.nilable(T::Boolean)) }
  attr_reader :enabled
end

class Level3Config < Optify::FromHashable
  sig { returns(String) }
  attr_reader :label

  sig { returns(Integer) }
  attr_reader :value

  sig { returns(T::Array[String]) }
  attr_reader :tags

  sig { returns(Level4Config) }
  attr_reader :child

  sig { returns(T::Array[Integer]) }
  attr_reader :scores
end

class Level2Config < Optify::FromHashable
  sig { returns(String) }
  attr_reader :title

  sig { returns(Integer) }
  attr_reader :index

  sig { returns(T.nilable(T::Boolean)) }
  attr_reader :active

  sig { returns(T::Set[String]) }
  attr_reader :label_set

  sig { returns(T::Array[Level3Config]) }
  attr_reader :items

  sig { returns(Level3Config) }
  attr_reader :meta
end

class Level1Config < Optify::FromHashable
  sig { returns(String) }
  attr_reader :name

  sig { returns(Integer) }
  attr_reader :version

  sig { returns(String) }
  attr_reader :description

  sig { returns(Level2Config) }
  attr_reader :section

  sig { returns(T::Array[Level2Config]) }
  attr_reader :sections
end

DEEP_HASH = {
  name: 'root',
  version: 42,
  description: 'A benchmark configuration with nested levels',
  section: {
    title: 'main',
    index: 1,
    active: true,
    label_set: %w[main primary],
    items: [
      {
        label: 'item1',
        value: 10,
        tags: %w[alpha beta gamma],
        child: { name: 'leaf1', count: 3, ratio: 1.5, enabled: true },
        scores: [1, 2, 3, 4]
      },
      {
        label: 'item2',
        value: 20,
        tags: ['delta'],
        child: { name: 'leaf2', count: 7, ratio: 0.5, enabled: false },
        scores: [5, 6, 7]
      }
    ],
    meta: {
      label: 'meta',
      value: 99,
      tags: %w[meta-tag other-tag],
      child: { name: 'meta-leaf', count: 0, ratio: 2.718, enabled: nil },
      scores: [10, 20, 30, 40]
    }
  },
  sections: [
    {
      title: 'secondary',
      index: 2,
      active: false,
      label_set: %w[secondary alternate],
      items: [
        {
          label: 'sec-item1',
          value: 5,
          tags: %w[epsilon zeta],
          child: { name: 'sec-leaf1', count: 1, ratio: 3.14, enabled: true },
          scores: [100, 200]
        },
        {
          label: 'sec-item2',
          value: 15,
          tags: ['eta'],
          child: { name: 'sec-leaf2', count: 4, ratio: 0.25, enabled: false },
          scores: [50]
        }
      ],
      meta: {
        label: 'sec-meta',
        value: 0,
        tags: [],
        child: { name: 'sec-meta-leaf', count: 2, ratio: 0.1, enabled: nil },
        scores: [7, 8, 9]
      }
    }
  ]
}.freeze

# Verify that round-tripping from_hash -> to_h produces a hash equal to the original.
round_tripped = Level1Config.from_hash(DEEP_HASH).to_h
raise "Round-trip check failed:\n#{round_tripped}" unless round_tripped == DEEP_HASH

Benchmark.bm(10) do |x|
  WARMUP_COUNT.times do
    Level1Config.from_hash(DEEP_HASH)
  end

  x.report('from_hash') do
    N.times do
      Level1Config.from_hash(DEEP_HASH)
    end
  end
end
