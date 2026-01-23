# typed: true
# frozen_string_literal: true

require 'json'
require 'optify'
require 'test/unit'

class BuilderErrorTest < Test::Unit::TestCase
  BUILDERS = [Optify::OptionsProviderBuilder, Optify::OptionsWatcherBuilder].freeze

  def test_builder_circular_imports
    BUILDERS.each do |klass|
      path = '../../rust/optify/tests/circular_imports'
      builder = klass.new
      err = assert_raise(RuntimeError) do
        builder.add_directory(path).build
      end
      # Check for key phrases in the error message
      assert_match(/Cycle detected with import/, err.message)
      assert_match(/Error when resolving imports/, err.message)
    end
  end

  def test_builder_cycle_in_imports
    BUILDERS.each do |klass|
      path = '../../rust/optify/tests/cycle_in_imports'
      builder = klass.new
      err = assert_raise(RuntimeError) do
        builder.add_directory(path).build
      end
      assert_match(/Cycle detected with import/, err.message)
      assert_match(/Error when resolving imports/, err.message)
    end
  end

  def test_builder_duplicate_alias
    BUILDERS.each do |klass|
      path = '../../rust/optify/tests/duplicate_alias'
      builder = klass.new
      err = assert_raise(ArgumentError, RuntimeError) do
        builder.add_directory(path).build
      end
      assert_match(/alias .* is already mapped/, err.message)
    end
  end

  def test_builder_invalid_file
    BUILDERS.each do |klass|
      path = '../../rust/optify/tests/invalid_file'
      builder = klass.new
      err = assert_raise(ArgumentError, RuntimeError) do
        builder.add_directory(path).build
      end
      assert_match(/Error loading file/, err.message)
    end
  end

  def test_builder_conditions_in_imported_feature
    BUILDERS.each do |klass|
      path = '../../tests/invalid_suites/conditions_in_import/configs'
      builder = klass.new
      err = assert_raise(RuntimeError) do
        builder.add_directory(path).build
      end
      assert_match(/Error when resolving imports for 'parent'/, err.message)
      assert_match(/Conditions cannot be used in imported features/, err.message)
    end
  end

  def test_builder_invalid_condition_pattern
    BUILDERS.each do |klass|
      path = '../../tests/invalid_suites/invalid_condition_pattern/configs'
      builder = klass.new
      err = assert_raise(ArgumentError, RuntimeError) do
        builder.add_directory(path).build
      end
      assert_match(/Error deserializing configuration/, err.message)
      assert_match(/regex parse error/, err.message)
    end
  end

  def test_builder_name_with_no_metadata
    BUILDERS.each do |klass|
      path = '../../rust/optify/tests/no_metadata'
      builder = klass.new
      builder.add_directory(path)
      provider = builder.build
      metadata = provider.get_feature_metadata('subdir/a') #: as !nil
      assert_not_nil(metadata)
      assert_equal('subdir/a', metadata.name)
      assert_nil(metadata.dependents)
      opts_json = provider.get_options_json('wtv', ['subdir/a'])
      # The result should be a JSON value, parse it to check
      parsed = JSON.parse(opts_json)
      assert_equal(3, parsed)
    end
  end

  def test_builder_used_canonical_alias
    BUILDERS.each do |klass|
      path = '../../rust/optify/tests/used_canonical_name'
      builder = klass.new
      err = assert_raise(ArgumentError, RuntimeError) do
        builder.add_directory(path).build
      end
      assert_match(/alias 'a' for canonical feature name 'a' is already mapped to 'a'/, err.message)
    end
  end

  def test_invalid_feature_name
    BUILDERS.each do |klass|
      provider = klass.new
                      .add_directory('../../tests/test_suites/simple/configs')
                      .build
      err = assert_raise(Optify::UnknownFeatureError) do
        provider.get_options_json('myConfig', ['nonexistent_feature'])
      end
      assert_equal('Feature name "nonexistent_feature" is not a known feature.', err.message)

      err = assert_raise(Optify::UnknownFeatureError) do
        provider.get_canonical_feature_name('nonexistent_feature')
      end
      assert_equal('Feature name "nonexistent_feature" is not a known feature.', err.message)
    end
  end

  def test_unknown_feature_error_is_raised_from_all_methods
    BUILDERS.each do |klass|
      provider = klass.new
                      .add_directory('../../tests/test_suites/simple/configs')
                      .build
      prefs = Optify::GetOptionsPreferences.new

      assert_raise(Optify::UnknownFeatureError) do
        provider.get_filtered_features(['nonexistent_feature'], prefs)
      end

      assert_raise(Optify::UnknownFeatureError) do
        provider.get_all_options_json(['nonexistent_feature'], prefs)
      end

      assert_raise(Optify::UnknownFeatureError) do
        provider.get_all_options_hash(['nonexistent_feature'], prefs)
      end

      assert_raise(Optify::UnknownFeatureError) do
        provider.get_options_hash('myConfig', ['nonexistent_feature'])
      end

      assert_raise(Optify::UnknownFeatureError) do
        provider.get_options_hash_with_preferences('myConfig', ['nonexistent_feature'], prefs)
      end

      assert_raise(Optify::UnknownFeatureError) do
        provider.get_options_json_with_preferences('myConfig', ['nonexistent_feature'], prefs)
      end
    end
  end

  def test_unknown_feature_error_inherits_from_standard_error
    assert(Optify::UnknownFeatureError < StandardError)
  end

  def test_get_options_with_invalid_key
    BUILDERS.each do |klass|
      provider = klass.new
                      .add_directory('../../tests/test_suites/simple/configs')
                      .build
      err = assert_raise(ArgumentError, RuntimeError) do
        provider.get_options_json('nonexistent_key', ['a'])
      end
      # The error message should indicate that the key was not found
      assert_match(/nonexistent_key/, err.message)
    end
  end
end
