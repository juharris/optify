# typed: strict
# frozen_string_literal: true

module Optify
  # Preferences for `get_options`.
  class GetOptionsPreferences
    # @param constraints [Hash]
    #: (Hash[untyped, untyped]? constraints) -> void
    def constraints=(constraints)
      self.constraints_json = constraints&.to_json
    end

    # @param overrides [Hash]
    #: (Hash[untyped, untyped]? overrides) -> void
    def overrides=(overrides)
      self.overrides_json = overrides&.to_json
    end
  end
end
