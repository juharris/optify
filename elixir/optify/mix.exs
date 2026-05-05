defmodule Optify.MixProject do
  use Mix.Project

  @version "0.2.0"
  @source_url "https://github.com/juharris/optify"
  @hex_url "https://hex.pm/packages/optify"
  @docs_url "https://hexdocs.pm/optify"
  @issues_url "#{@source_url}/issues"

  def project do
    [
      app: :optify,
      version: @version,
      elixir: "~> 1.18",
      start_permanent: Mix.env() == :prod,
      deps: deps(),
      description: description(),
      package: package(),
      name: "Optify",
      source_url: @source_url,
      homepage_url: @source_url,
      docs: docs()
    ]
  end

  def application do
    [
      extra_applications: [:logger]
    ]
  end

  defp deps do
    [
      {:rustler, "~> 0.37.0"},
      {:jason, "~> 1.4"},
      {:ex_doc, "~> 0.35", only: :dev, runtime: false}
    ]
  end

  defp description do
    "Configuration options management powered by Rust NIFs. " <>
      "Merges feature-flagged JSON/YAML configs with deep-merge semantics."
  end

  defp package do
    [
      licenses: ["MIT"],
      links: %{
        "GitHub" => @source_url,
        "Hex" => @hex_url,
        "HexDocs" => @docs_url,
        "Issues" => @issues_url
      },
      files: ~w(
        lib
        native/optify_nif/src
        native/optify_nif/Cargo.toml
        native/optify_nif/rustfmt.toml
        .formatter.exs
        mix.exs
        README.md
        LICENSE.txt
      )
    ]
  end

  defp docs do
    [
      main: "readme",
      extras: ["README.md", "LICENSE.txt"],
      source_ref: "v#{@version}",
      source_url_pattern: "#{@source_url}/blob/v#{@version}/elixir/optify/%{path}#L%{line}"
    ]
  end
end
