# Setup

(Optional) If `nvm` is used then activate the correct node version:

```shell
nvm use || nvm use `cat .nvmrc`
```

Install:

```shell
npm install
```

## Using Unreleased Changes in `@optify/config`

To try changes in [`js/optify-config`](../../js/optify-config/) before they are published, build the package and link it:

```shell
# Build the local package.
(cd ../../js/optify-config && yarn install && yarn build && yarn build:ts)

# Link it instead of the published package.
npm link ../../js/optify-config
```

`npm link` does not change `package.json` or `package-lock.json`, so do not commit anything for the link.
Rebuild the local package after changing it.
Restore the published package with `npm install`.

Features that depend on unreleased changes in `@optify/config` must wait for a release of `@optify/config` that includes them.
Then update the `@optify/config` dependency in `package.json` because CI and publishing use the published package.

# Starting

F5 in VS Code.

# Testing

```shell
npm run test
```

# Formatting & Linting

Check:

```shell
npm run lint
```

Fix:

```shell
npm run lint:fix
```

# Building

- Compile TypeScript: `npm run compile`
- Watch mode: `npm run watch`

# Publishing

## VS Code

Published to https://marketplace.visualstudio.com/items?itemName=optify-config.optify

A GitHub action will publish automatically.

Azure DevOps org: https://dev.azure.com/optify-config

Publisher: https://marketplace.visualstudio.com/manage/publishers/optify-config

## Open VSX

For Cursor use https://open-vsx.org/extension/optify-config/optify

A GitHub action will publish automatically.

Can publish manually at https://open-vsx.org/user-settings/extensions
