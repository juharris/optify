defmodule Optify.OptionsWatcher do
  @moduledoc """
  A file-watching variant of OptionsProvider that automatically reloads
  configuration when files change on disk.
  """

  defstruct [:ref]

  def build(directory) when is_binary(directory) do
    case Optify.Native.watcher_build(directory) do
      {:ok, ref} -> {:ok, %__MODULE__{ref: ref}}
      {:error, _} = err -> err
      ref -> {:ok, %__MODULE__{ref: ref}}
    end
  end

  def build!(directory) do
    case build(directory) do
      {:ok, watcher} -> watcher
      {:error, reason} -> raise ArgumentError, reason
    end
  end

  def build_with_schema(directory, schema_path)
      when is_binary(directory) and is_binary(schema_path) do
    case Optify.Native.watcher_build_with_schema(directory, schema_path) do
      {:ok, ref} -> {:ok, %__MODULE__{ref: ref}}
      {:error, _} = err -> err
      ref -> {:ok, %__MODULE__{ref: ref}}
    end
  end

  def build_with_schema!(directory, schema_path) do
    case build_with_schema(directory, schema_path) do
      {:ok, watcher} -> watcher
      {:error, reason} -> raise ArgumentError, reason
    end
  end

  def build_from_directories(directories) when is_list(directories) do
    case Optify.Native.watcher_build_from_directories(directories) do
      {:ok, ref} -> {:ok, %__MODULE__{ref: ref}}
      {:error, _} = err -> err
      ref -> {:ok, %__MODULE__{ref: ref}}
    end
  end

  def build_from_directories!(directories) do
    case build_from_directories(directories) do
      {:ok, watcher} -> watcher
      {:error, reason} -> raise ArgumentError, reason
    end
  end

  def build_from_directories_with_schema(directories, schema_path)
      when is_list(directories) and is_binary(schema_path) do
    case Optify.Native.watcher_build_from_directories_with_schema(directories, schema_path) do
      {:ok, ref} -> {:ok, %__MODULE__{ref: ref}}
      {:error, _} = err -> err
      ref -> {:ok, %__MODULE__{ref: ref}}
    end
  end

  def build_from_directories_with_schema!(directories, schema_path) do
    case build_from_directories_with_schema(directories, schema_path) do
      {:ok, watcher} -> watcher
      {:error, reason} -> raise ArgumentError, reason
    end
  end

  def features(%__MODULE__{ref: ref}) do
    Optify.Native.watcher_get_features(ref)
  end

  def aliases(%__MODULE__{ref: ref}) do
    Optify.Native.watcher_get_aliases(ref)
  end

  def features_and_aliases(%__MODULE__{ref: ref}) do
    Optify.Native.watcher_get_features_and_aliases(ref)
  end

  def get_canonical_feature_name(%__MODULE__{ref: ref}, feature_name)
      when is_binary(feature_name) do
    Optify.Native.watcher_get_canonical_feature_name(ref, feature_name)
  end

  def get_canonical_feature_name!(%__MODULE__{ref: ref}, feature_name)
      when is_binary(feature_name) do
    case Optify.Native.watcher_get_canonical_feature_name(ref, feature_name) do
      {:ok, name} -> name
      {:error, reason} -> raise ArgumentError, reason
      name when is_binary(name) -> name
    end
  end

  def get_canonical_feature_names(%__MODULE__{ref: ref}, feature_names)
      when is_list(feature_names) do
    Optify.Native.watcher_get_canonical_feature_names(ref, feature_names)
  end

  def get_all_options(%__MODULE__{ref: ref}, feature_names, %Optify.GetOptionsPreferences{
        ref: prefs_ref
      })
      when is_list(feature_names) do
    Optify.Native.watcher_get_all_options(ref, feature_names, prefs_ref)
  end

  def get_all_options(%__MODULE__{} = watcher, feature_names) do
    prefs = Optify.GetOptionsPreferences.new()
    get_all_options(watcher, feature_names, prefs)
  end

  def get_options(%__MODULE__{ref: ref}, key, feature_names, %Optify.GetOptionsPreferences{
        ref: prefs_ref
      })
      when is_binary(key) and is_list(feature_names) do
    Optify.Native.watcher_get_options(ref, key, feature_names, prefs_ref)
  end

  def get_options(%__MODULE__{} = watcher, key, feature_names) do
    prefs = Optify.GetOptionsPreferences.new()
    get_options(watcher, key, feature_names, prefs)
  end

  def get_filtered_feature_names(
        %__MODULE__{ref: ref},
        feature_names,
        %Optify.GetOptionsPreferences{ref: prefs_ref}
      )
      when is_list(feature_names) do
    Optify.Native.watcher_get_filtered_feature_names(ref, feature_names, prefs_ref)
  end

  def get_filtered_feature_names(%__MODULE__{} = watcher, feature_names) do
    prefs = Optify.GetOptionsPreferences.new()
    get_filtered_feature_names(watcher, feature_names, prefs)
  end

  def map_feature_names(%__MODULE__{ref: ref}, feature_names, %Optify.GetOptionsPreferences{
        ref: prefs_ref
      })
      when is_list(feature_names) do
    Optify.Native.watcher_map_feature_names(ref, feature_names, prefs_ref)
  end

  def map_feature_names(%__MODULE__{} = watcher, feature_names) do
    prefs = Optify.GetOptionsPreferences.new()
    map_feature_names(watcher, feature_names, prefs)
  end

  def has_conditions?(%__MODULE__{ref: ref}, canonical_feature_name)
      when is_binary(canonical_feature_name) do
    Optify.Native.watcher_has_conditions(ref, canonical_feature_name)
  end

  def last_modified(%__MODULE__{ref: ref}) do
    Optify.Native.watcher_last_modified(ref)
  end
end
