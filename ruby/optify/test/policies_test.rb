# typed: strict
# frozen_string_literal: true

require 'json'
require 'test/unit'
require 'optify'

class PoliciesTest < Test::Unit::TestCase
  PROVIDERS = [Optify::OptionsProvider, Optify::OptionsWatcher].freeze
  POLICIES_DIR = '../../tests/test_suites/policies/configs'

  #: -> void
  def test_get_policies_allowed
    PROVIDERS.each do |klass|
      provider = klass.build(POLICIES_DIR)
      result = provider.get_policies('feature_allowed') #: as !nil
      assert_not_nil(result, "Expected policies for feature_allowed from #{klass}")
      assert_instance_of(Optify::Policies, result)
      requester = result.requester #: as !nil
      assert_not_nil(requester)
      assert_equal(Set.new(%w[service_a service_b]), requester.allow,
                   "feature_allowed requester allow mismatch for #{klass}")
      assert_nil(requester.block)
    end
  end

  #: -> void
  def test_get_policies_blocked
    PROVIDERS.each do |klass|
      provider = klass.build(POLICIES_DIR)
      result = provider.get_policies('feature_blocked') #: as !nil
      assert_not_nil(result, "Expected policies for feature_blocked from #{klass}")
      assert_instance_of(Optify::Policies, result)
      requester = result.requester #: as !nil
      assert_not_nil(requester)
      assert_equal(Set.new(['untrusted_service']), requester.block,
                   "feature_blocked requester block mismatch for #{klass}")
      assert_nil(requester.allow)
    end
  end

  #: -> void
  def test_get_policies_missing
    PROVIDERS.each do |klass|
      provider = klass.build(POLICIES_DIR)
      assert_nil(provider.get_policies('nonexistent_feature'),
                 "Expected nil for unknown feature from #{klass}")
    end
  end

  #: -> void
  def test_requester_in_preferences
    preferences = Optify::GetOptionsPreferences.new
    assert_nil(preferences.requester)
    preferences.requester = 'service_a'
    assert_equal('service_a', preferences.requester)
  end

  #: -> void
  def test_raise_if_policy_denied_in_preferences_and_filtering
    PROVIDERS.each do |klass|
      provider = klass.build(POLICIES_DIR)
      preferences = Optify::GetOptionsPreferences.new
      assert_equal(false, preferences.raise_if_policy_denied)
      preferences.requester = 'unknown_service'
      preferences.raise_if_policy_denied = true
      assert_equal(true, preferences.raise_if_policy_denied)
      error = assert_raise(Optify::PolicyDeniedError) do
        provider.get_filtered_features(['feature_allowed'], preferences)
      end
      assert_equal(
        'Requester "unknown_service" is not permitted to use feature "feature_allowed".',
        error.message,
        "Error message mismatch for #{klass}",
      )
    end
  end

  #: -> void
  def test_check_policies
    PROVIDERS.each do |klass|
      provider = klass.build(POLICIES_DIR)

      # Allowed requester returns nil
      result = provider.check_policies('service_a', %w[feature_allowed feature_blocked])
      assert_nil(result, "Expected nil for allowed requester from #{klass}")

      exception = assert_raise(Optify::PolicyDeniedError) do
        provider.check_policies('untrusted_service', ['feature_blocked'])
      end
      assert_equal(
        'Requester "untrusted_service" is not permitted to use feature "feature_blocked".',
        exception.message,
        "Error message mismatch for #{klass}",
      )

      exception = assert_raise(Optify::PolicyDeniedError) do
        provider.check_policies('untrusted_service', %w[feature_allowed feature_blocked])
      end
      assert_equal(
        'Requester "untrusted_service" is not permitted to use feature "feature_allowed".',
        exception.message,
        "Error message mismatch for #{klass}",
      )

      exception = assert_raise(Optify::UnknownFeatureError) do
        provider.check_policies('untrusted_service', ['not a feature'])
      end
      assert_equal('Feature name "not a feature" is not a known feature.', exception.message, "Error message mismatch for #{klass}")
    end
  end

  #: -> void
  def test_check_policies_with_cache_allowed
    PROVIDERS.each do |klass|
      provider = klass.build(POLICIES_DIR)
      cache_events = []
      cache_options = Optify::CacheOptions.new(on_cache_event: lambda { |key, value, is_cache_hit|
        cache_events << { key: key, value: value, is_cache_hit: is_cache_hit }
      })

      # Allowed requester returns nil
      result = provider.check_policies('service_a', %w[feature_allowed feature_blocked], cache_options)
      assert_nil(result, "Expected nil for allowed requester from #{klass}")
      result2 = provider.check_policies('service_a', %w[feature_allowed feature_blocked], cache_options)
      assert_equal(result, result2)
      assert_equal(2, cache_events.length)
      assert_equal(
        { key: [:check_policies, %w[feature_allowed feature_blocked], 'service_a'], value: nil, is_cache_hit: false },
        cache_events[0],
        "Cache event mismatch for #{klass}",
      )
      assert_equal(
        { key: [:check_policies, %w[feature_allowed feature_blocked], 'service_a'], value: nil, is_cache_hit: true },
        cache_events[1],
        "Cache event mismatch for #{klass}",
      )
    end
  end

  #: -> void
  def test_check_policies_with_cache_denied
    PROVIDERS.each do |klass|
      provider = klass.build(POLICIES_DIR)
      cache_events = []
      cache_options = Optify::CacheOptions.new(on_cache_event: lambda { |key, value, is_cache_hit|
        cache_events << { key: key, value: value, is_cache_hit: is_cache_hit }
      })

      exception = assert_raise(Optify::PolicyDeniedError) do
        provider.check_policies('untrusted_service', ['feature_blocked'], cache_options)
      end
      assert_equal(
        'Requester "untrusted_service" is not permitted to use feature "feature_blocked".',
        exception.message,
        "Error message mismatch for #{klass}",
      )
      exception2 = assert_raise(Optify::PolicyDeniedError) do
        provider.check_policies('untrusted_service', ['feature_blocked'], cache_options)
      end

      # Don't cache exception because the stacktrace could be different
      # and we should only help in the happy path.
      assert_equal(0, cache_events.length)
      assert_equal(exception.message, exception2.message)
      assert_not_same(exception, exception2)
    end
  end
end
