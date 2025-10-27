#!/bin/bash
set -e

# major, minor, or patch
strategy=$1

# Go to the root directory of the project.
cd "$(dirname "$0")/.."

pushd vscode/extension
npm version ${strategy}
npm install @optify/config@latest
