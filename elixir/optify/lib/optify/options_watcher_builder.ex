defmodule Optify.OptionsWatcherBuilder do
  @moduledoc """
  Builder for constructing an OptionsWatcher from multiple directories.
  """

  defstruct [:ref]

  def new do
    %__MODULE__{ref: Optify.Native.watcher_builder_new()}
  end

  def add_directory(%__MODULE__{ref: ref} = builder, directory) when is_binary(directory) do
    case Optify.Native.watcher_builder_add_directory(ref, directory) do
      {:ok, _} -> {:ok, builder}
      {:error, _} = err -> err
      _ -> {:ok, builder}
    end
  end

  def add_directory!(%__MODULE__{} = builder, directory) do
    case add_directory(builder, directory) do
      {:ok, builder} -> builder
      {:error, reason} -> raise ArgumentError, reason
    end
  end

  def build(%__MODULE__{ref: ref}) do
    case Optify.Native.watcher_builder_build(ref) do
      {:ok, watcher_ref} -> {:ok, %Optify.OptionsWatcher{ref: watcher_ref}}
      {:error, _} = err -> err
      watcher_ref -> {:ok, %Optify.OptionsWatcher{ref: watcher_ref}}
    end
  end

  def build!(%__MODULE__{} = builder) do
    case build(builder) do
      {:ok, watcher} -> watcher
      {:error, reason} -> raise ArgumentError, reason
    end
  end
end
