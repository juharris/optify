# Optify

Helps manage and develop [Optify] feature files.

![demo](https://raw.githubusercontent.com/juharris/optify/refs/heads/main/vscode/extension/assets/demo.gif)

## Features

* Show a preview of the built feature file.
* Click on an import to open the file.
* Validate imports.

### Future Plans

* Syntax highlighting and coloring in the preview.
* Build a configuration for multiple features.

## Recommended Setup

See [here](https://github.com/juharris/optify?tab=readme-ov-file#schema-help) for how to enable schema help for Optify files.

Add a `.optify/` directory in the root of your folder containing your feature files.
I.e., the folder which you give to the builder in your application.
This is optional, but it will help the extension.
Otherwise, the extension will look for a parent folder called `configs`, `configurations`, or `options`.
You can place a file called `config.json` in the `.optify/` directory which may be used in the future.

## Release Notes

### Next
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