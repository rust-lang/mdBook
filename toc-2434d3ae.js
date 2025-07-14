// Populate the sidebar
//
// This is a script, and not included directly in the page, to control the total size of the book.
// The TOC contains an entry for each page, so if each page includes a copy of the TOC,
// the total size of the page becomes O(n**2).
class MDBookSidebarScrollbox extends HTMLElement {
    constructor() {
        super();
    }
    connectedCallback() {
        this.innerHTML = '<ol class="chapter"><li class="chapter-item expanded affix "><a href="index.html">Introduction</a></li><li class="chapter-item expanded affix "><li class="part-title">User Guide</li><li class="chapter-item expanded "><a href="guide/installation.html"><strong aria-hidden="true">1.</strong> Installation</a></li><li class="chapter-item expanded "><a href="guide/reading.html"><strong aria-hidden="true">2.</strong> Reading Books</a></li><li class="chapter-item expanded "><a href="guide/creating.html"><strong aria-hidden="true">3.</strong> Creating a Book</a></li><li class="chapter-item expanded affix "><li class="part-title">Reference Guide</li><li class="chapter-item expanded "><a href="cli/index.html"><strong aria-hidden="true">4.</strong> Command Line Tool</a></li><li><ol class="section"><li class="chapter-item expanded "><a href="cli/init.html"><strong aria-hidden="true">4.1.</strong> init</a></li><li class="chapter-item expanded "><a href="cli/build.html"><strong aria-hidden="true">4.2.</strong> build</a></li><li class="chapter-item expanded "><a href="cli/watch.html"><strong aria-hidden="true">4.3.</strong> watch</a></li><li class="chapter-item expanded "><a href="cli/serve.html"><strong aria-hidden="true">4.4.</strong> serve</a></li><li class="chapter-item expanded "><a href="cli/test.html"><strong aria-hidden="true">4.5.</strong> test</a></li><li class="chapter-item expanded "><a href="cli/clean.html"><strong aria-hidden="true">4.6.</strong> clean</a></li><li class="chapter-item expanded "><a href="cli/completions.html"><strong aria-hidden="true">4.7.</strong> completions</a></li></ol></li><li class="chapter-item expanded "><a href="format/index.html"><strong aria-hidden="true">5.</strong> Format</a></li><li><ol class="section"><li class="chapter-item expanded "><a href="format/summary.html"><strong aria-hidden="true">5.1.</strong> SUMMARY.md</a></li><li><ol class="section"><li class="chapter-item expanded "><div><strong aria-hidden="true">5.1.1.</strong> Draft chapter</div></li></ol></li><li class="chapter-item expanded "><a href="format/configuration/index.html"><strong aria-hidden="true">5.2.</strong> Configuration</a></li><li><ol class="section"><li class="chapter-item expanded "><a href="format/configuration/general.html"><strong aria-hidden="true">5.2.1.</strong> General</a></li><li class="chapter-item expanded "><a href="format/configuration/preprocessors.html"><strong aria-hidden="true">5.2.2.</strong> Preprocessors</a></li><li class="chapter-item expanded "><a href="format/configuration/renderers.html"><strong aria-hidden="true">5.2.3.</strong> Renderers</a></li><li class="chapter-item expanded "><a href="format/configuration/environment-variables.html"><strong aria-hidden="true">5.2.4.</strong> Environment Variables</a></li></ol></li><li class="chapter-item expanded "><a href="format/theme/index.html"><strong aria-hidden="true">5.3.</strong> Theme</a></li><li><ol class="section"><li class="chapter-item expanded "><a href="format/theme/index-hbs.html"><strong aria-hidden="true">5.3.1.</strong> index.hbs</a></li><li class="chapter-item expanded "><a href="format/theme/syntax-highlighting.html"><strong aria-hidden="true">5.3.2.</strong> Syntax highlighting</a></li><li class="chapter-item expanded "><a href="format/theme/editor.html"><strong aria-hidden="true">5.3.3.</strong> Editor</a></li></ol></li><li class="chapter-item expanded "><a href="format/mathjax.html"><strong aria-hidden="true">5.4.</strong> MathJax Support</a></li><li class="chapter-item expanded "><a href="format/mdbook.html"><strong aria-hidden="true">5.5.</strong> mdBook-specific features</a></li><li class="chapter-item expanded "><a href="format/markdown.html"><strong aria-hidden="true">5.6.</strong> Markdown</a></li></ol></li><li class="chapter-item expanded "><a href="continuous-integration.html"><strong aria-hidden="true">6.</strong> Continuous Integration</a></li><li class="chapter-item expanded "><a href="for_developers/index.html"><strong aria-hidden="true">7.</strong> For Developers</a></li><li><ol class="section"><li class="chapter-item expanded "><a href="for_developers/preprocessors.html"><strong aria-hidden="true">7.1.</strong> Preprocessors</a></li><li class="chapter-item expanded "><a href="for_developers/backends.html"><strong aria-hidden="true">7.2.</strong> Alternative Backends</a></li></ol></li><li class="chapter-item expanded "><li class="spacer"></li><li class="chapter-item expanded affix "><a href="misc/contributors.html">Contributors</a></li></ol>';
        // Set the current, active page, and reveal it if it's hidden
        let current_page = document.location.href.toString().split("#")[0].split("?")[0];
        if (current_page.endsWith("/")) {
            current_page += "index.html";
        }
        var links = Array.prototype.slice.call(this.querySelectorAll("a"));
        var l = links.length;
        for (var i = 0; i < l; ++i) {
            var link = links[i];
            var href = link.getAttribute("href");
            if (href && !href.startsWith("#") && !/^(?:[a-z+]+:)?\/\//.test(href)) {
                link.href = path_to_root + href;
            }
            // The "index" page is supposed to alias the first chapter in the book.
            if (link.href === current_page || (i === 0 && path_to_root === "" && current_page.endsWith("/index.html"))) {
                link.classList.add("active");
                var parent = link.parentElement;
                if (parent && parent.classList.contains("chapter-item")) {
                    parent.classList.add("expanded");
                }
                while (parent) {
                    if (parent.tagName === "LI" && parent.previousElementSibling) {
                        if (parent.previousElementSibling.classList.contains("chapter-item")) {
                            parent.previousElementSibling.classList.add("expanded");
                        }
                    }
                    parent = parent.parentElement;
                }
            }
        }
        // Track and set sidebar scroll position
        this.addEventListener('click', function(e) {
            if (e.target.tagName === 'A') {
                sessionStorage.setItem('sidebar-scroll', this.scrollTop);
            }
        }, { passive: true });
        var sidebarScrollTop = sessionStorage.getItem('sidebar-scroll');
        sessionStorage.removeItem('sidebar-scroll');
        if (sidebarScrollTop) {
            // preserve sidebar scroll position when navigating via links within sidebar
            this.scrollTop = sidebarScrollTop;
        } else {
            // scroll sidebar to current active section when navigating via "next/previous chapter" buttons
            var activeSection = document.querySelector('#sidebar .active');
            if (activeSection) {
                activeSection.scrollIntoView({ block: 'center' });
            }
        }
        // Toggle buttons
        var sidebarAnchorToggles = document.querySelectorAll('#sidebar a.toggle');
        function toggleSection(ev) {
            ev.currentTarget.parentElement.classList.toggle('expanded');
        }
        Array.from(sidebarAnchorToggles).forEach(function (el) {
            el.addEventListener('click', toggleSection);
        });
    }
}
window.customElements.define("mdbook-sidebar-scrollbox", MDBookSidebarScrollbox);
