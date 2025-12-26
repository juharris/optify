# typed: true
# frozen_string_literal: true

require 'json'
require 'test/unit'
require 'tmpdir'
require 'optify'

class SchemaValidationTest < Test::Unit::TestCase
  def test_simple_configs_adhere_to_schema
    configs_dir = '../../tests/test_suites/simple/configs'
    schema_path = '../../schemas/feature_file.json'
    provider = Optify::OptionsWatcher.build_with_schema(configs_dir, schema_path)
    assert_not_nil(provider)
  end

  def test_schema_with_urns
    configs_dir = '../../tests/test_suites/inheritance/configs'
    schema_path = '../../tests/test_suites/inheritance/configs/.optify/schema.json'
    provider = Optify::OptionsProvider.build_with_schema(configs_dir, schema_path)
    assert_not_nil(provider)
  end

  def test_invalid_file_fails_schema_validation
    temp_dir = Dir.mktmpdir
    invalid_file_path = File.join(temp_dir, 'invalid.json')

    invalid_config = {
      invalidProperty: 'this property is not allowed by the schema',
    }
    File.write(invalid_file_path, JSON.dump(invalid_config))

    schema_path = '../../schemas/feature_file.json'

    err = assert_raise(RuntimeError) do
      Optify::OptionsWatcher.build_with_schema(temp_dir, schema_path)
    end
    assert_match(/Schema validation failed/, err.message)
  ensure
    FileUtils.rm_rf(temp_dir) if temp_dir
  end

  def test_schema_validation_with_provider_builder
    temp_dir = Dir.mktmpdir
    invalid_file_path = File.join(temp_dir, 'invalid.json')

    invalid_config = {
      invalidProperty: 'this property is not allowed by the schema',
    }
    File.write(invalid_file_path, JSON.dump(invalid_config))

    schema_path = '../../schemas/feature_file.json'

    err = assert_raise(RuntimeError) do
      Optify::OptionsProvider.build_with_schema(temp_dir, schema_path)
    end
    assert_match(/Schema validation failed/, err.message)
  ensure
    FileUtils.rm_rf(temp_dir) if temp_dir
  end
end
