# Optify

Helps manage and develop [Optify] feature files.

![demo](https://raw.githubusercontent.com/juharris/optify/refs/heads/main/vscode/extension/assets/demo.gif)

## Features

* Show a preview of the built feature file.
* Click on an import to open the file.
* Validate imports.

### Future Plans

* Build a configuration for multiple features.
* Import completion/suggestions.
* Suggest the canonical name for an alias.
* See files that reference the current file.
* Syntax highlighting and coloring in the preview.
* Suggest keys in options based on other files.
* Email owners.

## Recommended Setup

See [here](https://github.com/juharris/optify?tab=readme-ov-file#schema-help) for how to enable schema help for Optify files.

Add a `.optify/` directory in the root of your folder containing your feature files.
I.e., the folder which you give to the builder in your application.
This is optional, but it will help the extension.
Otherwise, the extension will look for a parent folder called `configs`, `configurations`, or `options`.
You can place a file called `config.json` in the `.optify/` directory which may be used in the future.

## Release Notes

### Next
* Automatically refresh the preview when the file is saved.
* Don't associate all JSON and YAML files with the Optify schema.
* Ignore .optify folder when building provider.

### 0.2.2: Add gif

### 0.2.0: Publish for Cursor

* Drop VS Code required version to 1.96.0.

### 0.1.0: Initial release

* Show preview of built feature file.
* Click on an import to open the file.
* Validate imports.

## Development

See [CONTRIBUTING.md](./CONTRIBUTING.md) for details on how to contribute to this project.

[optify]: https://github.com/juharris/optify