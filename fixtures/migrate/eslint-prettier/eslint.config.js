import js from "@eslint/js";

export default [
  js.configs.recommended,
  {
    languageOptions: { globals: { console: "readonly" } },
    rules: {
      "no-var": "error",
      "prefer-const": "error",
      "no-console": ["warn", { allow: ["error"] }],
      eqeqeq: ["error", "always"],
      "no-alert": "error",
    },
  },
  {
    files: ["**/*.test.js"],
    rules: { "no-console": "off" },
  },
];
