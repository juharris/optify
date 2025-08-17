# frozen_string_literal: true
# typed: true

require 'sorbet-runtime'

require_relative './base_config'

module Optify
  # Information about a feature.
  class OptionsMetadata < BaseConfig
    sig { returns(T.nilable(T::Array[String])) }
    attr_reader :aliases

    # The canonical names of features that depend on this one.
    sig { returns(T.nilable(T::Array[String])) }
    attr_reader :dependents

    sig { returns(T.untyped) }
    attr_reader :details

    sig { returns(String) }
    attr_reader :name

    sig { returns(T.nilable(String)) }
    attr_reader :owners

    sig { returns(T.nilable(String)) }
    attr_reader :path
  end
end
