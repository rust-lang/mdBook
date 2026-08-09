package html_template

import (
	"strings"
)

// renderTocSidebar computes the sidebar HTML so the templates see a
// pre-computed string via {{.TocHTML}}. It is the port of the {{#toc}}
// block helper in crates/mdbook-html/src/html_handlebars/helpers/toc.rs.
func renderTocSidebar(chapters []any, foldEnable bool, foldLevel int, noSectionLabel bool, isTocHTML bool) string {
	var out strings.Builder
	out.WriteString(`<ol class="chapter">`)

	currentLevel := 1
	first := true
	for _, raw := range chapters {
		item := toStringMap(raw)

		// Section numbers carry a trailing dot ("1."), so the nesting level is
		// the dot count itself — mirrors Rust's toc.rs, which deliberately
		// relies on the trailing dot instead of adding one.
		itemLevel := 1
		if section, ok := item["section"]; ok && section != "" {
			itemLevel = strings.Count(section, ".")
		}
		expanded := !foldEnable || itemLevel-1 < foldLevel

		switch {
		case itemLevel > currentLevel:
			currentLevel++
			out.WriteString(`<ol class="section">`)
			writeLiOpenTag(&out, expanded)
		case itemLevel < currentLevel:
			for itemLevel < currentLevel {
				out.WriteString("</li></ol>")
				currentLevel--
			}
			writeLiOpenTag(&out, expanded)
		default:
			if !first {
				out.WriteString("</li>")
			}
			writeLiOpenTag(&out, expanded)
		}
		first = false

		if _, isSpacer := item["spacer"]; isSpacer {
			out.WriteString(`<li class="spacer"></li>`)
			continue
		}
		if title, isPart := item["part"]; isPart {
			out.WriteString(`<li class="part-title">`)
			out.WriteString(htmlEscape(title))
			out.WriteString("</li>")
			continue
		}

		out.WriteString(`<span class="chapter-link-wrapper">`)

		path, hasPath := item["path"]
		if hasPath && path != "" {
			out.WriteString(`<a href="`)
			// Rust's TOC omits any leading `./` on chapter hrefs; align.
			path = strings.TrimPrefix(path, "./")
			out.WriteString(htmlEscape(withHTMLExtension(path)))
			if isTocHTML {
				out.WriteString(`" target="_parent">`)
			} else {
				out.WriteString(`">`)
			}
		} else {
			hasPath = false
			out.WriteString("<span>")
		}

		if !noSectionLabel {
			if section, ok := item["section"]; ok {
				out.WriteString(`<strong aria-hidden="true">`)
				out.WriteString(section)
				out.WriteString("</strong> ")
			}
		}
		if name, ok := item["name"]; ok {
			out.WriteString(htmlEscape(name))
		}
		if hasPath {
			out.WriteString("</a>")
		} else {
			out.WriteString("</span>")
		}

		if flag, ok := item["has_sub_items"]; ok && foldEnable && flag == "true" {
			out.WriteString(`<a class="chapter-fold-toggle"><div>❱</div></a>`)
		}
		out.WriteString("</span>")
	}
	for currentLevel > 0 {
		out.WriteString("</li></ol>")
		currentLevel--
	}
	return out.String()
}

func writeLiOpenTag(out *strings.Builder, expanded bool) {
	out.WriteString(`<li class="chapter-item `)
	if expanded {
		out.WriteString("expanded ")
	}
	out.WriteString(`">`)
}

// withHTMLExtension replaces a path's extension with `.html`, matching
// Rust's Path::with_extension.
func withHTMLExtension(path string) string {
	slash := strings.LastIndexByte(path, '/')
	dot := strings.LastIndexByte(path, '.')
	if dot > slash {
		return path[:dot] + ".html"
	}
	return path + ".html"
}

func toStringMap(v any) map[string]string {
	out := map[string]string{}
	m, ok := v.(map[string]any)
	if !ok {
		return out
	}
	for k, value := range m {
		if s, ok := value.(string); ok {
			out[k] = s
		}
	}
	return out
}

func htmlEscape(s string) string {
	r := strings.NewReplacer(
		`&`, `&amp;`,
		`<`, `&lt;`,
		`>`, `&gt;`,
		`"`, `&#34;`,
		`'`, `&#39;`,
	)
	return r.Replace(s)
}

// SidebarHeaderNavSource is the IIFE that implements dynamic header tracking
// in the sidebar (gated on `sidebar_header_nav`). We expose it as a constant
// so callers (Env.SidebarHeaderNavBlocks) can splice it into toc.js without a
// template round-trip.
const SidebarHeaderNavSource = `
// ---------------------------------------------------------------------------
// Support for dynamically adding headers to the sidebar.

(function() {
    // This is used to detect which direction the page has scrolled since the
    // last scroll event.
    let lastKnownScrollPosition = 0;
    // This is the threshold in px from the top of the screen where it will
    // consider a header the "current" header when scrolling down.
    const defaultDownThreshold = 150;
    // Same as defaultDownThreshold, except when scrolling up.
    const defaultUpThreshold = 300;
    // The threshold is a virtual horizontal line on the screen where it
    // considers the "current" header to be above the line. The threshold is
    // modified dynamically to handle headers that are near the bottom of the
    // screen, and to slightly offset the behavior when scrolling up vs down.
    let threshold = defaultDownThreshold;
    // This is used to disable updates while scrolling. This is needed when
    // clicking the header in the sidebar, which triggers a scroll event. It
    // is somewhat finicky to detect when the scroll has finished, so this
    // uses a relatively dumb system of disabling scroll updates for a short
    // time after the click.
    let disableScroll = false;
    // Array of header elements on the page.
    let headers;
    // Array of li elements that are initially collapsed headers in the sidebar.
    // I'm not sure why eslint seems to have a false positive here.
    // eslint-disable-next-line prefer-const
    let headerToggles = [];
    // This is a debugging tool for the threshold which you can enable in the console.
    let thresholdDebug = false;

    // Updates the threshold based on the scroll position.
    function updateThreshold() {
        const scrollTop = window.pageYOffset || document.documentElement.scrollTop;
        const windowHeight = window.innerHeight;
        const documentHeight = document.documentElement.scrollHeight;

        // The number of pixels below the viewport, at most documentHeight.
        // This is used to push the threshold down to the bottom of the page
        // as the user scrolls towards the bottom.
        const pixelsBelow = Math.max(0, documentHeight - (scrollTop + windowHeight));
        // The number of pixels above the viewport, at least defaultDownThreshold.
        // Similar to pixelsBelow, this is used to push the threshold back towards
        // the top when reaching the top of the page.
        const pixelsAbove = Math.max(0, defaultDownThreshold - scrollTop);
        // How much the threshold should be offset once it gets close to the
        // bottom of the page.
        const bottomAdd = Math.max(0, windowHeight - pixelsBelow - defaultDownThreshold);
        let adjustedBottomAdd = bottomAdd;

        // Adjusts bottomAdd for a small document. The calculation above
        // assumes the document is at least twice the windowheight in size. If
        // it is less than that, then bottomAdd needs to be shrunk
        // proportional to the difference in size.
        if (documentHeight < windowHeight * 2) {
            const maxPixelsBelow = documentHeight - windowHeight;
            const t = 1 - pixelsBelow / Math.max(1, maxPixelsBelow);
            const clamp = Math.max(0, Math.min(1, t));
            adjustedBottomAdd *= clamp;
        }

        let scrollingDown = true;
        if (scrollTop < lastKnownScrollPosition) {
            scrollingDown = false;
        }

        if (scrollingDown) {
            // When scrolling down, move the threshold up towards the default
            // downwards threshold position. If near the bottom of the page,
            // adjustedBottomAdd will offset the threshold towards the bottom
            // of the page.
            const amountScrolledDown = scrollTop - lastKnownScrollPosition;
            const adjustedDefault = defaultDownThreshold + adjustedBottomAdd;
            threshold = Math.max(adjustedDefault, threshold - amountScrolledDown);
        } else {
            // When scrolling up, move the threshold down towards the default
            // upwards threshold position. If near the bottom of the page,
            // quickly transition the threshold back up where it normally
            // belongs.
            const amountScrolledUp = lastKnownScrollPosition - scrollTop;
            const adjustedDefault = defaultUpThreshold - pixelsAbove
                + Math.max(0, adjustedBottomAdd - defaultDownThreshold);
            threshold = Math.min(adjustedDefault, threshold + amountScrolledUp);
        }

        if (documentHeight <= windowHeight) {
            threshold = 0;
        }

        if (thresholdDebug) {
            const id = 'mdbook-threshold-debug-data';
            let data = document.getElementById(id);
            if (data === null) {
                data = document.createElement('div');
                data.id = id;
                data.style.cssText = 'position: fixed; top: 50px; right: 10px; background-color: 0xeeeeee; z-index: 9999; pointer-events: none;';
                document.body.appendChild(data);
            }
            data.innerHTML = '<table><tr><td>documentHeight</td><td>' + documentHeight.toFixed(1) + '</td></tr><tr><td>windowHeight</td><td>' + windowHeight.toFixed(1) + '</td></tr><tr><td>scrollTop</td><td>' + scrollTop.toFixed(1) + '</td></tr><tr><td>pixelsAbove</td><td>' + pixelsAbove.toFixed(1) + '</td></tr><tr><td>pixelsBelow</td><td>' + pixelsBelow.toFixed(1) + '</td></tr><tr><td>bottomAdd</td><td>' + bottomAdd.toFixed(1) + '</td></tr><tr><td>adjustedBottomAdd</td><td>' + adjustedBottomAdd.toFixed(1) + '</td></tr><tr><td>scrollingDown</td><td>' + scrollingDown + '</td></tr><tr><td>threshold</td><td>' + threshold.toFixed(1) + '</td></tr></table>';
            drawDebugLine();
        }

        lastKnownScrollPosition = scrollTop;
    }

    function drawDebugLine() {
        if (!document.body) {
            return;
        }
        const id = 'mdbook-threshold-debug-line';
        const existingLine = document.getElementById(id);
        if (existingLine) {
            existingLine.remove();
        }
        const line = document.createElement('div');
        line.id = id;
        line.style.cssText = 'position: fixed; top: ' + threshold + 'px; left: 0; width: 100vw; height: 2px; background-color: red; z-index: 9999; pointer-events: none;';
        document.body.appendChild(line);
    }

    function mdbookEnableThresholdDebug() {
        thresholdDebug = true;
        updateThreshold();
        drawDebugLine();
    }

    window.mdbookEnableThresholdDebug = mdbookEnableThresholdDebug;

    // Updates which headers in the sidebar should be expanded.
    function updateHeaderExpanded(currentA) {
        let current = currentA.parentElement;
        while (current) {
            if (current.tagName === 'LI' && current.classList.contains('header-item')) {
                current.classList.add('expanded');
            }
            current = current.parentElement;
        }
    }

    // Updates which header is marked as the "current" header in the sidebar.
    function updateCurrentHeader() {
        if (!headers || !headers.length) {
            return;
        }
        const els = document.getElementsByClassName('current-header');
        for (const el of els) {
            el.classList.remove('current-header');
        }
        for (const toggle of headerToggles) {
            toggle.classList.remove('expanded');
        }
        let lastHeader = null;
        for (const header of headers) {
            const rect = header.getBoundingClientRect();
            if (rect.top <= threshold) {
                lastHeader = header;
            } else {
                break;
            }
        }
        if (lastHeader === null) {
            lastHeader = headers[0];
            const rect = lastHeader.getBoundingClientRect();
            const windowHeight = window.innerHeight;
            if (rect.top >= windowHeight) {
                return;
            }
        }
        const href = '#' + lastHeader.id;
        const a = [...document.querySelectorAll('.header-in-summary')]
            .find(element => element.getAttribute('href') === href);
        if (!a) {
            return;
        }
        a.classList.add('current-header');
        updateHeaderExpanded(a);
    }

    function reloadCurrentHeader() {
        if (disableScroll) {
            return;
        }
        updateThreshold();
        updateCurrentHeader();
    }

    function headerThresholdClick(event) {
        disableScroll = true;
        setTimeout(() => {
            disableScroll = false;
        }, 100);
        requestAnimationFrame(() => {
            requestAnimationFrame(() => {
                const a = event.target.closest('a');
                const href = a.getAttribute('href');
                const targetId = href.substring(1);
                const targetElement = document.getElementById(targetId);
                if (targetElement) {
                    threshold = targetElement.getBoundingClientRect().bottom;
                    updateCurrentHeader();
                }
            });
        });
    }

    function filterHeader(source, dest) {
        const clone = source.cloneNode(true);
        clone.querySelectorAll('mark').forEach(mark => {
            mark.replaceWith(...mark.childNodes);
        });
        dest.append(...clone.childNodes);
    }

    document.addEventListener('DOMContentLoaded', function() {
        const activeSection = document.querySelector('#mdbook-sidebar .active');
        if (activeSection === null) {
            return;
        }
        const main = document.getElementsByTagName('main')[0];
        headers = Array.from(main.querySelectorAll('h2, h3, h4, h5, h6'))
            .filter(h => h.id !== '' && h.children.length && h.children[0].tagName === 'A');
        if (headers.length === 0) {
            return;
        }
        const stack = [];
        const firstLevel = parseInt(headers[0].tagName.charAt(1));
        for (let i = 1; i < firstLevel; i++) {
            const ol = document.createElement('ol');
            ol.classList.add('section');
            if (stack.length > 0) {
                stack[stack.length - 1].ol.appendChild(ol);
            }
            stack.push({level: i + 1, ol: ol});
        }
        const foldLevel = 3;
        for (let i = 0; i < headers.length; i++) {
            const header = headers[i];
            const level = parseInt(header.tagName.charAt(1));
            const currentLevel = stack[stack.length - 1].level;
            if (level > currentLevel) {
                for (let nextLevel = currentLevel + 1; nextLevel <= level; nextLevel++) {
                    const ol = document.createElement('ol');
                    ol.classList.add('section');
                    const last = stack[stack.length - 1];
                    const lastChild = last.ol.lastChild;
                    if (lastChild) {
                        lastChild.appendChild(ol);
                    } else {
                        last.ol.appendChild(ol);
                    }
                    stack.push({level: nextLevel, ol: ol});
                }
            } else if (level < currentLevel) {
                while (stack.length > 1 && stack[stack.length - 1].level > level) {
                    stack.pop();
                }
            }
            const li = document.createElement('li');
            li.classList.add('header-item');
            li.classList.add('expanded');
            if (level < foldLevel) {
                li.classList.add('expanded');
            }
            const span = document.createElement('span');
            span.classList.add('chapter-link-wrapper');
            const a = document.createElement('a');
            span.appendChild(a);
            a.href = '#' + header.id;
            a.classList.add('header-in-summary');
            filterHeader(header.children[0], a);
            a.addEventListener('click', headerThresholdClick);
            const nextHeader = headers[i + 1];
            if (nextHeader !== undefined) {
                const nextLevel = parseInt(nextHeader.tagName.charAt(1));
                if (nextLevel > level && level >= foldLevel) {
                    const toggle = document.createElement('a');
                    toggle.classList.add('chapter-fold-toggle');
                    toggle.classList.add('header-toggle');
                    toggle.addEventListener('click', () => {
                        li.classList.toggle('expanded');
                    });
                    const toggleDiv = document.createElement('div');
                    toggleDiv.textContent = '❱';
                    toggle.appendChild(toggleDiv);
                    span.appendChild(toggle);
                    headerToggles.push(li);
                }
            }
            li.appendChild(span);
            const currentParent = stack[stack.length - 1];
            currentParent.ol.appendChild(li);
        }
        const onThisPage = document.createElement('div');
        onThisPage.classList.add('on-this-page');
        onThisPage.append(stack[0].ol);
        const activeItemSpan = activeSection.parentElement;
        activeItemSpan.after(onThisPage);
    });

    document.addEventListener('DOMContentLoaded', reloadCurrentHeader);
    document.addEventListener('scroll', reloadCurrentHeader, { passive: true });
})();
`
