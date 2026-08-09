(function chapterNavigation() {
    function zoomOutImages() {
        for (const elem of Array.from(document.querySelectorAll('input.checkbox-img'))) {
            elem.checked = false;
        }
    }

    document.addEventListener('keydown', function(e) {
        if (e.altKey ||
            e.ctrlKey ||
            e.metaKey ||
            window.search && window.search.hasFocus() ||
            mdbook_something_else_has_focus(e)
        ) {
            return;
        }

        const html = document.querySelector('html');

        function next() {
            const nextButton = document.querySelector('.nav-chapters.next');
            if (nextButton) {
                window.location.href = nextButton.href;
            }
        }
        function prev() {
            const previousButton = document.querySelector('.nav-chapters.previous');
            if (previousButton) {
                window.location.href = previousButton.href;
            }
        }
        function showHelp() {
            const container = document.getElementById('mdbook-help-container');
            const overlay = document.getElementById('mdbook-help-popup');
            container.style.display = 'flex';

            // Clicking outside the popup will dismiss it.
            const mouseHandler = event => {
                if (overlay.contains(event.target)) {
                    return;
                }
                if (event.button !== 0) {
                    return;
                }
                event.preventDefault();
                event.stopPropagation();
                document.removeEventListener('mousedown', mouseHandler);
                hideHelp();
            };

            // Pressing esc will dismiss the popup.
            const escapeKeyHandler = event => {
                if (event.key === 'Escape') {
                    event.preventDefault();
                    event.stopPropagation();
                    document.removeEventListener('keydown', escapeKeyHandler, true);
                    hideHelp();
                }
            };
            document.addEventListener('keydown', escapeKeyHandler, true);
            document.getElementById('mdbook-help-container')
                .addEventListener('mousedown', mouseHandler);
        }
        function hideHelp() {
            document.getElementById('mdbook-help-container').style.display = 'none';
        }

        // Usually needs the Shift key to be pressed
        switch (e.key) {
        case '?':
            e.preventDefault();
            showHelp();
            break;
        case 'Escape':
            zoomOutImages();
            break;
        }

        // Rest of the keys are only active when the Shift key is not pressed
        if (e.shiftKey) {
            return;
        }

        switch (e.key) {
        case 'ArrowRight':
            e.preventDefault();
            if (html.dir === 'rtl') {
                prev();
            } else {
                next();
            }
            break;
        case 'ArrowLeft':
            e.preventDefault();
            if (html.dir === 'rtl') {
                next();
            } else {
                prev();
            }
            break;
        }
    });
})();
