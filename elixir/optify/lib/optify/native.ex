defmodule Optify.Native do
  @moduledoc false
  use Rustler, otp_app: :optify, crate: "optify_nif"

  # Provider Builder
  def provider_builder_new(), do: :erlang.nif_error(:nif_not_loaded)
  def provider_builder_add_directory(_builder, _directory), do: :erlang.nif_error(:nif_not_loaded)
  def provider_builder_build(_builder), do: :erlang.nif_error(:nif_not_loaded)

  # Provider Factory
  def provider_build(_directory), do: :erlang.nif_error(:nif_not_loaded)
  def provider_build_with_schema(_directory, _schema_path), do: :erlang.nif_error(:nif_not_loaded)
  def provider_build_from_directories(_directories), do: :erlang.nif_error(:nif_not_loaded)

  def provider_build_from_directories_with_schema(_directories, _schema_path),
    do: :erlang.nif_error(:nif_not_loaded)

  # Provider Queries
  def provider_get_features(_provider), do: :erlang.nif_error(:nif_not_loaded)
  def provider_get_aliases(_provider), do: :erlang.nif_error(:nif_not_loaded)
  def provider_get_features_and_aliases(_provider), do: :erlang.nif_error(:nif_not_loaded)

  def provider_get_canonical_feature_name(_provider, _name),
    do: :erlang.nif_error(:nif_not_loaded)

  def provider_get_canonical_feature_names(_provider, _names),
    do: :erlang.nif_error(:nif_not_loaded)

  # Provider Options
  def provider_get_all_options(_provider, _feature_names, _preferences),
    do: :erlang.nif_error(:nif_not_loaded)

  def provider_get_options(_provider, _key, _feature_names, _preferences),
    do: :erlang.nif_error(:nif_not_loaded)

  def provider_get_filtered_feature_names(_provider, _feature_names, _preferences),
    do: :erlang.nif_error(:nif_not_loaded)

  def provider_map_feature_names(_provider, _feature_names, _preferences),
    do: :erlang.nif_error(:nif_not_loaded)

  def provider_has_conditions(_provider, _canonical_feature_name),
    do: :erlang.nif_error(:nif_not_loaded)

  # Watcher Builder
  def watcher_builder_new(), do: :erlang.nif_error(:nif_not_loaded)
  def watcher_builder_add_directory(_builder, _directory), do: :erlang.nif_error(:nif_not_loaded)
  def watcher_builder_build(_builder), do: :erlang.nif_error(:nif_not_loaded)

  # Watcher Factory
  def watcher_build(_directory), do: :erlang.nif_error(:nif_not_loaded)
  def watcher_build_with_schema(_directory, _schema_path), do: :erlang.nif_error(:nif_not_loaded)
  def watcher_build_from_directories(_directories), do: :erlang.nif_error(:nif_not_loaded)

  def watcher_build_from_directories_with_schema(_directories, _schema_path),
    do: :erlang.nif_error(:nif_not_loaded)

  # Watcher Queries
  def watcher_get_features(_watcher), do: :erlang.nif_error(:nif_not_loaded)
  def watcher_get_aliases(_watcher), do: :erlang.nif_error(:nif_not_loaded)
  def watcher_get_features_and_aliases(_watcher), do: :erlang.nif_error(:nif_not_loaded)
  def watcher_get_canonical_feature_name(_watcher, _name), do: :erlang.nif_error(:nif_not_loaded)

  def watcher_get_canonical_feature_names(_watcher, _names),
    do: :erlang.nif_error(:nif_not_loaded)

  # Watcher Options
  def watcher_get_all_options(_watcher, _feature_names, _preferences),
    do: :erlang.nif_error(:nif_not_loaded)

  def watcher_get_options(_watcher, _key, _feature_names, _preferences),
    do: :erlang.nif_error(:nif_not_loaded)

  def watcher_get_filtered_feature_names(_watcher, _feature_names, _preferences),
    do: :erlang.nif_error(:nif_not_loaded)

  def watcher_map_feature_names(_watcher, _feature_names, _preferences),
    do: :erlang.nif_error(:nif_not_loaded)

  def watcher_has_conditions(_watcher, _canonical_feature_name),
    do: :erlang.nif_error(:nif_not_loaded)

  def watcher_last_modified(_watcher), do: :erlang.nif_error(:nif_not_loaded)

  # Preferences
  def preferences_new(), do: :erlang.nif_error(:nif_not_loaded)

  def preferences_are_configurable_strings_enabled(_preferences),
    do: :erlang.nif_error(:nif_not_loaded)

  def preferences_enable_configurable_strings(_preferences),
    do: :erlang.nif_error(:nif_not_loaded)

  def preferences_disable_configurable_strings(_preferences),
    do: :erlang.nif_error(:nif_not_loaded)

  def preferences_set_constraints(_preferences, _constraints),
    do: :erlang.nif_error(:nif_not_loaded)

  def preferences_set_constraints_json(_preferences, _json),
    do: :erlang.nif_error(:nif_not_loaded)

  def preferences_get_constraints(_preferences), do: :erlang.nif_error(:nif_not_loaded)
  def preferences_get_constraints_json(_preferences), do: :erlang.nif_error(:nif_not_loaded)

  def preferences_set_overrides(_preferences, _overrides), do: :erlang.nif_error(:nif_not_loaded)
  def preferences_set_overrides_json(_preferences, _json), do: :erlang.nif_error(:nif_not_loaded)
  def preferences_get_overrides_json(_preferences), do: :erlang.nif_error(:nif_not_loaded)

  def preferences_set_skip_feature_name_conversion(_preferences, _value),
    do: :erlang.nif_error(:nif_not_loaded)

  def preferences_get_skip_feature_name_conversion(_preferences),
    do: :erlang.nif_error(:nif_not_loaded)
end
