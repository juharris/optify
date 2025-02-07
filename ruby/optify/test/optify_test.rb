# frozen_string_literal: true
# typed: true

require 'json'
require 'test/unit'
require_relative '../lib/optify'
require_relative 'my_config'

require 'sorbet-runtime'

class OptifyTest < Test::Unit::TestCase
  extend T::Sig

  def test_empty_build
    builder = Optify::OptionsProviderBuilder.new
    provider = builder.build
    assert_not_nil(provider)
  end

  sig { params(suite_path: String).void }
  def run_suite(suite_path)
    puts "Running test suite: #{suite_path}"
    provider = Optify::OptionsProviderBuilder.new
                                             .add_directory(File.join(suite_path, 'configs'))
                                             .build
    expectations_path = File.join(suite_path, 'expectations')
    Dir.each_child(expectations_path) do |test_case|
      expectation_path = File.join(expectations_path, test_case)
      expected_info = JSON.parse(File.read(expectation_path))
      expected_options = expected_info['options']
      features = expected_info['features']
      expected_options.each do |key, expected_value|
        expected_json = provider.get_options_json(key, features)
        options = JSON.parse(expected_json, object_class: Hash)
        expected_json = expected_value.to_json
        expected_open_struct = JSON.parse(expected_json, object_class: Hash)
        assert_equal(expected_open_struct, options,
                     "Options for key \"#{key}\" with features #{features}
                     do not match for test suite at #{expectation_path}")
      end
    end
  end

  def test_suites
    test_suites_dir = '../../tests/test_suites'
    Dir.each_child(test_suites_dir) do |suite|
      suite_path = File.join(test_suites_dir, suite)
      next unless File.directory?(suite_path)

      run_suite(suite_path)
    end
  end

  def test_custom_config_class
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

  def test_custom_config_class2
    builder = Optify::OptionsProviderBuilder.new
                                            .add_directory('../../tests/test_suites/simple/configs')
    provider = builder.build
    config = provider.get_options('myConfig', ['A'], MyConfig)
    assert_equal('root string same', config.rootString)
    assert_equal(['example item 1'], config.myArray)
    assert_equal(2, config.myObject.two)
  end
end
