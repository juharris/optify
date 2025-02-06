# typed: true
# frozen_string_literal: true

# TODO Figure out a more generalized way to have immutable objects that can recursively hold other objects and be initialized from a hash.

require 'sorbet-runtime'

require 'tapioca'

class BaseConfig
  extend T::Sig

  # FIXME Return type is not correct.
  sig {params(hash: T::Hash[T.untyped, T.untyped]).returns(T.self_type)}
  def self.from_hash(hash)
    # When using `< T::Struct` for immutable properties.
    result = self.new

    # When using attr_writer and attr_reader.
    hash.each do |key, value|
      # TODO Recurse from_hash if needed for hashes and within arrays.
      # Get the type of the value.
      # If it is a hash, call from_hash on it.
      type_of_value = value.class
      # type_for_key = MyConfig.member_types[key.to_sym]
      # class_for_value = MyConfig.
      type_of_key = T::Utils.signature_for_method(self.instance_method(key)).return_type
      p "key: #{key}, type_of_key: #{type_of_key}, type_of_value: #{type_of_value}"
      if type_of_value == Hash && type_of_key.respond_to?(:from_hash)
        puts "making value from hash"
        value = type_of_key.from_hash(value)
      end
      # TODO Handle arrays.
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
  attr_writer :rootString, :myArray, :myObject
  public
  sig {returns(String)}
  attr_reader :rootString
  sig {returns(T::Array[String])}
  attr_reader :myArray
  sig {returns(MyObject)}
  attr_reader :myObject
end