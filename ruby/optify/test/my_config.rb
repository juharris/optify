# typed: true
# frozen_string_literal: true

require 'sorbet-runtime'
require 'tapioca'

class FromHashable
  extend T::Sig

  # Create a new instance of the class from a hash.
  #
  # @param hash [Hash] The hash to create the instance from.
  # @return The new instance.
  sig {params(hash: T::Hash[T.untyped, T.untyped]).returns(T.attached_class)}
  def self.from_hash(hash)
    result = self.new

    hash.each do |key, value|
      # TODO Might need some error handling here, but it should be fine if type signatures are used.
      # TODO Handle nillable types.
      case value
      when Array
        sig_return_type = T::Utils.signature_for_method(self.instance_method(key)).return_type
        inner_type = sig_return_type.type.raw_type
        if inner_type.respond_to?(:from_hash)
          value = value.map { |v| inner_type.from_hash(v) }
        end
      when Hash
        sig_return_type = T::Utils.signature_for_method(self.instance_method(key)).return_type
        if sig_return_type.respond_to?(:raw_type)
          type_for_key = sig_return_type.raw_type
          if type_for_key.respond_to?(:from_hash)
            value = type_for_key.from_hash(value)
          end
        end
      end

      result.instance_variable_set("@#{key}", value)
    end
    result
  end
end

class MyObject < FromHashable
  private
  attr_writer :one, :two, :string, :deeper
  public
  sig {returns(Integer)}
  attr_reader :one
  sig {returns(Integer)}
  attr_reader :two
  sig {returns(String)}
  attr_reader :string
  sig {returns(Hash)}
  attr_reader :deeper
end

# A custom configuration for testing.
class MyConfig < FromHashable
  private
  attr_writer :rootString, :myArray, :myObject, :myObjects, :deeper
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