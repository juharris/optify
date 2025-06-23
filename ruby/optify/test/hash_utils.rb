# typed: strict
# frozen_string_literal: true

class HashUtils
  #: (untyped) -> Hash[Symbol, untyped]
  def self.deep_symbolize_keys(object)
    case object
    when Hash
      object.map do |key, value|
        if key.is_a?(String)
          [key.to_sym, deep_symbolize_keys(value)]
        else
          [key, deep_symbolize_keys(value)]
        end
      end.to_h
    when Array
      object.map { |v| deep_symbolize_keys(v) }
    else
      object
    end
  end
end
