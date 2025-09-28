# frozen_string_literal: true
# typed: true

require 'test/unit'
require_relative '../lib/optify'
require_relative 'my_config'

module BaseConfigTest
  class TestObject < Optify::BaseConfig
    sig { returns(T.nilable(Integer)) }
    attr_reader :num

    sig { returns(T.nilable(Hash)) }
    attr_reader :hash
  end

  class TestObjectWithObject < Optify::BaseConfig
    sig { returns(TestObject) }
    attr_reader :object
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

      assert_equal(1, a1.num)
      assert_equal(1, a2.num)
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

    def test_unknown_property_symbol_key
      err = assert_raise(ArgumentError) do
        TestObject.from_hash({ bad: 1 })
      end
      assert_equal(
        'Error converting hash to `BaseConfigTest::TestObject` because of key "bad". Perhaps "bad" is not a valid attribute for `BaseConfigTest::TestObject`.',
        err.message
      )

      assert_instance_of(NameError, err.cause)
      assert_equal('undefined method \'bad\' for class \'BaseConfigTest::TestObject\'', err.cause.message)
    end

    def test_unknown_property_in_object
      err = assert_raise(ArgumentError) do
        TestObjectWithObject.from_hash({ object: { bad: 2 } })
      end
      assert_equal(
        'Error converting hash to `BaseConfigTest::TestObject` because of key "bad". Perhaps "bad" is not a valid attribute for `BaseConfigTest::TestObject`.',
        err.message
      )

      assert_instance_of(NameError, err.cause)
      assert_equal('undefined method \'bad\' for class \'BaseConfigTest::TestObject\'', err.cause.message)
    end
  end
end
