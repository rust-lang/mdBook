// hljs-example.js
hljs.registerLanguage('example', function(hljs) {
    return {
        keywords: 'print',
        contains: [hljs.QUOTE_STRING_MODE],
    };
});
