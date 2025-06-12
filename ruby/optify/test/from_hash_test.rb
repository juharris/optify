# frozen_string_literal: true
# typed: true

require 'test/unit'
require_relative '../lib/optify'
require_relative 'my_config'

class TestObject < Optify::BaseConfig
  sig { returns(Integer) }
  attr_reader :num

  sig { returns(T.nilable(Integer)) }
  attr_reader :nilable_num
end

# An object with distinct properties from `TestObject`.
class TestObject2 < Optify::BaseConfig
  sig { returns(String) }
  attr_reader :string

  sig { returns(T::Hash[Symbol, TestObject]) }
  attr_reader :hash_symbol_to_object
end

class TestConfig < Optify::BaseConfig
  sig { returns(T::Array[T.any(String, TestObject)]) }
  attr_reader :array_of_string_or_object

  sig { returns(T::Hash[String, Integer]) }
  attr_reader :hash

  sig { returns(T.nilable(T::Hash[String, Integer])) }
  attr_reader :nilable_hash

  sig { returns(T.nilable(T::Hash[String, TestObject])) }
  attr_reader :nilable_hash_with_object

  sig { returns(T::Hash[Symbol, TestObject]) }
  attr_reader :hash_with_object

  sig { returns(T::Hash[String, T::Hash[Symbol, TestObject]]) }
  attr_reader :hash_of_hash_with_object

  sig { returns(T::Array[TestObject]) }
  attr_reader :objects

  sig { returns(T::Array[T::Hash[Symbol, TestObject]]) }
  attr_reader :hashes

  sig { returns(T::Hash[Symbol, Symbol]) }
  attr_reader :symbol_to_symbol

  sig { returns(Hash) }
  attr_reader :hash_with_no_types

  sig { returns(T::Hash[String, T.untyped]) }
  attr_reader :hash_with_untyped_values

  sig { returns(T::Array[T.nilable(T::Hash[Symbol, TestObject])]) }
  attr_reader :nilable_hashes_of_objects

  sig { returns(T::Array[T.nilable(TestObject)]) }
  attr_reader :nilable_objects

  sig { returns(T.any(String, Integer)) }
  attr_reader :string_or_integer

  sig { returns(T.nilable(T.any(String, Integer))) }
  attr_reader :nilable_string_or_integer

  sig { returns(T.any(String, TestObject)) }
  attr_reader :string_or_object

  sig { returns(T.any(String, TestObject, TestObject2)) }
  attr_reader :string_or_object_or_object2

  sig { returns(T.nilable(T.any(String, TestObject))) }
  attr_reader :nilable_string_or_object

  sig { returns(T.nilable(T::Hash[String, T.any(String, TestObject)])) }
  attr_reader :nilable_hash_with_string_or_object

  sig { returns(T.nilable(T::Hash[String, T.any(TestObject, String)])) }
  attr_reader :nilable_hash_with_object_or_string

  sig { returns(T.nilable(T::Hash[String, T.any(String, TestObject, TestObject2)])) }
  attr_reader :nilable_hash_with_string_or_object_or_object2

  sig { returns(T.untyped) }
  attr_reader :untyped
end

class FromHashTest < Test::Unit::TestCase
  def test_array_objects
    hash = { objects: [{ num: 5 }, { 'num' => 4 }] }
    m = TestConfig.from_hash(hash)
    assert_equal(2, m.objects.size)
    assert_equal(TestObject.from_hash({ num: 5 }), m.objects[0])
    assert_equal(TestObject.from_hash({ num: 4 }), m.objects[1])
    assert_equal(5, m.objects[0]&.num)
    assert_equal(4, m.objects[1]&.num)
  end

  def test_array_nilable_objects
    hash = { nilable_objects: [nil, { num: 243 }, nil, { "nilable_num": nil }, { "nilable_num": 44 }] }
    m = TestConfig.from_hash(hash)
    assert_equal(5, m.nilable_objects.size)
    assert_nil(m.nilable_objects[0])
    assert_equal(TestObject.from_hash({ num: 243 }), m.nilable_objects[1])
    assert_nil(m.nilable_objects[2])
    assert_nil(m.nilable_objects[3]&.nilable_num)
    assert_equal(TestObject.from_hash({ nilable_num: 44 }), m.nilable_objects[4])
  end

  def test_array_of_string_or_object
    hash = { array_of_string_or_object: ['hello', { num: 923_471 }] }
    c = TestConfig.from_hash(hash)
    assert_equal('hello', c.array_of_string_or_object[0])
    assert_instance_of(TestObject, c.array_of_string_or_object[1])
    assert_equal(923_471, T.cast(c.array_of_string_or_object[1], TestObject).num)
  end

  def test_from_hash_deep
    value = 'hello'
    hash = { 'rootString' => value, :myObject => { 'two' => 2 }, 'myObjects' => [{ two: 222 }] }
    m = MyConfig.from_hash(hash)
    assert_equal(value, m.rootString)
    exception = assert_raises(NoMethodError) do
      T.unsafe(m).rootString = 'wtv'
    end
    assert_match(/undefined method [`']rootString=' for an instance of MyConfig/, exception.message)

    assert_equal(2, m.myObject.two)
    assert_equal(222, m.myObjects[0]&.two)
  end

  def test_from_hash_with_hash
    hash = { hash: { 'key' => 2 } }
    m = TestConfig.from_hash(hash)
    assert_equal({ 'key' => 2 }, m.hash)
    assert(m.hash.frozen?)
    assert_equal(2, m.hash['key'])
  end

  def test_from_hash_for_hash_with_object
    hash = { hash_with_object: { 'key' => { num: 3 } } }
    m = TestConfig.from_hash(hash)
    o = m.hash_with_object[:key]
    assert_instance_of(TestObject, o)
    assert(o.frozen?)
    assert_equal(3, m.hash_with_object[:key]&.num)

    hash = { hash_of_hash_with_object: { 'key' => { 'key2' => { 'num': 4 } } } }
    m = TestConfig.from_hash(hash)
    o = T.must(m.hash_of_hash_with_object['key'])[:key2]
    assert_instance_of(TestObject, o)
    assert(o.frozen?)
    assert_equal(4, o&.num)
  end

  def test_hash_to_string
    exception = assert_raises(TypeError) do
      TestObject2.from_hash({ string: { 'key' => 'value' } })
    end
    assert_match(/Could not convert hash {"key" ?=> ?"value"} to `String`/, exception.message)
  end

  def test_hash_with_untyped_values
    m = TestConfig.from_hash({ hash_with_untyped_values: { 'key' => 'value', 'key2' => { 'num' => 4 }, 'num' => 5 } })
    assert_equal({ 'key' => 'value', 'key2' => { 'num' => 4 }, 'num' => 5 }, m.hash_with_untyped_values)
  end

  def test_hash_with_no_types
    hash = { hash_with_no_types: { 'key' => 'value' } }
    m = TestConfig.from_hash(hash)
    assert_equal({ 'key' => 'value' }, m.hash_with_no_types)
  end

  def test_hash_symbol_to_object
    hash = { 'string_or_object_or_object2' => { 'hash_symbol_to_object' => { 'key' => { 'num' => 4 } } } }
    c = TestConfig.from_hash(hash)
    hash_symbol_to_object = T.cast(c.string_or_object_or_object2, TestObject2).hash_symbol_to_object
    assert_instance_of(Hash, hash_symbol_to_object)
    assert_instance_of(TestObject, hash_symbol_to_object[:key])
    assert_equal(4, hash_symbol_to_object[:key]&.num)
  end

  def test_hashes
    hash = { hashes: [{ key: { num: 6 } }, { key2: { 'num' => 7 } }] }
    m = TestConfig.from_hash(hash)
    assert_equal(2, m.hashes.size)
    assert_instance_of(Hash, m.hashes[0])
    assert_instance_of(Hash, m.hashes[1])
    assert_instance_of(TestObject, T.must(m.hashes[0])[:key])
    assert_instance_of(TestObject, T.must(m.hashes[1])[:key2])
    assert_equal(6, m.hashes[0]&.fetch(:key)&.num)
    assert_equal(7, m.hashes[1]&.fetch(:key2)&.num)
  end

  def test_hashes_from_strings
    hash = { hashes: [{ 'key' => { 'num' => 6 } }, { 'key2' => { 'num' => 7 } }] }
    m = TestConfig.from_hash(hash)
    assert_equal(2, m.hashes.size)
    assert_instance_of(Hash, m.hashes[0])
    assert_instance_of(Hash, m.hashes[1])
    assert_instance_of(TestObject, T.must(m.hashes[0])[:key])
    assert_instance_of(TestObject, T.must(m.hashes[1])[:key2])
    assert_equal(6, m.hashes[0]&.fetch(:key)&.num)
    assert_equal(7, m.hashes[1]&.fetch(:key2)&.num)
  end

  def test_nilable_num
    hash = { nilable_num: nil }
    m = TestObject.from_hash(hash)
    assert_nil(m.nilable_num)

    hash = { nilable_num: 44 }
    m = TestObject.from_hash(hash)
    assert_equal(44, m.nilable_num)
  end

  def test_nilable_hash
    hash = { nilable_hash: nil }
    m = TestConfig.from_hash(hash)
    assert_nil(m.nilable_hash)

    hash = { nilable_hash: { key: 72 } }
    m = TestConfig.from_hash(hash)
    assert_instance_of(Hash, m.nilable_hash)
    assert_equal({ key: 72 }, m.nilable_hash)
  end

  def test_nilable_hash_with_object
    hash = { nilable_hash_with_object: nil }
    c = TestConfig.from_hash(hash)
    assert_nil(c.nilable_hash_with_object)

    hash = { "nilable_hash_with_object": { 'key' => { num: 8 } } }
    c = TestConfig.from_hash(hash)
    assert_instance_of(Hash, c.nilable_hash_with_object)
    obj = T.must(c.nilable_hash_with_object)['key']
    assert_instance_of(TestObject, obj)
    assert_equal(8, T.must(obj).num)
  end

  def test_num
    hash = { num: 33 }
    m = TestObject.from_hash(hash)
    assert_equal(33, m.num)
  end

  def test_string_or_integer
    c = TestConfig.from_hash({ string_or_integer: 'hello' })
    assert_equal('hello', c.string_or_integer)

    c = TestConfig.from_hash({ string_or_integer: 45_629 })
    assert_equal(45_629, c.string_or_integer)
  end

  def test_nilable_string_or_integer
    c = TestConfig.from_hash({ nilable_string_or_integer: 'hello' })
    assert_equal('hello', c.nilable_string_or_integer)

    c = TestConfig.from_hash({ nilable_string_or_integer: 312_449 })
    assert_equal(312_449, c.nilable_string_or_integer)

    c = TestConfig.from_hash({ nilable_string_or_integer: nil })
    assert_nil(c.nilable_string_or_integer)
  end

  def test_string_or_object
    c = TestConfig.from_hash({ string_or_object: { num: 871_102 } })
    assert_instance_of(TestObject, c.string_or_object)
    o = TestObject.from_hash({ num: 871_102 })
    assert_equal(o, c.string_or_object)

    c = TestConfig.from_hash({ string_or_object: 'hello' })
    assert_equal('hello', c.string_or_object)
  end

  def test_nilable_string_or_object
    c = TestConfig.from_hash({ nilable_string_or_object: 'hello' })
    assert_equal('hello', c.nilable_string_or_object)

    c = TestConfig.from_hash({ nilable_string_or_object: { num: 42 } })
    assert_instance_of(TestObject, c.nilable_string_or_object)
    assert_equal(42, T.cast(c.nilable_string_or_object, TestObject).num)
  end

  def test_string_or_object_or_object2
    c = TestConfig.from_hash({ string_or_object_or_object2: 'hello' })
    assert_equal('hello', c.string_or_object_or_object2)

    c = TestConfig.from_hash({ string_or_object_or_object2: { num: 42 } })
    assert_instance_of(TestObject, c.string_or_object_or_object2)
    assert_equal(42, T.cast(c.string_or_object_or_object2, TestObject).num)

    c = TestConfig.from_hash({ string_or_object_or_object2: { string: 'hello' } })
    assert_instance_of(TestObject2, c.string_or_object_or_object2)
    assert_equal('hello', T.cast(c.string_or_object_or_object2, TestObject2).string)
  end

  def test_nilable_hash_with_string_or_object
    c = TestConfig.from_hash({ nilable_hash_with_string_or_object: { 'string' => 'hello', 'object' => { num: 42 } } })
    h = T.must(c.nilable_hash_with_string_or_object)
    assert_equal('hello', h['string'])
    assert_equal(42, T.cast(h['object'], TestObject).num)
  end

  def test_nilable_hash_with_object_or_string
    c = TestConfig.from_hash({ nilable_hash_with_object_or_string: { 'string' => 'hello', 'object' => { num: 42 } } })
    h = T.must(c.nilable_hash_with_object_or_string)
    assert_equal('hello', h['string'])
    assert_equal(42, T.cast(h['object'], TestObject).num)
  end

  def test_nilable_hash_with_string_or_object_or_object2
    c = TestConfig.from_hash({ nilable_hash_with_string_or_object_or_object2: { 'string' => 'hello', 'object' => { num: 42 }, 'object2' => { string: 'hello2' }, 'nil' => nil } })
    h = T.must(c.nilable_hash_with_string_or_object_or_object2)
    assert_equal('hello', h['string'])
    assert_equal(42, T.cast(h['object'], TestObject).num)
    assert_equal('hello2', T.cast(h['object2'], TestObject2).string)
    assert_nil(h['nil'])
  end

  def test_nilable_hash_with_string_or_object_or_object2_invalid_object
    exception = assert_raises(TypeError) do
      TestConfig.from_hash({ nilable_hash_with_string_or_object_or_object2: { 'string' => { 'invalid key' => 'value' } } })
    end
    assert_match(/Could not convert hash: {"string" ?=> ?{"invalid key" ?=> ?"value"}} to T.nilable\(T::Hash\[String, T.any\(String, TestObject, TestObject2\)\]\)./,
                 exception.message)
  end

  # Skip for now because we don't validate primitive value types.
  def skip_test_nilable_hash_with_string_or_object_or_object2_invalid_value
    exception = assert_raises(TypeError) do
      TestConfig.from_hash({ nilable_hash_with_string_or_object_or_object2: { 'string' => 3 } })
    end
    assert_match(/Could not convert value: 3 to T.nilable\(T.any\(String, TestObject, TestObject2\)\)./, exception.message)
  end

  def test_symbol_to_symbol
    hash = { 'symbol_to_symbol' => { 'key' => 'value' } }
    c = TestConfig.from_hash(hash)
    assert_equal({ key: :value }, c.symbol_to_symbol)
  end

  def test_untyped
    c = TestConfig.from_hash({ untyped: { 'key' => 'value' } })
    assert_equal({ 'key' => 'value' }, c.untyped)

    c = TestConfig.from_hash({ untyped: 'hello' })
    assert_equal('hello', c.untyped)

    c = TestConfig.from_hash({ untyped: 42 })
    assert_equal(42, c.untyped)

    c = TestConfig.from_hash({ untyped: [1, 2, 3] })
    assert_equal([1, 2, 3], c.untyped)
  end
end
