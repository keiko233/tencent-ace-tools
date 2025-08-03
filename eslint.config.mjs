// @ts-check
import path from "node:path";
import { fileURLToPath } from "node:url";
import { includeIgnoreFile } from "@eslint/compat";
import { FlatCompat } from "@eslint/eslintrc";
import eslintConfigPrettier from "eslint-config-prettier";
import eslintPluginPrettierRecommended from "eslint-plugin-prettier/recommended";
import react from "eslint-plugin-react";
import pluginReactCompiler from "eslint-plugin-react-compiler";
import pluginReactHooks from "eslint-plugin-react-hooks";
import globals from "globals";
import neostandard from "neostandard";
import tseslint from "typescript-eslint";

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
// eslint-disable-next-line @typescript-eslint/no-unused-vars
const compat = new FlatCompat({
  // import.meta.dirname is available after Node.js v20.11.0
  baseDirectory: __dirname,
});
const gitignorePath = path.resolve(__dirname, ".gitignore");

const ignores = [
  path.resolve(__dirname, "index.html"),
  "node_modules/",
  "dist/",
  "./src/paraglide/",
];

export default tseslint.config(
  includeIgnoreFile(gitignorePath),
  {
    ignores,
  },
  {
    files: ["**/*.{jsx,mjsx,tsx,mtsx}"],
    extends: [react.configs.flat.recommended],
    plugins: {
      "react-hooks": pluginReactHooks,
      "react-compiler": pluginReactCompiler,
    },
    settings: {
      react: {
        version: "detect",
      },
    },
    rules: {
      "react-hooks/rules-of-hooks": "error",
      "react-hooks/exhaustive-deps": "warn",
      "react-compiler/react-compiler": "warn",
    },
  },
  {
    files: ["**/*.{js,mjs,cjs,jsx,mjsx,ts,tsx,mtsx}"],
    extends: [
      ...neostandard({ ts: true, semi: true, noStyle: true }),
      eslintConfigPrettier,
      eslintPluginPrettierRecommended,
    ],
    plugins: {
      react: react,
    },
    rules: {
      "no-console": process.env.NODE_ENV === "production" ? "error" : "off",
      "no-debugger": process.env.NODE_ENV === "production" ? "error" : "off",
      "no-unused-vars": "off",
      "@typescript-eslint/no-unused-vars": "warn",
      "react/react-in-jsx-scope": "off",
      "prettier/prettier": ["error"],
      "react/no-unknown-property": ["error", { ignore: ["css"] }],
    },
  },
  {
    files: ["**/*.{ts,tsx,mtsx}"],
    extends: [...tseslint.configs.recommended],
    ignores,
    rules: {
      "@typescript-eslint/no-unused-vars": "warn",
      "@typescript-eslint/no-explicit-any": "warn",
    },
    languageOptions: {
      parserOptions: {
        project: true,
        projectService: true,
        tsconfigRootDir: import.meta.dirname,
      },
    },
  },
  {
    files: ["**/*.{jsx,mjsx,tsx,mtsx}"],
    languageOptions: {
      ...react.configs.flat?.recommended.languageOptions,
      globals: {
        ...globals.serviceworker,
        ...globals.browser,
      },
    },
  },
);
