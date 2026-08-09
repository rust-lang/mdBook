'use strict';

/* global default_theme, default_dark_theme, default_light_theme, hljs, ClipboardJS */

// Fix back button cache problem
window.onunload = function() { };

/**
 * Helper for global keypress handlers so they don't trigger when certain elements are active.
 * @returns {boolean} True if the keypress handler should be skipped.
 */
function mdbook_something_else_has_focus(e) {
    // Check composedPath in case the event happened from something generated
    // from the shadowDOM.
    const target = e.composedPath()[0] || e.target;
    // If this is the `checkbox-img` input which has the focus, we want to handle it here.
    if (target.classList.contains('checkbox-img')) {
        return false;
    }
    return /^(?:input|select|textarea)$/i.test(target.nodeName);
}

(function codeSnippets() {
    // Syntax highlighting Configuration
    hljs.configure({
        tabReplace: '    ', // 4 spaces
        languages: [], // Languages used for auto-detection
    });

    const code_nodes = Array
        .from(document.querySelectorAll('code'))
        // Don't highlight `inline code` blocks in headers.
        .filter(function(node) {
            return !node.parentElement.classList.contains('header');
        });

    code_nodes.forEach(function(block) {
        hljs.highlightBlock(block);
    });

    // Adding the hljs class gives code blocks the color css
    // even if highlighting doesn't apply
    code_nodes.forEach(function(block) {
        block.classList.add('hljs');
    });

    Array.from(document.querySelectorAll('code.hljs')).forEach(function(block) {

        const lines = Array.from(block.querySelectorAll('.boring'));
        // If no lines were hidden, return
        if (!lines.length) {
            return;
        }
        block.classList.add('hide-boring');

        const buttons = document.createElement('div');
        buttons.className = 'buttons';
        buttons.innerHTML = '<button title="Show hidden lines" \
aria-label="Show hidden lines"></button>';

        // add expand button
        const pre_block = block.parentNode;
        pre_block.insertBefore(buttons, pre_block.firstChild);

        buttons.firstChild.addEventListener('click', function(e) {
            if (this.title === 'Show hidden lines') {
                this.title = 'Hide lines';
                this.setAttribute('aria-label', e.target.title);

                block.classList.remove('hide-boring');
            } else if (this.title === 'Hide lines') {
                this.title = 'Show hidden lines';
                this.setAttribute('aria-label', e.target.title);

                block.classList.add('hide-boring');
            }
        });
    });

    Array.from(document.querySelectorAll('pre code')).forEach(function(block) {
        const pre_block = block.parentNode;
        let buttons = pre_block.querySelector('.buttons');
        if (!buttons) {
            buttons = document.createElement('div');
            buttons.className = 'buttons';
            pre_block.insertBefore(buttons, pre_block.firstChild);
        }

        const clipButton = document.createElement('button');
        clipButton.className = 'clip-button';
        clipButton.title = 'Copy to clipboard';
        clipButton.setAttribute('aria-label', clipButton.title);
        clipButton.innerHTML = '<i class="tooltiptext"></i>';

        buttons.insertBefore(clipButton, buttons.firstChild);
    });
})();

(function themes() {
    const html = document.querySelector('html');
    const themeToggleButton = document.getElementById('mdbook-theme-toggle');
    const themeColorMetaTag = document.querySelector('meta[name="theme-color"]');
    const stylesheets = {
        ayuHighlight: document.querySelector('#mdbook-ayu-highlight-css'),
        tomorrowNight: document.querySelector('#mdbook-tomorrow-night-css'),
        highlight: document.querySelector('#mdbook-highlight-css'),
        githubMarkdownLight: document.querySelector('#mdbook-github-markdown-light-css'),
        githubMarkdownDark: document.querySelector('#mdbook-github-markdown-dark-css'),
    };

    function get_saved_theme() {
        let theme = null;
        try {
            theme = localStorage.getItem('mdbook-theme');
        } catch {
            // ignore error.
        }
        return theme;
    }

    function delete_saved_theme() {
        localStorage.removeItem('mdbook-theme');
    }

    function get_theme() {
        const theme = get_saved_theme();
        if (theme === null || theme === undefined) {
            if (typeof default_dark_theme === 'undefined') {
                // A customized index.hbs might not define this, so fall back to
                // old behavior of determining the default on page load.
                return default_theme;
            }
            return window.matchMedia('(prefers-color-scheme: dark)').matches
                ? default_dark_theme
                : default_light_theme;
        }
        return theme;
    }

    let previousTheme = default_theme;
    function set_theme(theme, store = true) {
        let ace_theme;

        if (theme === 'coal' || theme === 'navy') {
            stylesheets.ayuHighlight.disabled = true;
            stylesheets.tomorrowNight.disabled = false;
            stylesheets.highlight.disabled = true;
            stylesheets.githubMarkdownLight.disabled = true;
            stylesheets.githubMarkdownDark.disabled = false;

            ace_theme = 'ace/theme/tomorrow_night';
        } else if (theme === 'ayu') {
            stylesheets.ayuHighlight.disabled = false;
            stylesheets.tomorrowNight.disabled = true;
            stylesheets.highlight.disabled = true;
            stylesheets.githubMarkdownLight.disabled = true;
            stylesheets.githubMarkdownDark.disabled = false;
            ace_theme = 'ace/theme/tomorrow_night';
        } else {
            stylesheets.ayuHighlight.disabled = true;
            stylesheets.tomorrowNight.disabled = true;
            stylesheets.highlight.disabled = false;
            stylesheets.githubMarkdownLight.disabled = false;
            stylesheets.githubMarkdownDark.disabled = true;
            ace_theme = 'ace/theme/dawn';
        }

        setTimeout(function() {
            themeColorMetaTag.content = getComputedStyle(document.documentElement).backgroundColor;
        }, 1);

        if (store) {
            try {
                localStorage.setItem('mdbook-theme', theme);
            } catch {
                // ignore error.
            }
        }

        html.classList.remove(previousTheme);
        html.classList.add(theme);
        previousTheme = theme;
    }

    const query = window.matchMedia('(prefers-color-scheme: dark)');
    query.onchange = function() {
        // set_theme(get_theme(), false);
    };

    // Set theme.
    // set_theme(get_theme(), false);
})();

(function sidebar() {
    return

    const sidebar = document.getElementById('mdbook-sidebar');
    const sidebarLinks = document.querySelectorAll('#mdbook-sidebar a');
    const sidebarToggleButton = document.getElementById('mdbook-sidebar-toggle');
    const sidebarResizeHandle = document.getElementById('mdbook-sidebar-resize-handle');
    const sidebarCheckbox = document.getElementById('mdbook-sidebar-toggle-anchor');
    let firstContact = null;


    /* Because we cannot change the `display` using only CSS after/before the transition, we
       need JS to do it. We change the display to prevent the browsers search to find text inside
       the collapsed sidebar. */
    if (!document.documentElement.classList.contains('sidebar-visible')) {
        sidebar.style.display = 'none';
    }
    sidebar.addEventListener('transitionend', () => {
        /* We only change the display to "none" if we're collapsing the sidebar. */
        if (!sidebarCheckbox.checked) {
            sidebar.style.display = 'none';
        }
    });
    // sidebarToggleButton.addEventListener('click', () => {
    //     /* To allow the sidebar expansion animation, we first need to put back the display. */
    //     if (!sidebarCheckbox.checked) {
    //         sidebar.style.display = '';
    //         // Workaround for Safari skipping the animation when changing
    //         // `display` and a transform in the same event loop. This forces a
    //         // reflow after updating the display.
    //         sidebar.offsetHeight;
    //     }
    // });

    function showSidebar() {
        document.documentElement.classList.add('sidebar-visible');
        Array.from(sidebarLinks).forEach(function(link) {
            link.setAttribute('tabIndex', 0);
        });
        sidebarToggleButton.setAttribute('aria-expanded', true);
        sidebar.setAttribute('aria-hidden', false);
        try {
            localStorage.setItem('mdbook-sidebar', 'visible');
        } catch {
            // Ignore error.
        }
    }

    function hideSidebar() {
        document.documentElement.classList.remove('sidebar-visible');
        Array.from(sidebarLinks).forEach(function(link) {
            link.setAttribute('tabIndex', -1);
        });
        sidebarToggleButton.setAttribute('aria-expanded', false);
        sidebar.setAttribute('aria-hidden', true);
        try {
            localStorage.setItem('mdbook-sidebar', 'hidden');
        } catch {
            // Ignore error.
        }
    }

    // Toggle sidebar
    sidebarCheckbox.addEventListener('change', function sidebarToggle() {
        if (sidebarCheckbox.checked) {
            const current_width = parseInt(
                document.documentElement.style.getPropertyValue('--sidebar-target-width'), 10);
            if (current_width < 150) {
                document.documentElement.style.setProperty('--sidebar-target-width', '150px');
            }
            showSidebar();
        } else {
            hideSidebar();
        }
    });

    sidebarResizeHandle.addEventListener('mousedown', initResize, false);

    function initResize() {
        window.addEventListener('mousemove', resize, false);
        window.addEventListener('mouseup', stopResize, false);
        document.documentElement.classList.add('sidebar-resizing');
    }
    function resize(e) {
        let pos = e.clientX - sidebar.offsetLeft;
        if (pos < 20) {
            hideSidebar();
        } else {
            if (!document.documentElement.classList.contains('sidebar-visible')) {
                showSidebar();
            }
            pos = Math.min(pos, window.innerWidth - 100);
            document.documentElement.style.setProperty('--sidebar-target-width', pos + 'px');
        }
    }
    //on mouseup remove windows functions mousemove & mouseup
    function stopResize() {
        document.documentElement.classList.remove('sidebar-resizing');
        window.removeEventListener('mousemove', resize, false);
        window.removeEventListener('mouseup', stopResize, false);
    }

    document.addEventListener('touchstart', function(e) {
        firstContact = {
            x: e.touches[0].clientX,
            time: Date.now(),
        };
    }, { passive: true });

    document.addEventListener('touchmove', function(e) {
        if (!firstContact) {
            return;
        }

        const curX = e.touches[0].clientX;
        const xDiff = curX - firstContact.x,
            tDiff = Date.now() - firstContact.time;

        if (tDiff < 250 && Math.abs(xDiff) >= 150) {
            if (xDiff >= 0 && firstContact.x < Math.min(document.body.clientWidth * 0.25, 300)) {
                showSidebar();
            } else if (xDiff < 0 && curX < 300) {
                hideSidebar();
            }

            firstContact = null;
        }
    }, { passive: true });
})();
