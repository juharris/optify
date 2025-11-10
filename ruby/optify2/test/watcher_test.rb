# frozen_string_literal: true
# typed: true

require 'json'
require 'test/unit'
require 'tmpdir'
require_relative '../lib/optify'
require_relative 'my_config'

class OptifyWatcherTest < Test::Unit::TestCase
  def test_last_modified
    # Create a temporary directory for the test.
    temp_dir = Dir.mktmpdir
    temp_file = File.join(temp_dir, 'test.json')
    File.write(temp_file, JSON.dump({ 'options' => { 'myConfig' => { 'rootString' => 'value wtv' } } }))

    provider = Optify::OptionsWatcherBuilder.new
                                            .add_directory(temp_dir)
                                            .build
    last_modified = provider.last_modified
    assert_equal(last_modified, provider.last_modified)

    config_a = provider.get_options('myConfig', ['test'], MyConfig)
    assert_equal('value wtv', config_a.rootString)
    assert_equal(last_modified, provider.last_modified)

    File.write(temp_file, JSON.dump({ 'options' => { 'myConfig' => { 'rootString' => 'value changed' } } }))
    start_time = Time.now
    while provider.last_modified == last_modified
      sleep(0.1)
      raise 'Timeout waiting for last_modified to change' if Time.now - start_time > 3
    end
    assert_true(provider.last_modified > last_modified)
    last_modified = provider.last_modified

    assert_equal(last_modified, provider.last_modified)
    config_a = provider.get_options('myConfig', ['test'], MyConfig)
    assert_equal('value changed', config_a.rootString)
    assert_equal(last_modified, provider.last_modified)
  end

  def test_watcher_with_cache
    temp_dir = Dir.mktmpdir
    temp_file = File.join(temp_dir, 'test.json')
    File.write(temp_file, JSON.dump({ 'options' => { 'myConfig' => { 'rootString' => 'value wtv' } } }))

    provider = Optify::OptionsWatcherBuilder.new
                                            .add_directory(temp_dir)
                                            .build
                                            .init
    last_modified = provider.last_modified
    cache_options = Optify::CacheOptions.new
    config_a = provider.get_options('myConfig', ['test'], MyConfig, cache_options)
    assert_equal('value wtv', config_a.rootString)
    config_a2 = provider.get_options('myConfig', ['test'], MyConfig, cache_options)
    assert_same(config_a, config_a2)
    all_metadata = provider.features_with_metadata
    assert_equal(1, all_metadata.size)
    assert_same(all_metadata, provider.features_with_metadata)

    File.write(temp_file, JSON.dump({ 'options' => { 'myConfig' => { 'rootString' => 'value changed' } } }))
    start_time = Time.now
    while provider.last_modified == last_modified
      sleep(0.1)
      raise 'Timeout waiting for last_modified to change' if Time.now - start_time > 3
    end

    assert(provider.last_modified > last_modified)
    last_modified = provider.last_modified
    assert_not_same(all_metadata, provider.features_with_metadata)

    config_a = provider.get_options('myConfig', ['test'], MyConfig, cache_options)
    assert_not_same(config_a2, config_a)
    assert_equal('value changed', config_a.rootString)

    assert_equal(last_modified, provider.last_modified)
    config_a2 = provider.get_options('myConfig', ['test'], MyConfig, cache_options)
    assert_same(config_a, config_a2)
    assert_equal(last_modified, provider.last_modified)
  end
end
