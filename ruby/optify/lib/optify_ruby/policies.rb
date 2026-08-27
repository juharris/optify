# typed: true
# frozen_string_literal: true

require 'sorbet-runtime'

require_relative './base_config'

module Optify
  # The policy for the requester identifier passed via preferences.
  #
  # Either `allowed` or `blocked` will be set, not both.
  # - `allowed`: only the listed requesters may use the feature; can be empty.
  # - `blocked`: the listed requesters may not use the feature; all others are allowed.
  class RequesterPolicy < FromHashable
    extend T::Sig

    sig { returns(T.nilable(T::Set[String])) }
    attr_reader :allowed

    sig { returns(T.nilable(T::Set[String])) }
    attr_reader :blocked
  end

  # Policies that restrict access to a feature based on values in the request's preferences.
  #
  # See https://github.com/juharris/optify#policies for details.
  class Policies < FromHashable
    extend T::Sig

    sig { returns(T.nilable(RequesterPolicy)) }
    attr_reader :requester
  end
end
