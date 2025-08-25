import { defineConfig, globalIgnores } from "eslint/config";

export default defineConfig([
    globalIgnores(["**/**min.js", "**/highlight.js", "**/playground_editor/*"]),
    {
        rules: {
            indent: ["error", 4],
            "linebreak-style": ["error", "unix"],
            quotes: ["error", "single"],
            semi: ["error", "always"],

            "brace-style": ["error", "1tbs", {
                allowSingleLine: false,
            }],

            curly: "error",
            "no-trailing-spaces": "error",
            "no-multi-spaces": "error",

            "keyword-spacing": ["error", {
                before: true,
                after: true,
            }],

            "comma-spacing": ["error", {
                before: false,
                after: true,
            }],

            "arrow-spacing": ["error", {
                before: true,
                after: true,
            }],

            "key-spacing": ["error", {
                beforeColon: false,
                afterColon: true,
                mode: "strict",
            }],

            "func-call-spacing": ["error", "never"],
            "space-infix-ops": "error",
            "space-before-function-paren": ["error", "never"],
            "space-before-blocks": "error",

            "no-console": ["error", {
                allow: ["warn", "error"],
            }],

            "comma-dangle": ["error", "always-multiline"],
            "comma-style": ["error", "last"],

            "max-len": ["error", {
                code: 100,
                tabWidth: 2,
            }],

            "eol-last": ["error", "always"],
            "no-extra-parens": "error",
            "arrow-parens": ["error", "as-needed"],

            "no-unused-vars": ["error", {
                argsIgnorePattern: "^_",
                varsIgnorePattern: "^_",
            }],

            "prefer-const": ["error"],
            "no-var": "error",
            eqeqeq: "error",
        },
    },
]);
