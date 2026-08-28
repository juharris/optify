# Contributing

## Markdown Formatting

Markdown files are formatted with [Prettier](https://prettier.io) and [`prettier-plugin-sentences-per-line`](https://www.npmjs.com/package/prettier-plugin-sentences-per-line) to enforce one sentence per line, which keeps diffs small and easy to review.

Install dependencies with [pnpm](https://pnpm.io):

```shell
corepack enable
pnpm install
```

Check formatting:

```shell
pnpm run fmt:check
```

Apply formatting:

```shell
pnpm run fmt
```

CI runs `pnpm run fmt:check` on pushes and pull requests that touch Markdown files (see `.github/workflows/markdown_format.yml`).

For language-specific contribution instructions, see the `README.md` and `CONTRIBUTING.md` files (where present) in each language's folder, e.g. [`vscode/extension/CONTRIBUTING.md`](./vscode/extension/CONTRIBUTING.md).
