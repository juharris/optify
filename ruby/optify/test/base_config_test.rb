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

  sig { returns(T.nilable(Hash)) }
  attr_reader :hash
end

class BaseConfigTest < Test::Unit::TestCase
  def test_same
    a1 = TestObject.from_hash({ hash: { k: 1 } })
    assert_same(a1, a1)
    assert_equal(a1, a1)
  end

  def test_equality_hash
    a1 = TestObject.from_hash({ hash: { k: 1 } })
    a2 = TestObject.from_hash({ hash: { k: 1 } })
    assert_not_same(a1, a2)
    assert_equal(a1, a2)
    assert a1 == a2
  end

  def test_equality_numbers
    a1 = TestObject.from_hash({ num: 1 })
    a2 = TestObject.from_hash({ num: 1 })
    assert_same(a1, a1)
    assert_not_same(a1, a2)
    assert_equal(a1, a2)
    assert a1 == a2
  end

  def test_inequality_hash
    a1 = TestObject.from_hash({ hash: { k: 1 } })
    a2 = TestObject.from_hash({ hash: { k2: 2 } })
    assert_not_equal(a1, a2)
  end

  def test_inequality_numbers
    a1 = TestObject.from_hash({ num: 1 })
    a2 = TestObject.from_hash({ num: 2 })
    assert_not_same(a1, a2)
    assert_not_equal(a1, a2)
    assert a1 != a2
  end
end
