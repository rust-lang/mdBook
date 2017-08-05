window.editors = [];
(function(editors) {
    if (typeof(ace) === 'undefined' || !ace) {
        return;
    }

    $(".editable").each(function() {
        let editor = ace.edit(this);
            editor.setOptions({
            highlightActiveLine: false,
            showPrintMargin: false,
            showLineNumbers: false,
            showGutter: false,
            maxLines: Infinity
        });

        editor.$blockScrolling = Infinity;

        editor.getSession().setMode("ace/mode/rust");

        editor.originalCode = editor.getValue();

        editors.push(editor);
    });
})(window.editors);
