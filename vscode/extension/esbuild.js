const esbuild = require("esbuild");

const watch = process.argv.includes("--watch");

const buildWebview = async () => {
  const ctx = await esbuild.context({
    entryPoints: ["src/webview/index.tsx"],
    bundle: true,
    outfile: "out/webview.js",
    platform: "browser",
    format: "iife",
    target: "es2020",
    minify: !watch,
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

buildWebview().catch((error) => {
  console.error("Failed to build webview:", error);
  process.exit(1);
});
