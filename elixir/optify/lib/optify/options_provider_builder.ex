defmodule Optify.OptionsProviderBuilder do
  @moduledoc """
  Builder for constructing an OptionsProvider from multiple directories.
  """

  defstruct [:ref]

  def new do
    %__MODULE__{ref: Optify.Native.provider_builder_new()}
  end

  def add_directory(%__MODULE__{ref: ref} = builder, directory) when is_binary(directory) do
    case Optify.Native.provider_builder_add_directory(ref, directory) do
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
    case Optify.Native.provider_builder_build(ref) do
      {:ok, provider_ref} -> {:ok, %Optify.OptionsProvider{ref: provider_ref}}
      {:error, _} = err -> err
      provider_ref -> {:ok, %Optify.OptionsProvider{ref: provider_ref}}
    end
  end

  def build!(%__MODULE__{} = builder) do
    case build(builder) do
      {:ok, provider} -> provider
      {:error, reason} -> raise ArgumentError, reason
    end
  end
end
