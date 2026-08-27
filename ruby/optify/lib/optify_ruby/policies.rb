# typed: true
# frozen_string_literal: true

require 'sorbet-runtime'

module Optify
  # The policy for the requester identifier passed via preferences.
  #
  # Either `allowed` or `blocked` will be set, not both.
  # - `allowed`: only the listed requesters may use the feature; can be empty.
  # - `blocked`: the listed requesters may not use the feature; all others are allowed.
  class RequesterPolicy
    #: Array[String]?
    attr_reader :allowed

    #: Array[String]?
    attr_reader :blocked

    #: (Hash[String, untyped]) -> RequesterPolicy
    def self.from_hash(hash)
      policy = new
      policy.instance_variable_set(:@allowed, hash['allowed']&.sort)
      policy.instance_variable_set(:@blocked, hash['blocked']&.sort)
      policy
    end

    #: (untyped) -> bool
    def ==(other)
      return false unless other.is_a?(RequesterPolicy)

      allowed == other.allowed && blocked == other.blocked
    end
  end

  # Policies that restrict access to a feature based on values in the request's preferences.
  #
  # See https://github.com/juharris/optify#policies for details.
  class Policies
    #: RequesterPolicy?
    attr_reader :requester

    #: (Hash[String, untyped]) -> Policies
    def self.from_hash(hash)
      policies = new
      requester_hash = hash['requester']
      policies.instance_variable_set(:@requester, requester_hash && RequesterPolicy.from_hash(requester_hash))
      policies
    end

    #: (untyped) -> bool
    def ==(other)
      return false unless other.is_a?(Policies)

      requester == other.requester
    end
  end
end
