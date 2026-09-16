# frozen_string_literal: true
# typed: true

require 'enummify'
require 'test/unit'
require_relative '../lib/optify-from_hash'

# Ensures that strings can be deserialized to enum values.
module EnumTest
  class TestEnum < Enummify::Enum
    ACTIVE = new
    INACTIVE = new
  end

  class TestKeyEnum < Enummify::Enum
    JOHN = new
    JANE = new
    AGE = new
  end

  class TestObject < Optify::FromHashable
    sig { returns(TestEnum) }
    attr_reader :enum
  end

  class TestConfig < Optify::FromHashable
    sig { returns(T::Hash[TestKeyEnum, T.any(TestEnum, Integer)]) }
    attr_reader :hash_with_enum_keys_and_enum_or_integer_values

    sig { returns(T::Hash[TestKeyEnum, TestEnum]) }
    attr_reader :hash_with_enum_keys_and_enum_values

    sig { returns(T::Hash[TestKeyEnum, T.nilable(TestEnum)]) }
    attr_reader :hash_with_enum_keys_and_nilable_enum_values

    sig { returns(T::Hash[TestKeyEnum, T.any(String, TestEnum)]) }
    attr_reader :hash_with_enum_keys_and_string_or_enum_values

    sig { returns(T::Hash[String, T.any(TestEnum, Integer)]) }
    attr_reader :hash_with_enum_or_integer_values

    sig { returns(T::Hash[String, TestEnum]) }
    attr_reader :hash_with_enum_values

    sig { returns(T::Hash[String, T.nilable(TestEnum)]) }
    attr_reader :hash_with_nilable_enum_values

    sig { returns(T::Hash[String, T.any(String, TestEnum)]) }
    attr_reader :hash_with_string_or_enum_values
  end

  class EnumTest < Test::Unit::TestCase
    def test_valid_enum
      object = TestObject.from_hash({ enum: 'ACTIVE' })
      assert_same(TestEnum::ACTIVE, object.enum)

      object = TestObject.from_hash({ enum: 'INACTIVE' })
      assert_same(TestEnum::INACTIVE, object.enum)
    end

    def test_invalid_enum
      error = assert_raises(ArgumentError) do
        TestObject.from_hash({ enum: 'not valid' })
      end
      assert_equal('Unknown EnumTest::TestEnum value: "not valid"', error.message)
    end

    def test_hash_with_enum_keys_and_enum_or_integer_values
      hash = { hash_with_enum_keys_and_enum_or_integer_values: { 'JOHN' => 'ACTIVE', 'AGE' => 42 } }
      m = TestConfig.from_hash(hash)
      assert_equal({ TestKeyEnum::JOHN => TestEnum::ACTIVE, TestKeyEnum::AGE => 42 }, m.hash_with_enum_keys_and_enum_or_integer_values)
      assert_equal(
        hash[:hash_with_enum_keys_and_enum_or_integer_values],
        m.to_h[:hash_with_enum_keys_and_enum_or_integer_values]
          .transform_keys(&:serialize)
          .transform_values { |v| v.respond_to?(:serialize) ? v.serialize : v }
      )
    end

    def test_hash_with_enum_keys_and_enum_values
      hash = { hash_with_enum_keys_and_enum_values: { 'JOHN' => 'ACTIVE', 'JANE' => 'INACTIVE' } }
      m = TestConfig.from_hash(hash)
      assert_equal({ TestKeyEnum::JOHN => TestEnum::ACTIVE, TestKeyEnum::JANE => TestEnum::INACTIVE }, m.hash_with_enum_keys_and_enum_values)
      assert_equal(
        hash[:hash_with_enum_keys_and_enum_values],
        m.to_h[:hash_with_enum_keys_and_enum_values].transform_keys(&:serialize).transform_values(&:serialize)
      )
    end

    def test_hash_with_enum_keys_and_nilable_enum_values
      hash = { hash_with_enum_keys_and_nilable_enum_values: { 'JOHN' => 'ACTIVE', 'JANE' => nil } }
      m = TestConfig.from_hash(hash)
      assert_equal({ TestKeyEnum::JOHN => TestEnum::ACTIVE, TestKeyEnum::JANE => nil }, m.hash_with_enum_keys_and_nilable_enum_values)
      assert_equal(
        hash[:hash_with_enum_keys_and_nilable_enum_values],
        m.to_h[:hash_with_enum_keys_and_nilable_enum_values]
          .transform_keys(&:serialize)
          .transform_values { |v| v&.serialize }
      )
    end

    def test_hash_with_enum_keys_and_string_or_enum_values
      hash = { hash_with_enum_keys_and_string_or_enum_values: { 'JOHN' => 'ACTIVE' } }
      m = TestConfig.from_hash(hash)
      assert_equal({ TestKeyEnum::JOHN => 'ACTIVE' }, m.hash_with_enum_keys_and_string_or_enum_values)
      assert_equal(
        hash[:hash_with_enum_keys_and_string_or_enum_values],
        m.to_h[:hash_with_enum_keys_and_string_or_enum_values].transform_keys(&:serialize)
      )

      hash = { hash_with_enum_keys_and_string_or_enum_values: { 'JOHN' => TestEnum::ACTIVE } }
      m = TestConfig.from_hash(hash)
      assert_equal({ TestKeyEnum::JOHN => TestEnum::ACTIVE }, m.hash_with_enum_keys_and_string_or_enum_values)
      assert_equal(
        { 'JOHN' => 'ACTIVE' },
        m.to_h[:hash_with_enum_keys_and_string_or_enum_values].transform_keys(&:serialize).transform_values(&:serialize)
      )
    end

    def test_hash_with_enum_or_integer_values
      hash = { hash_with_enum_or_integer_values: { 'john' => 'ACTIVE', 'age' => 42 } }
      m = TestConfig.from_hash(hash)
      assert_equal({ 'john' => TestEnum::ACTIVE, 'age' => 42 }, m.hash_with_enum_or_integer_values)
      assert_equal(
        hash[:hash_with_enum_or_integer_values],
        m.to_h[:hash_with_enum_or_integer_values].transform_values { |v| v.respond_to?(:serialize) ? v.serialize : v }
      )
    end

    def test_hash_with_enum_values
      hash = { hash_with_enum_values: { 'john' => 'ACTIVE', 'jane' => 'INACTIVE' } }
      m = TestConfig.from_hash(hash)
      assert_equal({ 'john' => TestEnum::ACTIVE, 'jane' => TestEnum::INACTIVE }, m.hash_with_enum_values)
      assert_equal(hash[:hash_with_enum_values], m.to_h[:hash_with_enum_values].transform_values(&:serialize))
    end

    def test_hash_with_nilable_enum_values
      hash = { hash_with_nilable_enum_values: { 'john' => 'ACTIVE', 'jane' => nil } }
      m = TestConfig.from_hash(hash)
      assert_equal({ 'john' => TestEnum::ACTIVE, 'jane' => nil }, m.hash_with_nilable_enum_values)
      assert_equal(
        hash[:hash_with_nilable_enum_values],
        m.to_h[:hash_with_nilable_enum_values].transform_values { |v| v&.serialize }
      )
    end

    def test_hash_with_string_or_enum_values
      hash = { hash_with_string_or_enum_values: { 'john' => 'ACTIVE' } }
      m = TestConfig.from_hash(hash)
      assert_equal(hash[:hash_with_string_or_enum_values], m.hash_with_string_or_enum_values)
      assert_equal(hash[:hash_with_string_or_enum_values], m.to_h[:hash_with_string_or_enum_values])

      hash = { hash_with_string_or_enum_values: { 'john' => TestEnum::ACTIVE } }
      m = TestConfig.from_hash(hash)
      assert_equal(hash[:hash_with_string_or_enum_values], m.hash_with_string_or_enum_values)
      assert_equal(hash[:hash_with_string_or_enum_values], m.to_h[:hash_with_string_or_enum_values])
    end
  end
end
