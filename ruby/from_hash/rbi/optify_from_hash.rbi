# frozen_string_literal: true
# typed: strong

# Tools for working with configurations declared in files.
module Optify
  # A base class for classes that can be created from a hash.
  class FromHashable
    abstract!

    # Create a new instance of the class from a hash.
    #
    # This is a class method that so that it can set members with private setters.
    # @param hash The hash to create the instance from.
    # @return The new instance.
    sig { params(hash: T::Hash[T.untyped, T.untyped]).returns(T.attached_class) }
    def self.from_hash(hash); end

    # Convert this object to a JSON string.
    sig { params(args: T.untyped).returns(String) }
    def to_json(*args); end

    # Convert this object to a Hash recursively.
    # This is mostly the reverse operation of `from_hash`,
    # as keys will be symbols
    # and `from_hash` will convert strings to symbols if that's how the attribute is declared.
    # @return The hash representation of this object.
    sig { returns(T::Hash[Symbol, T.untyped]) }
    def to_h; end

    # Compare this object with another object for equality.
    # @param other The object to compare.
    # @return [Boolean] true if the objects are equal; otherwise, false.
    sig { params(other: T.untyped).returns(T::Boolean) }
    def ==(other); end
  end
end
