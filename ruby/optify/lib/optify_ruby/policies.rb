# typed: true
# frozen_string_literal: true

require 'sorbet-runtime'

require 'optify-from_hash'

module Optify
  # The policy for the requester identifier passed via preferences.
  #
  # Either `allow` or `block` will be set, not both.
  # - `allow`: only the listed requesters may use the feature; can be empty.
  # - `block`: the listed requesters may not use the feature; all others are allowed.
  class RequesterPolicy < FromHashable
    extend T::Sig

    sig { returns(T.nilable(T::Set[String])) }
    attr_reader :allow

    sig { returns(T.nilable(T::Set[String])) }
    attr_reader :block
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
