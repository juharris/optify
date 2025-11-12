# typed: true
# frozen_string_literal: true

require 'sorbet-runtime'
require 'json'
require_relative './base_config'

module Optify
  # Information about a feature.
  class OptionsMetadata < FromHashable
    extend T::Sig

    sig { returns(T::Array[String]) }
    attr_reader :aliases

    sig { returns(T::Array[String]) }
    attr_reader :dependents

    sig { returns(T.untyped) }
    attr_reader :details

    sig { returns(String) }
    attr_reader :name

    sig { returns(T.nilable(String)) }
    attr_reader :owners

    sig { returns(T.nilable(String)) }
    attr_reader :path

    #: (*untyped _args) -> String
    def to_json(*_args)
      {
        'aliases' => @aliases,
        'dependents' => @dependents,
        'details' => @details,
        'name' => @name,
        'owners' => @owners,
        'path' => @path
      }.to_json
    end
  end
end
