# typed: true
# frozen_string_literal: true

require_relative '../lib/optify_ruby/base_config'

# A custom configuration for testing for the conditions folder.
class MyConditionsConfig < Optify::FromHashable
  sig { returns(String) }
  attr_reader :key

  sig { returns(T.nilable(String)) }
  attr_reader :key_a

  sig { returns(T.nilable(String)) }
  attr_reader :key_b

  sig { returns(T.nilable(String)) }
  attr_reader :key_docs_example
end
