(function() {
    let shadowHost = null;
    let shadowInput = null;

    document.addEventListener('keypress', function(e) {
        if (e.key === 'x' || e.key === 'X') {
            if (shadowHost && shadowHost.isConnected) {
                shadowInput.focus();
                return;
            }

            shadowHost = document.createElement('div');
            shadowHost.id = 'shadow-input-host';
            shadowHost.style.cssText = 'position:fixed;top:50%;left:50%;transform:translate(-50%,-50%);z-index:9999;';

            document.body.appendChild(shadowHost);

            const shadowRoot = shadowHost.attachShadow({ mode: 'open' });

            shadowInput = document.createElement('input');
            shadowInput.type = 'text';
            shadowInput.id = 'shadow-input';
            shadowInput.placeholder = 'Shadow DOM input (press Escape to close)';
            shadowInput.style.cssText = 'font-size:1.2em;padding:8px;width:300px;';

            shadowRoot.appendChild(shadowInput);
            shadowInput.focus();

            shadowInput.addEventListener('keydown', function(e) {
                if (e.key === 'Escape') {
                    shadowHost.remove();
                    shadowHost = null;
                    shadowInput = null;
                }
            });
        }
    });
})();
