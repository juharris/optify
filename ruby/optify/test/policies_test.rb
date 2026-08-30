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

      # Disallowed requester on feature_allowed returns error string
      result = provider.check_policies('untrusted_service', ['feature_allowed'])
      assert_equal(
        'Requester "untrusted_service" is not permitted to use feature "feature_allowed".',
        result,
        "Error message mismatch for #{klass}",
      )

      # Disallowed requester on feature_blocked returns error string
      result = provider.check_policies('untrusted_service', ['feature_blocked'])
      assert_equal(
        'Requester "untrusted_service" is not permitted to use feature "feature_blocked".',
        result,
        "Error message mismatch for #{klass}",
      )

      # Disallowed requester with multiple features returns error for first disallowed feature
      result = provider.check_policies('untrusted_service', %w[feature_allowed feature_blocked])
      assert_equal(
        'Requester "untrusted_service" is not permitted to use feature "feature_allowed".',
        result,
        "Error message mismatch for #{klass}",
      )

      # Nonexistent feature returns nil (no policies defined)
      result = provider.check_policies('untrusted_service', ['not a feature'])
      assert_equal('Feature name "not a feature" is not a known feature.', result, "Error message mismatch for #{klass}")
    end
  end
end
