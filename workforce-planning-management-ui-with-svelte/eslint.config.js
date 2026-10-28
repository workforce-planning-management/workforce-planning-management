// ESLint (flat config): TypeScript and Svelte rules, with Prettier owning formatting. `pnpm lint` runs this and `prettier --check`.
import js from "@eslint/js";
import prettier from "eslint-config-prettier";
import svelte from "eslint-plugin-svelte";
import globals from "globals";
import ts from "typescript-eslint";

export default ts.config(
  {
    ignores: [
      ".svelte-kit/",
      "build/",
      "node_modules/",
      "playwright-report/",
      "test-results/",
      "static/",
      "src/lib/api/schema.d.ts",
    ],
  },
  js.configs.recommended,
  ...ts.configs.recommended,
  ...svelte.configs["flat/recommended"],
  prettier,
  ...svelte.configs["flat/prettier"],
  {
    languageOptions: { globals: { ...globals.browser, ...globals.node } },
  },
  {
    files: ["**/*.svelte", "**/*.svelte.ts", "**/*.svelte.js"],
    languageOptions: { parserOptions: { parser: ts.parser } },
  },
  {
    // SvelteKit's own template declares these empty on purpose.
    files: ["src/app.d.ts"],
    rules: { "@typescript-eslint/no-empty-object-type": "off" },
  },
  {
    rules: {
      // Every link is built by `l()`, which puts the locale in the URL (WPM rule 5); SvelteKit's `resolve()` knows nothing of it.
      "svelte/no-navigation-without-resolve": "off",
      // A `Map` built inside `$derived` is a local lookup, not state that changes after it is made.
      "svelte/prefer-svelte-reactivity": "off",
      // `{" "}` keeps a deliberate space that Prettier would otherwise remove between two inline parts.
      "svelte/no-useless-mustaches": "off",
      // An unused name that starts with an underscore is deliberate.
      "@typescript-eslint/no-unused-vars": [
        "error",
        {
          argsIgnorePattern: "^_",
          varsIgnorePattern: "^_",
          caughtErrorsIgnorePattern: "^_",
        },
      ],
    },
  },
);
