import { Bench } from "tinybench";
import path from "path";
import { fileURLToPath } from "url";

const __dirname = path.dirname(fileURLToPath(import.meta.url));

// Import the native module
const { OptionsProvider, GetOptionsPreferences } = await import(
  "../dist/index.js"
);

const configurableConfigsPath = path.join(
  __dirname,
  "../../../tests/test_suites/configurable_values/configs"
);

const provider = OptionsProvider.build(configurableConfigsPath);

const preferences = new GetOptionsPreferences();
preferences.enableConfigurableStrings();

const featureTrials = [
  ["simple"],
  ["simple", "imports"],
  ["imports_imports"],
  ["simple", "override_name"],
  ["simple", "raw_overrides"],
  ["simple", "with_files"],
  ["simple", "with_files_in_arguments"],
  ["simple", "complex_deep_merge"],
  ["simple", "complex_wide_structure"],
  [
    "simple",
    "complex_deep_merge",
    "complex_nested_objects",
    "complex_wide_structure",
  ],
];

const WARMUP_ITERATIONS = 20;
const ITERATIONS = 5000;

console.log(
  "Benchmarking getAllOptions vs JSON.parse(getAllOptionsJson(...))\n"
);
console.log("=".repeat(80));

for (const features of featureTrials) {
  const featureLabel = features.join(", ");

  // Warmup both methods
  for (let i = 0; i < WARMUP_ITERATIONS; ++i) {
    provider.getAllOptions(features, preferences);
    JSON.parse(provider.getAllOptionsJson(features, preferences));
  }

  const bench = new Bench({
    time: 3000,
    iterations: ITERATIONS,
  });

  bench
    .add(`getAllOptions`, () => {
      provider.getAllOptions(features, preferences);
    })
    .add(`JSON.parse(getAllOptionsJson)`, () => {
      JSON.parse(provider.getAllOptionsJson(features, preferences));
    });

  await bench.run();

  console.log(`\nFeatures: [${featureLabel}]`);
  console.log("-".repeat(80));
  console.table(bench.table());
}

console.log("\n" + "=".repeat(80));
console.log("Benchmark complete.");
