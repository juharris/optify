defmodule Optify.GetOptionsPreferences do
  @moduledoc """
  Preferences for controlling how options are resolved.

  Allows setting constraints, overrides, configurable string processing,
  and feature name conversion behavior.
  """

  defstruct [:ref]

  def new do
    %__MODULE__{ref: Optify.Native.preferences_new()}
  end

  def are_configurable_strings_enabled?(%__MODULE__{ref: ref}) do
    Optify.Native.preferences_are_configurable_strings_enabled(ref)
  end

  def enable_configurable_strings(%__MODULE__{ref: ref} = prefs) do
    Optify.Native.preferences_enable_configurable_strings(ref)
    prefs
  end

  def disable_configurable_strings(%__MODULE__{ref: ref} = prefs) do
    Optify.Native.preferences_disable_configurable_strings(ref)
    prefs
  end

  def set_constraints(%__MODULE__{ref: ref} = prefs, constraints) when is_map(constraints) do
    Optify.Native.preferences_set_constraints(ref, constraints)
    prefs
  end

  def set_constraints_json(%__MODULE__{ref: ref} = prefs, constraints_json)
      when is_binary(constraints_json) do
    Optify.Native.preferences_set_constraints_json(ref, constraints_json)
    prefs
  end

  def get_constraints(%__MODULE__{ref: ref}) do
    Optify.Native.preferences_get_constraints(ref)
  end

  def get_constraints_json(%__MODULE__{ref: ref}) do
    Optify.Native.preferences_get_constraints_json(ref)
  end

  def set_overrides_json(%__MODULE__{ref: ref} = prefs, json) when is_binary(json) do
    Optify.Native.preferences_set_overrides_json(ref, json)
    prefs
  end

  def set_overrides(%__MODULE__{ref: ref} = prefs, overrides) when is_map(overrides) do
    Optify.Native.preferences_set_overrides(ref, overrides)
    prefs
  end

  def get_overrides_json(%__MODULE__{ref: ref}) do
    Optify.Native.preferences_get_overrides_json(ref)
  end

  def set_skip_feature_name_conversion(%__MODULE__{ref: ref} = prefs, value)
      when is_boolean(value) do
    Optify.Native.preferences_set_skip_feature_name_conversion(ref, value)
    prefs
  end

  def skip_feature_name_conversion?(%__MODULE__{ref: ref}) do
    Optify.Native.preferences_get_skip_feature_name_conversion(ref)
  end
end
