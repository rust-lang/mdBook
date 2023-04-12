"use strict";

// Fix back button cache problem
window.onunload = function () { };

// Global variable, shared between modules
function playground_text(playground, hidden = true) {
    let code_block = playground.querySelector("code");

    if (window.ace && code_block.classList.contains("editable")) {
        let editor = window.ace.edit(code_block);
        return editor.getValue();
    } else if (hidden) {
        return code_block.textContent;
    } else {
        return code_block.innerText;
    }
}

(function codeSnippets() {
    function fetch_with_timeout(url, options, timeout = 6000) {
        return Promise.race([
            fetch(url, options),
            new Promise((_, reject) => setTimeout(() => reject(new Error('timeout')), timeout))
        ]);
    }

    var playgrounds = Array.from(document.querySelectorAll(".playground"));
    if (playgrounds.length > 0) {
        fetch_with_timeout("https://play.rust-lang.org/meta/crates", {
            headers: {
                'Content-Type': "application/json",
            },
            method: 'POST',
            mode: 'cors',
        })
        .then(response => response.json())
        .then(response => {
            // get list of crates available in the rust playground
            let playground_crates = response.crates.map(item => item["id"]);
            playgrounds.forEach(block => handle_crate_list_update(block, playground_crates));
        });
    }

    function handle_crate_list_update(playground_block, playground_crates) {
        // update the play buttons after receiving the response
        update_play_button(playground_block, playground_crates);

        // and install on change listener to dynamically update ACE editors
        if (window.ace) {
            let code_block = playground_block.querySelector("code");
            if (code_block.classList.contains("editable")) {
                let editor = window.ace.edit(code_block);
                editor.addEventListener("change", function (e) {
                    update_play_button(playground_block, playground_crates);
                });
                // add Ctrl-Enter command to execute rust code
                editor.commands.addCommand({
                    name: "run",
                    bindKey: {
                        win: "Ctrl-Enter",
                        mac: "Ctrl-Enter"
                    },
                    exec: _editor => run_rust_code(playground_block)
                });
            }
        }
    }

    // updates the visibility of play button based on `no_run` class and
    // used crates vs ones available on https://play.rust-lang.org
    function update_play_button(pre_block, playground_crates) {
        var play_button = pre_block.querySelector(".play-button");

        // skip if code is `no_run`
        if (pre_block.querySelector('code').classList.contains("no_run")) {
            play_button.classList.add("hidden");
            return;
        }

        // get list of `extern crate`'s from snippet
        var txt = playground_text(pre_block);
        var re = /extern\s+crate\s+([a-zA-Z_0-9]+)\s*;/g;
        var snippet_crates = [];
        var item;
        while (item = re.exec(txt)) {
            snippet_crates.push(item[1]);
        }

        // check if all used crates are available on play.rust-lang.org
        var all_available = snippet_crates.every(function (elem) {
            return playground_crates.indexOf(elem) > -1;
        });

        if (all_available) {
            play_button.classList.remove("hidden");
        } else {
            play_button.classList.add("hidden");
        }
    }

    function run_rust_code(code_block) {
        var result_block = code_block.querySelector(".result");
        if (!result_block) {
            result_block = document.createElement('code');
            result_block.className = 'result hljs language-bash';

            code_block.append(result_block);
        }

        let text = playground_text(code_block);
        let classes = code_block.querySelector('code').classList;
        let edition = "2015";
        if(classes.contains("edition2018")) {
            edition = "2018";
        } else if(classes.contains("edition2021")) {
            edition = "2021";
        }
        var params = {
            version: "stable",
            optimize: "0",
            code: text,
            edition: edition
        };

        if (text.indexOf("#![feature") !== -1) {
            params.version = "nightly";
        }

        result_block.innerText = "Running...";

        fetch_with_timeout("https://play.rust-lang.org/evaluate.json", {
            headers: {
                'Content-Type': "application/json",
            },
            method: 'POST',
            mode: 'cors',
            body: JSON.stringify(params)
        })
        .then(response => response.json())
        .then(response => {
            if (response.result.trim() === '') {
                result_block.innerText = "No output";
                result_block.classList.add("result-no-output");
            } else {
                result_block.innerText = response.result;
                result_block.classList.remove("result-no-output");
            }
        })
        .catch(error => result_block.innerText = "Playground Communication: " + error.message);
    }

    // Syntax highlighting Configuration
    hljs.configure({
        tabReplace: '    ', // 4 spaces
        languages: [],      // Languages used for auto-detection
    });

    let code_nodes = Array
        .from(document.querySelectorAll('code'))
        // Don't highlight `inline code` blocks in headers.
        .filter(function (node) {return !node.parentElement.classList.contains("header"); });

    if (window.ace) {
        // language-rust class needs to be removed for editable
        // blocks or highlightjs will capture events
        code_nodes
            .filter(function (node) {return node.classList.contains("editable"); })
            .forEach(function (block) { block.classList.remove('language-rust'); });

        code_nodes
            .filter(function (node) {return !node.classList.contains("editable"); })
            .forEach(function (block) { hljs.highlightBlock(block); });
    } else {
        code_nodes.forEach(function (block) { hljs.highlightBlock(block); });
    }

    // Adding the hljs class gives code blocks the color css
    // even if highlighting doesn't apply
    code_nodes.forEach(function (block) { block.classList.add('hljs'); });

    Array.from(document.querySelectorAll("code.language-rust")).forEach(function (block) {

        var lines = Array.from(block.querySelectorAll('.boring'));
        // If no lines were hidden, return
        if (!lines.length) { return; }
        block.classList.add("hide-boring");

        var buttons = document.createElement('div');
        buttons.className = 'buttons';
        buttons.innerHTML = "<button class=\"fa fa-eye\" title=\"Show hidden lines\" aria-label=\"Show hidden lines\"></button>";

        // add expand button
        var pre_block = block.parentNode;
        pre_block.insertBefore(buttons, pre_block.firstChild);

        pre_block.querySelector('.buttons').addEventListener('click', function (e) {
            if (e.target.classList.contains('fa-eye')) {
                e.target.classList.remove('fa-eye');
                e.target.classList.add('fa-eye-slash');
                e.target.title = 'Hide lines';
                e.target.setAttribute('aria-label', e.target.title);

                block.classList.remove('hide-boring');
            } else if (e.target.classList.contains('fa-eye-slash')) {
                e.target.classList.remove('fa-eye-slash');
                e.target.classList.add('fa-eye');
                e.target.title = 'Show hidden lines';
                e.target.setAttribute('aria-label', e.target.title);

                block.classList.add('hide-boring');
            }
        });
    });

    if (window.playground_copyable) {
        Array.from(document.querySelectorAll('pre code')).forEach(function (block) {
            var pre_block = block.parentNode;
            if (!pre_block.classList.contains('playground')) {
                var buttons = pre_block.querySelector(".buttons");
                if (!buttons) {
                    buttons = document.createElement('div');
                    buttons.className = 'buttons';
                    pre_block.insertBefore(buttons, pre_block.firstChild);
                }

                var clipButton = document.createElement('button');
                clipButton.className = 'fa fa-copy clip-button';
                clipButton.title = 'Copy to clipboard';
                clipButton.setAttribute('aria-label', clipButton.title);
                clipButton.innerHTML = '<i class=\"tooltiptext\"></i>';

                buttons.insertBefore(clipButton, buttons.firstChild);
            }
        });
    }

    // Process playground code blocks
    Array.from(document.querySelectorAll(".playground")).forEach(function (pre_block) {
        // Add play button
        var buttons = pre_block.querySelector(".buttons");
        if (!buttons) {
            buttons = document.createElement('div');
            buttons.className = 'buttons';
            pre_block.insertBefore(buttons, pre_block.firstChild);
        }

        var runCodeButton = document.createElement('button');
        runCodeButton.className = 'fa fa-play play-button';
        runCodeButton.hidden = true;
        runCodeButton.title = 'Run this code';
        runCodeButton.setAttribute('aria-label', runCodeButton.title);

        buttons.insertBefore(runCodeButton, buttons.firstChild);
        runCodeButton.addEventListener('click', function (e) {
            run_rust_code(pre_block);
        });

        if (window.playground_copyable) {
            var copyCodeClipboardButton = document.createElement('button');
            copyCodeClipboardButton.className = 'fa fa-copy clip-button';
            copyCodeClipboardButton.innerHTML = '<i class="tooltiptext"></i>';
            copyCodeClipboardButton.title = 'Copy to clipboard';
            copyCodeClipboardButton.setAttribute('aria-label', copyCodeClipboardButton.title);

            buttons.insertBefore(copyCodeClipboardButton, buttons.firstChild);
        }

        let code_block = pre_block.querySelector("code");
        if (window.ace && code_block.classList.contains("editable")) {
            var undoChangesButton = document.createElement('button');
            undoChangesButton.className = 'fa fa-history reset-button';
            undoChangesButton.title = 'Undo changes';
            undoChangesButton.setAttribute('aria-label', undoChangesButton.title);

            buttons.insertBefore(undoChangesButton, buttons.firstChild);

            undoChangesButton.addEventListener('click', function () {
                let editor = window.ace.edit(code_block);
                editor.setValue(editor.originalCode);
                editor.clearSelection();
            });
        }
    });
})();

(function themes() {
    var html = document.documentElement;
    var themeColorMetaTag = document.querySelector('meta[name="theme-color"]');
    var stylesheets = {
        ayuHighlight: document.querySelector("[href$='ayu-highlight.css']"),
        tomorrowNight: document.querySelector("[href$='tomorrow-night.css']"),
        highlight: document.querySelector("[href$='highlight.css']"),
    };

    function get_theme() {
        var theme;
        try { theme = localStorage.getItem('mdbook-theme'); } catch (e) { }
        if (theme === null || theme === undefined) {
            return window.default_theme;
        } else {
            return theme;
        }
    }

    function set_theme(theme, store = true) {
        let ace_theme;

        if (theme == 'coal' || theme == 'navy') {
            stylesheets.ayuHighlight.disabled = true;
            stylesheets.tomorrowNight.disabled = false;
            stylesheets.highlight.disabled = true;

            ace_theme = "ace/theme/tomorrow_night";
        } else if (theme == 'ayu') {
            stylesheets.ayuHighlight.disabled = false;
            stylesheets.tomorrowNight.disabled = true;
            stylesheets.highlight.disabled = true;
            ace_theme = "ace/theme/tomorrow_night";
        } else {
            stylesheets.ayuHighlight.disabled = true;
            stylesheets.tomorrowNight.disabled = true;
            stylesheets.highlight.disabled = false;
            ace_theme = "ace/theme/dawn";
        }

        setTimeout(function () {
            themeColorMetaTag.content = getComputedStyle(document.body).backgroundColor;
        }, 1);

        if (window.ace && window.editors) {
            window.editors.forEach(function (editor) {
                editor.setTheme(ace_theme);
            });
        }

        var previousTheme = get_theme();

        if (store) {
            try { localStorage.setItem('mdbook-theme', theme); } catch (e) { }
        }

        html.classList.remove(previousTheme);
        html.classList.add(theme);
    }

    set_theme(get_theme(), false);

    window.set_theme = set_theme;
})();

(function sidebar() {
    var html = document.querySelector("html");
    var sidebar = document.getElementById("sidebar");
    var sidebarLinks = document.querySelectorAll('#sidebar a');
    var sidebarToggleButton = document.getElementById("sidebar-toggle");
    var sidebarResizeHandle = document.getElementById("sidebar-resize-handle");
    var firstContact = null;

    function showSidebar() {
        html.classList.remove('sidebar-hidden')
        html.classList.add('sidebar-visible');
        Array.from(sidebarLinks).forEach(function (link) {
            link.setAttribute('tabIndex', 0);
        });
        sidebarToggleButton.setAttribute('aria-expanded', true);
        sidebar.setAttribute('aria-hidden', false);
        try { localStorage.setItem('mdbook-sidebar', 'visible'); } catch (e) { }
    }


    var sidebarAnchorToggles = document.querySelectorAll('#sidebar a.toggle');

    function toggleSection(ev) {
        ev.currentTarget.parentElement.classList.toggle('expanded');
    }

    Array.from(sidebarAnchorToggles).forEach(function (el) {
        el.addEventListener('click', toggleSection);
    });

    function hideSidebar() {
        html.classList.remove('sidebar-visible')
        html.classList.add('sidebar-hidden');
        Array.from(sidebarLinks).forEach(function (link) {
            link.setAttribute('tabIndex', -1);
        });
        sidebarToggleButton.setAttribute('aria-expanded', false);
        sidebar.setAttribute('aria-hidden', true);
        try { localStorage.setItem('mdbook-sidebar', 'hidden'); } catch (e) { }
    }

    // Toggle sidebar
    sidebarToggleButton.addEventListener('click', function sidebarToggle() {
        if (html.classList.contains("sidebar-hidden")) {
            var current_width = parseInt(
                document.documentElement.style.getPropertyValue('--sidebar-width'), 10);
            if (current_width < 150) {
                document.documentElement.style.setProperty('--sidebar-width', '150px');
            }
            showSidebar();
        } else if (html.classList.contains("sidebar-visible")) {
            hideSidebar();
        } else {
            if (getComputedStyle(sidebar)['transform'] === 'none') {
                hideSidebar();
            } else {
                showSidebar();
            }
        }
    });

    sidebarResizeHandle.addEventListener('mousedown', initResize, false);

    function initResize(e) {
        window.addEventListener('mousemove', resize, false);
        window.addEventListener('mouseup', stopResize, false);
        html.classList.add('sidebar-resizing');
    }
    function resize(e) {
        var pos = (e.clientX - sidebar.offsetLeft);
        if (pos < 20) {
            hideSidebar();
        } else {
            if (html.classList.contains("sidebar-hidden")) {
                showSidebar();
            }
            pos = Math.min(pos, window.innerWidth - 100);
            document.documentElement.style.setProperty('--sidebar-width', pos + 'px');
        }
    }
    //on mouseup remove windows functions mousemove & mouseup
    function stopResize(e) {
        html.classList.remove('sidebar-resizing');
        window.removeEventListener('mousemove', resize, false);
        window.removeEventListener('mouseup', stopResize, false);
    }

    document.addEventListener('touchstart', function (e) {
        firstContact = {
            x: e.touches[0].clientX,
            time: Date.now()
        };
    }, { passive: true });

    document.addEventListener('touchmove', function (e) {
        if (!firstContact)
            return;

        var curX = e.touches[0].clientX;
        var xDiff = curX - firstContact.x,
            tDiff = Date.now() - firstContact.time;

        if (tDiff < 250 && Math.abs(xDiff) >= 150) {
            if (xDiff >= 0 && firstContact.x < Math.min(document.body.clientWidth * 0.25, 300))
                showSidebar();
            else if (xDiff < 0 && curX < 300)
                hideSidebar();

            firstContact = null;
        }
    }, { passive: true });

    // Scroll sidebar to current active section
    var activeSection = document.getElementById("sidebar").querySelector(".active");
    if (activeSection) {
        // https://developer.mozilla.org/en-US/docs/Web/API/Element/scrollIntoView
        activeSection.scrollIntoView({ block: 'center' });
    }
})();

(function clipboard() {
    var clipButtons = document.querySelectorAll('.clip-button');

    function hideTooltip(elem) {
        elem.firstChild.innerText = "";
        elem.className = 'fa fa-copy clip-button';
    }

    function showTooltip(elem, msg) {
        elem.firstChild.innerText = msg;
        elem.className = 'fa fa-copy tooltipped';
    }

    var clipboardSnippets = new ClipboardJS('.clip-button', {
        text: function (trigger) {
            hideTooltip(trigger);
            let playground = trigger.closest("pre");
            return playground_text(playground, false);
        }
    });

    Array.from(clipButtons).forEach(function (clipButton) {
        clipButton.addEventListener('mouseout', function (e) {
            hideTooltip(e.currentTarget);
        });
    });

    clipboardSnippets.on('success', function (e) {
        e.clearSelection();
        showTooltip(e.trigger, "Copied!");
    });

    clipboardSnippets.on('error', function (e) {
        showTooltip(e.trigger, "Clipboard error!");
    });
})();

(function scrollToTop () {
    var menuTitle = document.querySelector('.menu-title');

    menuTitle.addEventListener('click', function () {
        document.scrollingElement.scrollTo({ top: 0, behavior: 'smooth' });
    });
})();

(function controllMenu() {
    var menu = document.getElementById('menu-bar');

    (function controllPosition() {
        var scrollTop = document.scrollingElement.scrollTop;
        var prevScrollTop = scrollTop;
        var minMenuY = -menu.clientHeight - 50;
        // When the script loads, the page can be at any scroll (e.g. if you reforesh it).
        menu.style.top = scrollTop + 'px';
        // Same as parseInt(menu.style.top.slice(0, -2), but faster
        var topCache = menu.style.top.slice(0, -2);
        menu.classList.remove('sticky');
        var stickyCache = false; // Same as menu.classList.contains('sticky'), but faster
        document.addEventListener('scroll', function () {
            scrollTop = Math.max(document.scrollingElement.scrollTop, 0);
            // `null` means that it doesn't need to be updated
            var nextSticky = null;
            var nextTop = null;
            var scrollDown = scrollTop > prevScrollTop;
            var menuPosAbsoluteY = topCache - scrollTop;
            if (scrollDown) {
                nextSticky = false;
                if (menuPosAbsoluteY > 0) {
                    nextTop = prevScrollTop;
                }
            } else {
                if (menuPosAbsoluteY > 0) {
                    nextSticky = true;
                } else if (menuPosAbsoluteY < minMenuY) {
                    nextTop = prevScrollTop + minMenuY;
                }
            }
            if (nextSticky === true && stickyCache === false) {
                menu.classList.add('sticky');
                stickyCache = true;
            } else if (nextSticky === false && stickyCache === true) {
                menu.classList.remove('sticky');
                stickyCache = false;
            }
            if (nextTop !== null) {
                menu.style.top = nextTop + 'px';
                topCache = nextTop;
            }
            prevScrollTop = scrollTop;
        }, { passive: true });
    })();
    (function controllBorder() {
        menu.classList.remove('bordered');
        document.addEventListener('scroll', function () {
            if (menu.offsetTop === 0) {
                menu.classList.remove('bordered');
            } else {
                menu.classList.add('bordered');
            }
        }, { passive: true });
    })();
})();

(function settings() {
    const toggle = document.querySelector("#settings-toggle");
    const menu = document.querySelector("#settings-menu");

    const isMac = /^Mac/i.test(navigator.userAgentData?.platform ?? navigator.platform);
    const isTouchDevice = window.matchMedia("(pointer: coarse)").matches;

    const eventModifiers = Object.fromEntries(["ctrl", "alt", "shift", "meta"]
        .map((k) => [k, `${k}Key`]));

    const defaultComboModifier = isMac ? "Meta" : "Control";

    const defaultShortkeys = [
        {
            id: "toc",
            name: "Toggle Table of Contents",
            combo: "t",
            selector: "#sidebar-toggle",
        },
        {
            id: "settings",
            name: "Open settings",
            combo: "/",
            selector: "#settings-toggle",
        },
        {
            id: "search",
            name: "Search",
            combo: "s",
            selector: "#search-toggle",
        },
        {
            id: "previous",
            name: "Previous chapter",
            combo: `${defaultComboModifier}+ArrowLeft`,
            selector: ".nav-chapters.previous",
            altSelectors: [".mobile-nav-chapters.previous"],
        },
        {
            id: "next",
            name: "Next chapter",
            combo: `${defaultComboModifier}+ArrowRight`,
            selector: ".nav-chapters.next",
            altSelectors: [".mobile-nav-chapters.next"],
        },
    ];

    function getCombo(storageKey) {
        const shortkey = localStorage.getItem(`mdbook-shortkeys::${storageKey}`);
        return shortkey ?? defaultShortkeys.find((x) => x.id === storageKey).combo;
    }

    function setCombo(storageKey, combo) {
        localStorage.setItem(`mdbook-shortkeys::${storageKey}`, combo);
    }

    function checkIsTextInputMode() {
        return document.activeElement.isContentEditable ||
            ["INPUT", "TEXTAREA"].includes(document.activeElement.nodeName);
    }

    function eventToCombo(e) {
        const normalized = new Map([
            [" ", "Space"],
            ["+", "Plus"],
            ["Ctrl", "Control"],
        ]);

        const modifierKeys = Object.keys(eventModifiers)
            .filter((k) => e[eventModifiers[k]])
            .map((x) => x.charAt(0).toUpperCase() + x.slice(1));

        if (["Control", "Alt", "Shift", "Meta"].includes(e.key)) return null;

        return [...modifierKeys, e.key]
            .map((x) => normalized.has(x) ? normalized.get(x) : x).join("+");
    }

    function eventMatchesCombo(e, combo) {
        const eventCombo = eventToCombo(e);
        return eventCombo && (eventCombo === combo);
    }

    function keyToPretty(key) {
        const fmtMap = new Map([
            ["ArrowRight", "→"],
            ["ArrowLeft", "←"],
            ["ArrowUp", "↑"],
            ["ArrowDown", "↓"],
            ["Plus", "+"],
            ["Control", isMac ? "Control" : "Ctrl"],
            ["Alt", isMac ? "⌥" : "Alt"],
            ["Meta", isMac ? "⌘" : "Meta"],
        ]);

        return fmtMap.has(key) ? fmtMap.get(key) : key;
    }

    function comboToPretty(combo) {
        return combo.split("+").map(keyToPretty).join("+");
    }

    function comboToPrettyHtml(combo) {
        const html = (text) =>
            Object.assign(document.createElement("span"), { textContent: text })
                .innerHTML;

        return combo.split("+").map((x) => `<kbd>${html(keyToPretty(x))}</kbd>`).join(
            "<span>+</span>",
        );
    }

    function renderShortkeyField(shortkey) {
        const div = document.createElement("div");

        const combo = getCombo(shortkey.id);
        const touched = combo !== shortkey.combo;
        const changeLabel = `Change shortcut key for ${shortkey.name}`;
        const buttonAttrs = (label) =>
            `aria-label="${label}" title="${label}" aria-controls="shortkey-${shortkey.id}"`;

        div.classList.add("shortkey");
        div.innerHTML = `<label for="shortkey-${shortkey.id}">${shortkey.name}</label>
            <div class="shortkey__control${
                touched ? " shortkey__control--touched" : ""
            }" data-shortkey-item="${shortkey.id}">
                <span class="shortkey__input">
                    <input${
                        touched ? "" : " disabled"
                    } id="shortkey-${shortkey.id}" autocomplete="off" value="Control+ArrowLeft">
                    <span aria-hidden="true" class="shortkey__display">${
                        comboToPrettyHtml(combo)
                    }</span>
                </span>
                <button class="shortkey__change" type="button" ${buttonAttrs(changeLabel)}>
                    <i class="fa fa-pencil" aria-hidden="true"></i>
                </button>
            </div>`;

        return div;
    }

    menu.innerHTML = `
        <h2>Settings</h2>
        <fieldset>
        <legend>
            <span>Appearance</span>
        </legend>
        <div>
            <label for="theme">Theme</label>
            <select id="theme">
                ${["light", "rust", "coal", "navy", "ayu"].map((theme) =>
                    `<option${(localStorage.getItem("mdbook-theme") ?? window.default_theme) ===
                        theme
                            ? " selected"
                            : ""
                    } value="${theme}">${theme.charAt(0).toUpperCase() + theme.slice(1)}</option>`
                ).join("")
                }
            </select>
        </div>
        </fieldset>
        ${isTouchDevice ? "" : `<fieldset id="shortkeys" class="shortkeys">
            <legend>
                <span>Keyboard shortcuts</span>
            </legend>
            ${defaultShortkeys.map((x) => renderShortkeyField(x).outerHTML).join("")}
            <div>
                <button class="shortkeys__reset-all" type="reset">
                    Reset all keyboard shortcuts
                </button>
            </div>
        </fieldset>`}
    `;

    function updateButtons() {
        for (const shortkey of defaultShortkeys) {
            for (
                const button of document.querySelectorAll(
                    [shortkey.selector, ...(shortkey.altSelectors ?? [])].join(", "),
                )
            ) {
                const combo = getCombo(shortkey.id);
                button.setAttribute("aria-keyshortcuts", combo);
                button.title = button.title.replace(
                    /(?: \(.+\))?$/,
                    ` (${comboToPretty(combo)})`,
                );
            }
        }
    }

    updateButtons();

    function toggleSettingsPopup(open = toggle.getAttribute("aria-expanded") !== "true") {
        toggle.setAttribute("aria-expanded", String(open));
        menu.hidden = !open;

        if (open) {
            menu.querySelector("input, button, textarea, select").focus();
        }
    }

    toggle.addEventListener("click", () => toggleSettingsPopup());
    menu.addEventListener("submit", (e) => e.preventDefault());
    menu.addEventListener("change", updateButtons);

    menu.querySelector("#theme").addEventListener("change", (e) => {
        window.set_theme(e.target.value);
    });

    menu.addEventListener("keydown", (e) => {
        if (!e.target.matches(".shortkey__control input")) return;
        if (["Escape", "Tab", "Enter", " "].includes(e.key)) return;

        const parent = e.target.closest(".shortkey__control");

        e.preventDefault();
        const combo = eventToCombo(e);

        if (!combo) return;

        const html = comboToPrettyHtml(combo);

        setCombo(parent.dataset.shortkeyItem, combo);
        e.target.value = combo;
        e.currentTarget.dispatchEvent(new Event("change"));

        parent.querySelector(".shortkey__display").innerHTML = html;
    });

    menu.addEventListener("click", (e) => {
        if (e.target.closest(".shortkey__control .shortkey__change")) {
            e.preventDefault();

            const input = e.target.closest(".shortkey__control").querySelector(
                "input",
            );
            input.disabled = false;
            e.target.closest(".shortkey__control").classList.add(
                "shortkey__control--touched",
            );
            input.focus();
        } else if (e.target.closest("#shortkeys .shortkeys__reset-all")) {
            e.preventDefault();

            for (const el of e.currentTarget.querySelectorAll(".shortkey__control")) {
                const shortkey = defaultShortkeys.find((x) =>
                    x.id === el.dataset.shortkeyItem
                );

                setCombo(el.dataset.shortkeyItem, shortkey.combo);
                el.closest(".shortkey").replaceWith(renderShortkeyField(shortkey));
            }
            e.currentTarget.dispatchEvent(new Event("change"));
        }
    });

    menu.addEventListener("focusout", (e) => {
        if (e.target.matches(".shortkey__control input")) {
            e.target.closest(".shortkey").replaceWith(
                renderShortkeyField(
                    defaultShortkeys.find((x) =>
                        x.id === e.target.closest(".shortkey__control").dataset.shortkeyItem
                    ),
                ),
            );
        }
    });

    document.addEventListener("keydown", (e) => {
        if (checkIsTextInputMode()) return;

        for (const shortkey of defaultShortkeys) {
            if (eventMatchesCombo(e, getCombo(shortkey.id))) {
                e.preventDefault();
                const button = document.querySelector(shortkey.selector);
                if (button) {
                    button.focus();
                    button.click();
                }

                return;
            }
        }
    });

    window.addEventListener("keydown", (e) => {
        if (e.key === "Escape") {
            if (e.target.closest("#settings-menu")) {
                toggleSettingsPopup(false);
                toggle.focus();
            } else if (!checkIsTextInputMode()) {
                toggleSettingsPopup(false);
            }
        }
    });

    window.addEventListener("click", (e) => {
        if (
            e.isTrusted && !e.target.closest("#settings-menu") &&
            !e.target.closest("#settings-toggle")
        ) {
            toggleSettingsPopup(false);
        }
    });
})();
