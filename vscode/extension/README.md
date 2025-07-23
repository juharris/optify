# Optify

Helps manage and develop [Optify] feature files.

![demo](https://raw.githubusercontent.com/juharris/optify/refs/heads/main/vscode/extension/assets/demo.gif)

## Features

* Show a preview of the built feature file or show the error in the preview window.
* Click on an import or use Go to Definition (F12) to open the file.
* Validate imports.
* Completions for imports.
* Suggest the canonical name for an alias.

### Future Plans

* Build a configuration for multiple features.
* See files that import the current file.
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

See all changes in [CHANGELOG.md](https://github.com/juharris/optify/blob/main/vscode/extension/CHANGELOG.md).

## Development

See [CONTRIBUTING.md](https://github.com/juharris/optify/blob/main/vscode/extension/CONTRIBUTING.md) for details on how to contribute to this project.

[optify]: https://github.com/juharris/optify