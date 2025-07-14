# Change Log

<!--
All notable changes to the "Optify" extension will be documented in this file.
Check [Keep a Changelog](http://keepachangelog.com/) for recommendations on how to structure this file.
-->

## 1.2.1

Optimization: Use OptionsWatcher to more natively watch for changes and cache the OptionsProvider.

## 1.2.0

* Enable find in preview.
* Bump to Optify to 0.5.0 to read conditions in files.
* Preview while editing: add 250ms edit debounce
* Optimize parsing imports

## 1.1.0

* Change preview while editing.
* Color JSON in preview.

## 1.0.1
* Only publish for Mac.

## 1.0.0

* Enable using Go to Definition (F12) to open the file for an import.
* Show the build error in the preview window.
* Add error if a file tries to import itself.

## 0.3.0:

* Automatically refresh the preview when the file is saved.
* Don't associate all JSON and YAML files with the Optify schema.
* Ignore .optify folder when building provider.

## 0.2.2: Add gif

## 0.2.0: Publish for Cursor

* Drop VS Code required version to 1.96.0.

## 0.1.0: Initial release

* Show preview of built feature file.
* Click on an import to open the file.
* Validate imports.
