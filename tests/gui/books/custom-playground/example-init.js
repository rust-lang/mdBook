"use strict";

// Replace with a call to your runtime.
function example_language_run(source) {
    return "ran: " + source.trim();
}

function wrapInPlaygroundPre(codeEl) {
    const pre = codeEl.parentNode;
    pre.classList.add("playground");
    codeEl.classList.remove("playground");
    return pre;
}

function ensureResultBlock(preBlock) {
    let next = preBlock.nextElementSibling;
    if (!(next && next.classList.contains("example-result"))) {
        next = document.createElement("pre");
        next.className = "example-result";
        next.appendChild(document.createElement("code"));
        preBlock.insertAdjacentElement("afterend", next);
    }
    return next.querySelector("code");
}

function runExample(preBlock, codeEl, editor) {
    const resultCode = ensureResultBlock(preBlock);
    const source = editor ? editor.getValue() : codeEl.textContent;
    try {
        resultCode.textContent = example_language_run(source);
    } catch (err) {
        resultCode.textContent = String(err);
    }
}

function addButton(buttons, className, title, iconId, onClick) {
    const button = document.createElement("button");
    button.className = className;
    button.type = "button";
    button.title = title;
    button.setAttribute("aria-label", title);
    button.innerHTML = document.getElementById(iconId).innerHTML;
    button.addEventListener("click", onClick);
    buttons.insertBefore(button, buttons.firstChild);
}

function addPlaygroundButtons(preBlock, codeEl, editor) {
    let buttons = preBlock.querySelector(":scope > .buttons");
    if (!buttons) {
        buttons = document.createElement("div");
        buttons.className = "buttons";
        preBlock.insertBefore(buttons, preBlock.firstChild);
    }
    addButton(buttons, "play-button", "Run this code", "fa-play",
        () => runExample(preBlock, codeEl, editor));
    if (editor) {
        addButton(buttons, "reset-button", "Undo changes", "fa-clock-rotate-left", () => {
            editor.setValue(editor.originalCode);
            editor.clearSelection();
        });
    }
}

function editorFor(codeEl) {
    return (window.editors || []).find(e => e.container === codeEl) || null;
}

window.addEventListener("load", () => {
    document.querySelectorAll("code.playground").forEach(codeEl => {
        const preBlock = wrapInPlaygroundPre(codeEl);
        addPlaygroundButtons(preBlock, codeEl, editorFor(codeEl));
    });
});
