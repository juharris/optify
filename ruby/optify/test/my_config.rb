# typed: true
# frozen_string_literal: true

# TODO Figure out a more generalized way to have immutable objects that can recursively hold other objects and be initialized from a hash.

require 'sorbet-runtime'

require 'tapioca'

class BaseConfig
  extend T::Sig

  # FIXME Get the return type correct. It's tricky with inheritance.
  sig {params(hash: T::Hash[T.untyped, T.untyped]).returns(T.untyped)}
  def self.from_hash(hash)
    result = self.new

    hash.each do |key, value|
      # TODO Might need some error handling here.
      case value
      when Array
        inner_type = T::Utils.signature_for_method(self.instance_method(key)).return_type.type.raw_type
        if inner_type.methods.include?(:from_hash)
          value = value.map { |v| inner_type.from_hash(v) }
        end
      when Hash
        type_for_key = T::Utils.signature_for_method(self.instance_method(key)).return_type.raw_type
        if type_for_key.methods.include?(:from_hash)
          value = type_for_key.from_hash(value)
        end
      end

      result.instance_variable_set("@#{key}", value)
    end
    result
  end
end

# A simple object with a single integer property.


class MyObject < BaseConfig
  extend T::Sig
  private
  attr_writer :two
  public
  sig {returns(Integer)}
  attr_reader :two
end

# A custom configuration for testing.
class MyConfig < BaseConfig
  extend T::Sig
  private
  attr_writer :rootString, :myArray, :myObject, :myObjects
  public
  sig {returns(String)}
  attr_reader :rootString
  sig {returns(T::Array[String])}
  attr_reader :myArray
  sig {returns(MyObject)}
  attr_reader :myObject
  sig {returns(T::Array[MyObject])}
  attr_reader :myObjects
end