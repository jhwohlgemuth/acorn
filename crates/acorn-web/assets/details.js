(() => {
    let trigger;
    let previousOverflow;
    let width = 480;
    let onClose;
    const dialog = () => document.getElementById("details-drawer");
    function resize(panel, requested, remember = true) {
        if (remember) width = Math.round(Math.max(360, Math.min(requested, 900)));
        const size = Math.min(width, innerWidth);
        panel.style.setProperty("--details-width", `${size}px`);
        const grip = panel.querySelector(".details-resize");
        grip.setAttribute("aria-valuenow", String(size));
        grip.setAttribute("aria-valuemax", String(Math.min(900, innerWidth)));
        grip.setAttribute("aria-valuemin", String(Math.min(360, innerWidth)));
        grip.setAttribute("aria-valuetext", `${size} pixels`);
    }
    function setup(panel) {
        if (panel.dataset.ready) return;
        panel.dataset.ready = "true";
        const grip = panel.querySelector(".details-resize");
        let drag;
        grip.addEventListener("pointerdown", event => {
            if (event.button !== 0 || !event.isPrimary) return;
            event.preventDefault();
            grip.focus();
            drag = { x: event.clientX, width: panel.getBoundingClientRect().width };
            grip.setPointerCapture(event.pointerId);
        });
        grip.addEventListener("pointermove", event => {
            if (drag && grip.hasPointerCapture(event.pointerId)) resize(panel, drag.width + drag.x - event.clientX);
        });
        grip.addEventListener("pointerup", event => { if (grip.hasPointerCapture(event.pointerId)) grip.releasePointerCapture(event.pointerId); });
        grip.addEventListener("lostpointercapture", () => { drag = undefined; });
        grip.addEventListener("keydown", event => {
            if (!['ArrowLeft', 'ArrowRight'].includes(event.key)) return;
            event.preventDefault();
            resize(panel, width + (event.key === 'ArrowLeft' ? 24 : -24));
        });
        panel.addEventListener("keydown", event => {
            const tab = event.target.closest('[role="tab"]');
            if (!tab || !['ArrowLeft', 'ArrowRight', 'Home', 'End'].includes(event.key)) return;
            event.preventDefault();
            const tabs = [...panel.querySelectorAll('[role="tab"]')];
            const index = tabs.indexOf(tab);
            const next = event.key === 'Home' ? 0 : event.key === 'End' ? tabs.length - 1 : (index + (event.key === 'ArrowRight' ? 1 : -1) + tabs.length) % tabs.length;
            tabs[next].click(); tabs[next].focus();
        });
        panel.addEventListener("click", event => {
            const rect = panel.getBoundingClientRect();
            if (event.target === panel && (event.clientX < rect.left || event.clientX > rect.right || event.clientY < rect.top || event.clientY > rect.bottom)) panel.close();
        });
        panel.addEventListener("close", () => {
            if (panel.open) return;
            if (previousOverflow !== undefined) document.body.style.overflow = previousOverflow;
            previousOverflow = undefined;
            if (trigger?.isConnected) trigger.focus({ preventScroll: true });
            trigger = undefined;
            onClose?.();
        });
    }
    window.acornDetails = {
        connect(callback) { onClose = callback; },
        rememberFocus() { if (!dialog()?.open) trigger = document.activeElement; },
        open() {
            const panel = dialog();
            if (!panel?.querySelector("#details-title")) return;
            setup(panel);
            resize(panel, width, false);
            if (!panel.open) {
                previousOverflow = document.body.style.overflow;
                document.body.style.overflow = "hidden";
                panel.showModal();
            }
            panel.querySelector(".details-body")?.scrollTo(0, 0);
            panel.querySelector(".details-close")?.focus({ preventScroll: true });
        },
        close(restore = true) { if (!restore) trigger = undefined; dialog()?.close(); },
    };
    window.addEventListener("resize", () => { const panel = dialog(); if (panel?.open) resize(panel, width, false); });
    window.dispatchEvent(new Event("acorn-details-ready"));
})();
