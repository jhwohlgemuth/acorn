// Explicit user-mediated output effects; no content leaves the browser.
window.acornDownload = (text) => {
    const url = URL.createObjectURL(new Blob([text], { type: "application/json;charset=utf-8" }));
    const link = document.createElement("a");
    link.href = url;
    link.download = "acorn-result.json";
    link.click();
    setTimeout(() => URL.revokeObjectURL(url), 1000);
};
window.acornCopy = async (text) => {
    const status = document.getElementById("output-status");
    try { await navigator.clipboard.writeText(text); if (status) status.textContent = "Copied to clipboard."; }
    catch { if (status) status.textContent = "Clipboard unavailable. Use Download JSON or select the raw result."; }
};
async function installWebMcp() {
    const status = document.getElementById("webmcp-status");
    const moduleScript = document.querySelector('script[type="module"][src*="acorn-web"]');
    if (!status || !moduleScript) return;
    try {
        const [adapterModule, bindings] = await Promise.all([
            import(status.dataset.adapter), import(moduleScript.src),
        ]);
        const adapter = adapterModule.createAcornWebMcpAdapter(bindings);
        const installation = adapter.install();
        status.textContent = `WebMCP: ${await installation.ready}`;
        window.addEventListener("pagehide", (event) => {
            if (!event.persisted) installation.dispose();
        });
    } catch (error) {
        status.textContent = "WebMCP: failed (manual tools available)";
        console.error("WebMCP installation failed", error);
    }
}
installWebMcp();
