const esbuild = require("esbuild");

const watch = process.argv.includes("--watch");
const production = !watch;

const buildExtension = async () => {
  const ctx = await esbuild.context({
    entryPoints: ["src/extension.ts"],
    bundle: true,
    outfile: "out/extension.js",
    platform: "node",
    format: "cjs",
    target: "node20",
    minify: production,
    sourcemap: watch ? "inline" : false,
    external: ["vscode", "@optify/config"],
    logLevel: "info",
  });

  if (watch) {
    await ctx.watch();
    console.log("Watching extension...");
  } else {
    await ctx.rebuild();
    await ctx.dispose();
    console.log("Extension built successfully");
  }
};

const buildWebview = async () => {
  const ctx = await esbuild.context({
    entryPoints: ["src/webview/index.tsx"],
    bundle: true,
    outfile: "out/webview.js",
    platform: "browser",
    format: "iife",
    target: "es2020",
    minify: production,
    sourcemap: watch ? "inline" : false,
    loader: {
      ".tsx": "tsx",
      ".ts": "ts",
    },
    define: {
      "process.env.NODE_ENV": watch ? '"development"' : '"production"',
    },
    logLevel: "info",
  });

  if (watch) {
    await ctx.watch();
    console.log("Watching webview...");
  } else {
    await ctx.rebuild();
    await ctx.dispose();
    console.log("Webview built successfully");
  }
};

Promise.all([buildExtension(), buildWebview()]).catch((error) => {
  console.error("Failed to build:", error);
  process.exit(1);
});
