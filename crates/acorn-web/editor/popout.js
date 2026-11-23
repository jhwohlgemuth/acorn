export function createEditorPopouts(environment = window) {
    const { document, location } = environment;
    const popouts = new Map();
    const popoutId = new URLSearchParams(location.search).get("acorn-editor") || /^acorn-editor:([^:]+):/.exec(environment.name)?.[1];
    if (popoutId) document.body.classList.add("editor-popout");
    const route = () => `${location.pathname}${location.hash}`;
    function refresh(host) {
        const id = host.querySelector("textarea")?.id;
        if (!id) return;
        const record = popouts.get(id);
        const active = record && !record.popup.closed && record.route === route();
        const placeholder = host.querySelector(".editor-popout-placeholder");
        if (!active) {
            host.classList.remove("is-popped-out");
            placeholder?.remove();
            if (record?.popup.closed) popouts.delete(id);
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
    }
    function refreshAll() {
        document.querySelectorAll(".code-editor").forEach(refresh);
    }
    function open(id) {
        const source = document.getElementById(id);
        const host = source?.closest(".code-editor");
        if (!host) return;
        const existing = popouts.get(id);
        if (existing && !existing.popup.closed) { existing.popup.focus(); return; }
        const url = new URL(location.href);
        url.searchParams.set("acorn-editor", id);
        const popup = environment.open(url.href, `acorn-editor:${id}:${environment.crypto.randomUUID()}`);
        if (!popup) return;
        popouts.set(id, { popup, route: route(), url: url.href });
        refresh(host);
    }
    function connect() {
        const sync = environment.acornWindowSync;
        if (popoutId && environment.opener) {
            environment.addEventListener("pageshow", () => sync.postToOpener({ type: "acorn-editor-opened", id: popoutId, route: route(), url: location.href }));
            environment.addEventListener("pagehide", () => sync.postToOpener({ type: "acorn-editor-closed", id: popoutId }));
        }
        sync.onDirect(event => {
            if (event.data?.type === "acorn-editor-opened" && event.source?.opener === environment) {
                popouts.set(event.data.id, { popup: event.source, route: event.data.route, url: event.data.url });
                refreshAll();
            } else if (event.data?.type === "acorn-editor-closed") {
                const record = popouts.get(event.data.id);
                if (record?.popup === event.source) {
                    popouts.delete(event.data.id);
                    refreshAll();
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
