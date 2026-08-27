# typed: true
# frozen_string_literal: true

require 'json'
require 'test/unit'
require 'optify'

class PoliciesTest < Test::Unit::TestCase
  PROVIDERS = [Optify::OptionsProvider, Optify::OptionsWatcher].freeze
  POLICIES_DIR = '../../tests/test_suites/policies/configs'

  def test_get_policies_allowed
    PROVIDERS.each do |klass|
      provider = klass.build(POLICIES_DIR)
      result = provider.get_policies('feature_allowed')
      assert_not_nil(result, "Expected policies for feature_allowed from #{klass}")
      requester = result['requester']
      requester['allowed']&.sort!
      assert_equal({ 'allowed' => %w[service_a service_b] }, requester,
                   "feature_allowed requester policies mismatch for #{klass}")
    end
  end

  def test_get_policies_blocked
    PROVIDERS.each do |klass|
      provider = klass.build(POLICIES_DIR)
      result = provider.get_policies('feature_blocked')
      assert_not_nil(result, "Expected policies for feature_blocked from #{klass}")
      assert_equal({ 'blocked' => ['untrusted_service'] }, result['requester'],
                   "feature_blocked requester policies mismatch for #{klass}")
    end
  end

  def test_get_policies_missing
    PROVIDERS.each do |klass|
      provider = klass.build(POLICIES_DIR)
      assert_nil(provider.get_policies('nonexistent_feature'),
                 "Expected nil for unknown feature from #{klass}")
    end
  end

  def test_requester_in_preferences
    preferences = Optify::GetOptionsPreferences.new
    assert_nil(preferences.requester)
    preferences.requester = 'service_a'
    assert_equal('service_a', preferences.requester)
  end

  def test_raise_if_policy_denied_in_preferences
    preferences = Optify::GetOptionsPreferences.new
    assert_equal(false, preferences.raise_if_policy_denied)
    preferences.raise_if_policy_denied = true
    assert_equal(true, preferences.raise_if_policy_denied)
  end

  def test_filtering_removes_denied_requester
    PROVIDERS.each do |klass|
      provider = klass.build(POLICIES_DIR)
      preferences = Optify::GetOptionsPreferences.new
      # "unknown_service" is not in the allowed list for feature_allowed, so it gets filtered out.
      # It is also not in the blocked list for feature_blocked, so feature_blocked is kept.
      preferences.requester = 'unknown_service'
      result = provider.get_filtered_features(%w[feature_allowed feature_blocked], preferences)
      assert_equal(['feature_blocked'], result, "Denied feature should be filtered out for #{klass}")
    end
  end

  def test_policy_denied_raises_when_requested
    PROVIDERS.each do |klass|
      provider = klass.build(POLICIES_DIR)
      preferences = Optify::GetOptionsPreferences.new
      preferences.requester = 'unknown_service'
      preferences.raise_if_policy_denied = true
      assert_raise(Optify::PolicyDeniedError) do
        provider.get_filtered_features(['feature_allowed'], preferences)
      end
    end
  end
end
