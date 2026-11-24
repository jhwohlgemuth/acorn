export function createEditorPopouts(environment = window) {
    const { document, location } = environment;
    const popouts = new Map();
    const popoutId = environment.acornInitialEditorId || new URLSearchParams(location.search).get("acorn-editor") || /^acorn-editor:([^:]+):/.exec(environment.name)?.[1];
    if (popoutId) document.body.classList.add("editor-popout");
    const route = () => location.pathname;
    const key = (id, page = route()) => `${page}:${id}`;
    function refresh(host) {
        const id = host.querySelector("textarea")?.id;
        if (!id) return;
        const record = popouts.get(key(id));
        const active = record && !record.popup.closed && record.route === route();
        const placeholder = host.querySelector(".editor-popout-placeholder");
        if (!active) {
            host.classList.remove("is-popped-out");
            placeholder?.remove();
            if (record?.popup.closed) popouts.delete(key(id));
            return;
        }
        host.classList.add("is-popped-out");
        if (placeholder) return;
        const panel = document.createElement("div");
        panel.className = "editor-popout-placeholder";
        panel.setAttribute("role", "status");
        const message = document.createElement("p");
        message.textContent = "Code editor is open in another tab.";
        const link = document.createElement("a");
        link.href = record.url;
        link.target = "_blank";
        link.textContent = "Open code editor tab ↗";
        link.addEventListener("click", event => {
            if (record.popup.closed) return;
            event.preventDefault();
            record.popup.focus();
        });
        panel.append(message, link);
        host.append(panel);
        record.popup.postMessage({ type: "acorn-editor-input-request", id, route: route() }, location.origin);
    }
    function refreshAll() {
        document.querySelectorAll(".code-editor").forEach(refresh);
    }
    function open(id, state) {
        const source = document.getElementById(id);
        const host = source?.closest(".code-editor");
        if (!host) return;
        const existing = popouts.get(key(id));
        if (existing && !existing.popup.closed) { existing.popup.focus(); return; }
        const url = new URL(location.href);
        url.searchParams.set("acorn-editor", id);
        try { if (state) environment.sessionStorage.setItem("acorn.editor.handoff.v1", state); } catch {}
        const popup = environment.open(url.href, `acorn-editor:${id}:${environment.crypto.randomUUID()}`);
        try { environment.sessionStorage.removeItem("acorn.editor.handoff.v1"); } catch {}
        if (!popup) return;
        popouts.set(key(id), { popup, route: route(), url: url.href });
        refresh(host);
    }
    function connect() {
        const sync = environment.acornWindowSync;
        let applyingInput = false;
        document.addEventListener("input", event => {
            const input = event.target;
            const id = input?.id;
            if (applyingInput || input?.tagName !== "TEXTAREA" || !id || !input.closest(".code-editor")) return;
            const message = { type: "acorn-editor-input", id, route: route(), value: input.value };
            if (popoutId === id && environment.opener) sync.postToOpener(message);
            else {
                const record = popouts.get(key(id));
                if (record && !record.popup.closed) record.popup.postMessage(message, location.origin);
            }
        });
        if (popoutId && environment.opener) {
            environment.addEventListener("pageshow", () => sync.postToOpener({ type: "acorn-editor-opened", id: popoutId, route: route(), url: location.href }));
            environment.addEventListener("pagehide", () => sync.postToOpener({ type: "acorn-editor-closed", id: popoutId, route: route() }));
        }
        sync.onDirect(event => {
            if (event.data?.type === "acorn-editor-opened" && event.source?.opener === environment) {
                popouts.set(key(event.data.id, event.data.route), { popup: event.source, route: event.data.route, url: event.data.url });
                refreshAll();
            } else if (event.data?.type === "acorn-editor-closed") {
                const record = popouts.get(key(event.data.id, event.data.route));
                if (record?.popup === event.source) {
                    popouts.delete(key(event.data.id, event.data.route));
                    refreshAll();
                }
            } else if (event.data?.type === "acorn-editor-input-request" && popoutId === event.data.id && event.data.route === route() && event.source === environment.opener) {
                const input = document.getElementById(popoutId);
                if (input?.tagName === "TEXTAREA") sync.postToOpener({ type: "acorn-editor-input", id: popoutId, route: route(), value: input.value });
            } else if (event.data?.type === "acorn-editor-input") {
                const { id, value } = event.data;
                const record = popouts.get(key(id, event.data.route));
                const paired = event.data.route === route() && (popoutId ? id === popoutId && event.source === environment.opener : record?.popup === event.source);
                const input = paired && document.getElementById(id);
                if (input?.tagName === "TEXTAREA" && input.closest(".code-editor") && typeof value === "string" && input.value !== value) {
                    applyingInput = true;
                    try {
                        environment.acornEditors?.sync(id, value, input.closest(".code-editor").dataset.language, input.getAttribute("aria-label"));
                        input.value = value;
                        input.dispatchEvent(new Event("input", { bubbles: true }));
                    } finally { applyingInput = false; }
                }
            }
        });
    }
    if (environment.acornWindowSync) connect();
    else environment.addEventListener("acorn-window-sync-ready", connect, { once: true });
    environment.setInterval(() => {
        if (Array.from(popouts.values()).some(({ popup }) => popup.closed)) refreshAll();
    }, 500);
    return { isPopout: id => popoutId === id, open, refresh };
}
