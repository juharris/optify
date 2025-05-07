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

class TestConfig < Optify::BaseConfig
  sig { returns(T::Hash[String, Integer]) }
  attr_reader :hash

  sig { returns(T.nilable(TestObject)) }
  attr_reader :nilable_object

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

  sig { returns(T.nilable(T.any(String, TestObject))) }
  attr_reader :nilable_string_or_object

  sig { returns(T.nilable(T::Hash[String, T.any(String, TestObject)])) }
  attr_reader :nilable_hash_with_string_or_object
end

class FromHashTest < Test::Unit::TestCase
  def test_array_objects
    hash = { objects: [{ num: 5 }, { 'num' => 4 }] }
    m = TestConfig.from_hash(hash)
    assert_equal(2, m.objects.size)
    assert_instance_of(TestObject, m.objects[0])
    assert_instance_of(TestObject, m.objects[1])
    assert_equal(5, m.objects[0]&.num)
    assert_equal(4, m.objects[1]&.num)
  end

  def test_array_nilable_objects
    hash = { nilable_objects: [nil, { num: 42 }, nil, { "nilable_num": nil }, { "nilable_num": 44 }] }
    m = TestConfig.from_hash(hash)
    assert_equal(5, m.nilable_objects.size)
    assert_nil(m.nilable_objects[0])
    assert_instance_of(TestObject, m.nilable_objects[1])
    assert_nil(m.nilable_objects[2])
    assert_nil(m.nilable_objects[3]&.nilable_num)
    assert_equal(44, m.nilable_objects[4]&.nilable_num)
  end

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
    hash = { hash: { 'key' => 2 } }
    m = TestConfig.from_hash(hash)
    assert_equal({ 'key' => 2 }, m.hash)
    assert(m.hash.frozen?)
    assert_equal(2, m.hash['key'])
  end

  def test_from_hash_for_hash_with_object
    hash = { hash_with_object: { key: { num: 3 } } }
    m = TestConfig.from_hash(hash)
    o = m.hash_with_object[:key]
    assert_instance_of(TestObject, o)
    assert(o.frozen?)
    assert_equal(3, m.hash_with_object[:key]&.num)

    hash = { hash_of_hash_with_object: { 'key' => { key2: { 'num': 4 } } } }
    m = TestConfig.from_hash(hash)
    o = T.must(m.hash_of_hash_with_object['key'])[:key2]
    assert_instance_of(TestObject, o)
    assert(o.frozen?)
    assert_equal(4, o&.num)
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

    c = TestConfig.from_hash({ string_or_integer: 42 })
    assert_equal(42, c.string_or_integer)
  end

  def test_nilable_string_or_integer
    c = TestConfig.from_hash({ nilable_string_or_integer: 'hello' })
    assert_equal('hello', c.nilable_string_or_integer)

    c = TestConfig.from_hash({ nilable_string_or_integer: 42 })
    assert_equal(42, c.nilable_string_or_integer)

    c = TestConfig.from_hash({ nilable_string_or_integer: nil })
    assert_nil(c.nilable_string_or_integer)
  end

  def test_string_or_object
    c = TestConfig.from_hash({ string_or_object: { num: 42 } })
    assert_instance_of(TestObject, c.string_or_object)
    assert_equal(42, T.cast(c.string_or_object, TestObject).num)

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

  def test_nilable_hash_with_string_or_object
    c = TestConfig.from_hash({ nilable_hash_with_string_or_object: { 'string' => 'hello', 'object' => { num: 42 } } })
    h = T.must(c.nilable_hash_with_string_or_object)
    assert_equal('hello', h['string'])
    assert_equal(42, T.cast(h['object'], TestObject).num)
  end
end
