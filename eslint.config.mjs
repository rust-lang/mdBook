import { defineConfig, globalIgnores } from "eslint/config";

// Custom preprocessor to strip Handlebars templates.
const handlebarsPreprocessor = {
    processors: {
        "handlebars-js": {
            meta: {
                name: 'handlebars-js',
                version: '1.0.0',
            },
            preprocess(text, filename) {
                if (filename.endsWith('.hbs')) {
                    // Handle block expressions: {{#if ...}}...{{else}}...{{/if}}
                    // We keep the first branch and remove the else branch
                    let result = text;
                    // Remove {{else}}...{{/if}} (the else branch)
                    result = result.replace(/\{\{else\}\}[\s\S]*?\{\{\/if\}\}/g, '');
                    // Remove remaining {{#if ...}} and {{/if}} tags
                    result = result.replace(/\{\{#if[^}]*\}\}/g, '');
                    result = result.replace(/\{\{\/if\}\}/g, '');
                    // Remove simple Handlebars tags
                    result = result.replace(/\{\{[^}]*\}\}/g, '');
                    // Strip trailing whitespace from lines that only had handlebars tags
                    result = result.replace(/[ \t]+$/gm, '');
                    return [result];
                }
                return [text];
            },
            postprocess(messages, filename) {
                // Ideally this would update the locations so that they would
                // compensate for the removed ranges.
                return [].concat(...messages);
            },
        },
    },
};

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
    {
        files: ["**/*.js.hbs"],
        processor: handlebarsPreprocessor.processors["handlebars-js"],
    },
]);
