import { defineConfig, globalIgnores } from "eslint/config";

// Custom preprocessor to strip Handlebars templates.
const handlebarsPreprocessor = {
    processors: {
        "handlebars-js": {
            preprocess(text, filename) {
                if (filename.endsWith('.hbs')) {
                    // This is a really dumb strip, which will likely not work
                    // for more complex expressions, but for our use is good
                    // enough for now.
                    return [text.replace(/\{\{.*?\}\}/g, '')];
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
