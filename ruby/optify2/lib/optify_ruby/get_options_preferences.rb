# typed: strict
# frozen_string_literal: true

require 'sorbet-runtime'
require 'json'

module Optify
  # Preferences for `get_options`.
  class GetOptionsPreferences
    extend T::Sig

    #: bool
    attr_accessor :skip_feature_name_conversion

    #: String?
    attr_accessor :constraints_json

    #: String?
    attr_accessor :overrides_json

    #: bool
    attr_reader :configurable_strings_enabled

    #: bool
    attr_reader :configurable_strings_explicitly_set

    #: -> void
    def initialize
      @skip_feature_name_conversion = false #: bool
      @constraints_json = nil #: String?
      @overrides_json = nil #: String?
      @configurable_strings_enabled = false #: bool
      @configurable_strings_explicitly_set = false #: bool
    end

    #: (Hash[String, untyped]? constraints) -> void
    def constraints=(constraints)
      @constraints_json = constraints&.to_json
    end

    #: -> Hash[String, untyped]?
    def constraints
      return nil unless @constraints_json

      JSON.parse(@constraints_json)
    end

    #: (Hash[String, untyped]? overrides) -> void
    def overrides=(overrides)
      @overrides_json = overrides&.to_json
    end

    #: -> Hash[String, untyped]?
    def overrides
      return nil unless @overrides_json

      JSON.parse(@overrides_json)
    end

    #: -> void
    def enable_configurable_strings
      @configurable_strings_enabled = true
      @configurable_strings_explicitly_set = true
    end

    #: -> void
    def disable_configurable_strings
      @configurable_strings_enabled = false
      @configurable_strings_explicitly_set = true
    end

    #: -> bool
    def are_configurable_strings_enabled?
      @configurable_strings_enabled
    end

    #: -> bool
    def overrides?
      !@overrides_json.nil?
    end
  end
end
