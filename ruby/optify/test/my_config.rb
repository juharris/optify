# typed: true
# frozen_string_literal: true

require 'sorbet-runtime'
require 'tapioca'

class MyObject < Optify::BaseConfig
  sig { returns(Integer) }
  attr_reader :one

  sig { returns(Integer) }
  attr_reader :two

  sig { returns(String) }
  attr_reader :string

  sig { returns(Hash) }
  attr_reader :deeper
end

# A custom configuration for testing.
class MyConfig < Optify::BaseConfig
  sig { returns(String) }
  attr_reader :rootString # rubocop:disable Naming/MethodName

  sig { returns(T::Array[String]) }
  attr_reader :myArray # rubocop:disable Naming/MethodName

  sig { returns(MyObject) }
  attr_reader :myObject # rubocop:disable Naming/MethodName

  sig { returns(T::Array[MyObject]) }
  attr_reader :myObjects # rubocop:disable Naming/MethodName
end
