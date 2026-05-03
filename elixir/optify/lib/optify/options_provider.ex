defmodule Optify.OptionsProvider do
  @moduledoc """
  Provides access to configuration options built from feature files.

  The provider loads and merges configuration files from one or more directories,
  then resolves the final options based on active feature names.
  """

  defstruct [:ref]

  def build(directory) do
    case Optify.Native.provider_build(directory) do
      {:ok, ref} -> {:ok, %__MODULE__{ref: ref}}
      {:error, _} = err -> err
      ref -> {:ok, %__MODULE__{ref: ref}}
    end
  end

  def build!(directory) do
    case build(directory) do
      {:ok, provider} -> provider
      {:error, reason} -> raise ArgumentError, reason
    end
  end

  def build_with_schema(directory, schema_path) do
    case Optify.Native.provider_build_with_schema(directory, schema_path) do
      {:ok, ref} -> {:ok, %__MODULE__{ref: ref}}
      {:error, _} = err -> err
      ref -> {:ok, %__MODULE__{ref: ref}}
    end
  end

  def build_with_schema!(directory, schema_path) do
    case build_with_schema(directory, schema_path) do
      {:ok, provider} -> provider
      {:error, reason} -> raise ArgumentError, reason
    end
  end

  def build_from_directories(directories) do
    case Optify.Native.provider_build_from_directories(directories) do
      {:ok, ref} -> {:ok, %__MODULE__{ref: ref}}
      {:error, _} = err -> err
      ref -> {:ok, %__MODULE__{ref: ref}}
    end
  end

  def build_from_directories!(directories) do
    case build_from_directories(directories) do
      {:ok, provider} -> provider
      {:error, reason} -> raise ArgumentError, reason
    end
  end

  def build_from_directories_with_schema(directories, schema_path) do
    case Optify.Native.provider_build_from_directories_with_schema(directories, schema_path) do
      {:ok, ref} -> {:ok, %__MODULE__{ref: ref}}
      {:error, _} = err -> err
      ref -> {:ok, %__MODULE__{ref: ref}}
    end
  end

  def build_from_directories_with_schema!(directories, schema_path) do
    case build_from_directories_with_schema(directories, schema_path) do
      {:ok, provider} -> provider
      {:error, reason} -> raise ArgumentError, reason
    end
  end

  def features(%__MODULE__{ref: ref}) do
    Optify.Native.provider_get_features(ref)
  end

  def aliases(%__MODULE__{ref: ref}) do
    Optify.Native.provider_get_aliases(ref)
  end

  def features_and_aliases(%__MODULE__{ref: ref}) do
    Optify.Native.provider_get_features_and_aliases(ref)
  end

  def get_canonical_feature_name(%__MODULE__{ref: ref}, feature_name) do
    Optify.Native.provider_get_canonical_feature_name(ref, feature_name)
  end

  def get_canonical_feature_name!(%__MODULE__{ref: ref}, feature_name) do
    case Optify.Native.provider_get_canonical_feature_name(ref, feature_name) do
      {:ok, name} -> name
      {:error, reason} -> raise ArgumentError, reason
      name when is_binary(name) -> name
    end
  end

  def get_canonical_feature_names(%__MODULE__{ref: ref}, feature_names) do
    Optify.Native.provider_get_canonical_feature_names(ref, feature_names)
  end

  def get_all_options(%__MODULE__{ref: ref}, feature_names, %Optify.GetOptionsPreferences{
        ref: prefs_ref
      }) do
    Optify.Native.provider_get_all_options(ref, feature_names, prefs_ref)
  end

  def get_all_options(%__MODULE__{} = provider, feature_names) do
    prefs = Optify.GetOptionsPreferences.new()
    get_all_options(provider, feature_names, prefs)
  end

  def get_options(%__MODULE__{ref: ref}, key, feature_names, %Optify.GetOptionsPreferences{
        ref: prefs_ref
      }) do
    Optify.Native.provider_get_options(ref, key, feature_names, prefs_ref)
  end

  def get_options(%__MODULE__{} = provider, key, feature_names) do
    prefs = Optify.GetOptionsPreferences.new()
    get_options(provider, key, feature_names, prefs)
  end

  def get_filtered_feature_names(
        %__MODULE__{ref: ref},
        feature_names,
        %Optify.GetOptionsPreferences{ref: prefs_ref}
      ) do
    Optify.Native.provider_get_filtered_feature_names(ref, feature_names, prefs_ref)
  end

  def get_filtered_feature_names(%__MODULE__{} = provider, feature_names) do
    prefs = Optify.GetOptionsPreferences.new()
    get_filtered_feature_names(provider, feature_names, prefs)
  end

  def map_feature_names(%__MODULE__{ref: ref}, feature_names, %Optify.GetOptionsPreferences{
        ref: prefs_ref
      }) do
    Optify.Native.provider_map_feature_names(ref, feature_names, prefs_ref)
  end

  def map_feature_names(%__MODULE__{} = provider, feature_names) do
    prefs = Optify.GetOptionsPreferences.new()
    map_feature_names(provider, feature_names, prefs)
  end

  def has_conditions?(%__MODULE__{ref: ref}, canonical_feature_name) do
    Optify.Native.provider_has_conditions(ref, canonical_feature_name)
  end
end
