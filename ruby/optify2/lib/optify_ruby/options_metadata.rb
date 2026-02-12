# typed: true
# frozen_string_literal: true

require 'sorbet-runtime'
require 'json'
require_relative './base_config'

module Optify
  # Information about a feature.
  class OptionsMetadata < FromHashable
    sig { returns(T.nilable(T::Array[String])) }
    attr_reader :aliases

    sig { returns(T.nilable(T::Array[String])) }
    attr_reader :dependents

    sig { returns(T.untyped) }
    attr_reader :details

    sig { returns(T.nilable(String)) }
    attr_reader :name

    sig { returns(T.nilable(String)) }
    attr_reader :owners

    sig { returns(T.nilable(String)) }
    attr_reader :path

    sig { params(hash: T::Hash[T.untyped, T.untyped]).returns(OptionsMetadata) }
    def self.from_hash(hash)
      # Convert empty arrays to nil before creating the object
      modified_hash = hash.dup
      modified_hash['dependents'] = nil if hash['dependents'].is_a?(Array) && hash['dependents'].empty?
      modified_hash['aliases'] = nil if hash['aliases'].is_a?(Array) && hash['aliases'].empty?
      super(modified_hash)
    end
  end
end
