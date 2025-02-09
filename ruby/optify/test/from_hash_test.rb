# frozen_string_literal: true
# typed: true

require 'test/unit'
require_relative '../lib/optify'
require_relative 'my_config'

class TestObject < Optify::BaseConfig
  sig { returns(Integer) }
  attr_reader :num
end

class TestConfig < Optify::BaseConfig
  sig { returns(T::Hash[String, Integer]) }
  attr_reader :hash

  sig { returns(T::Hash[Symbol, TestObject]) }
  attr_reader :hash_with_object

  sig { returns(T::Hash[String, T::Hash[Symbol, TestObject]]) }
  attr_reader :hash_of_hash_with_object

  sig { returns(T::Array[TestObject]) }
  attr_reader :objects

  sig { returns(T::Array[T.nilable(TestObject)]) }
  attr_reader :nilable_objects

  sig { returns(T.nilable(TestObject)) }
  attr_reader :nilable_object
end

class FromHashTest < Test::Unit::TestCase
  def test_from_hash_deep
    value = 'hello'
    hash = { 'rootString' => value, :myObject => { 'two' => 2 }, 'myObjects' => [{ two: 222 }] }
    m = MyConfig.from_hash(hash)
    assert_equal(value, m.rootString)
    assert_raises(NoMethodError) do
      T.unsafe(m).rootString = 'wtv'
    end
    assert_equal(2, m.myObject.two)
    assert_equal(222, m.myObjects[0]&.two)
  end

  def test_from_hash_with_hash
    hash = { hash: { key: 2 } }
    m = TestConfig.from_hash(hash)
    assert_equal({ key: 2 }, m.hash)

    hash = { hash_with_object: { key: { num: 3 } } }
    m = TestConfig.from_hash(hash)
    assert_instance_of(TestObject, m.hash_with_object[:key])
    assert_equal(3, m.hash_with_object[:key]&.num)

    hash = { hash_of_hash_with_object: { 'key' => { key2: { 'num': 4 } } } }
    m = TestConfig.from_hash(hash)
    assert_instance_of(TestObject, T.must(m.hash_of_hash_with_object['key'])[:key2])
    assert_equal(4, T.must(m.hash_of_hash_with_object['key'])[:key2]&.num)

    # TODO: Add tests for the other cases with nilable.
  end
end
