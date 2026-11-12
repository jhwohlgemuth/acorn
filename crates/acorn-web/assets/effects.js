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

function createTheme(preference, apply) {
    let systemDark = preference;
    let explicit = null;
    const sync = () => apply(explicit ?? systemDark, explicit === null);
    return {
        sync,
        reset() { explicit = null; sync(); },
        toggle() { explicit = !(explicit ?? systemDark); sync(); },
        update(dark) { systemDark = dark; sync(); },
    };
}
function createNotifier(show, schedule = setTimeout, cancel = clearTimeout) {
    let timer;
    const dismiss = () => { cancel(timer); timer = undefined; show(""); };
    return {
        dismiss,
        notify(message) { dismiss(); show(message); timer = schedule(dismiss, 6000); },
    };
}
function createTour(loadDriver, environment) {
    let active;
    let generation = 0;
    let silent = false;
    const cancel = () => {
        generation += 1;
        silent = true;
        active?.destroy();
        active = undefined;
    };
    return {
        cancel,
        async start() {
            cancel();
            const request = generation;
            const trigger = environment.find("#tour-start");
            const steps = [
                ["#home-actions", "Start with an example", "Try sample data to explore the tools without opening a file."],
                ["#skill-entry", "Use the ACORN Skill", "Read the release's workflows for your coding agent."],
                ["#local-entry", "Open your data", "Choose or drop one local UTF-8 file. Loading never runs a tool."],
                ["#privacy-note", "Keep control", "Your content stays in this tab. Choose a tool and run it explicitly."],
            ].map(([element, title, description]) => ({ element, popover: { title, description } }));
            try {
                const { driver } = await loadDriver();
                if (request !== generation) return;
                if (!trigger?.isConnected || steps.some((step) => !environment.find(step.element))) {
                    environment.notify("The tour is unavailable on this page. Return Home and try again.");
                    return;
                }
                silent = false;
                active = driver({
                    steps,
                    animate: !environment.reducedMotion(),
                    smoothScroll: false,
                    allowClose: true,
                    allowKeyboardControl: true,
                    disableActiveInteraction: true,
                    showProgress: true,
                    progressText: "{{current}} of {{total}}",
                    nextBtnText: "Next",
                    prevBtnText: "Back",
                    doneBtnText: "Done",
                    closeBtnLabel: "Close tour",
                    popoverClass: `acorn-tour ${environment.theme()}`,
                    onDestroyed(_element, _step, { reason }) {
                        active = undefined;
                        if (!silent) {
                            if (trigger.isConnected) trigger.focus();
                            environment.notify(reason === "done" ? "Tour complete. Choose sample data or open a file." : "Tour closed. You can restart it from Home.");
                        }
                    },
                });
                active.drive();
            } catch {
                if (request === generation) environment.notify("The tour could not load. All research tools remain available.");
            }
        },
    };
}
function installChrome() {
    const workspace = document.querySelector(".workspace");
    if (!workspace || !window.matchMedia) return;
    const preference = window.matchMedia("(prefers-color-scheme: dark)");
    const theme = createTheme(preference.matches, (dark, system) => {
        workspace.dataset.theme = dark ? "dark" : "light";
        const button = document.getElementById("theme-toggle");
        button?.setAttribute("aria-pressed", String(dark));
        button?.setAttribute("aria-label", dark ? "Dark theme" : "Light theme");
        const label = document.getElementById("theme-label");
        if (label) label.textContent = dark ? "Dark" : "Light";
        const mode = document.getElementById("theme-mode");
        if (mode) mode.textContent = system ? "Following your system preference." : "Using your explicit choice for this session.";
    });
    const notifications = createNotifier((message) => {
        const region = document.getElementById("notification");
        const text = document.getElementById("notification-text");
        if (region) region.hidden = !message;
        if (text) text.textContent = message;
    });
    const tour = createTour(() => import(document.getElementById("tour-start").dataset.driver), {
        find: (selector) => document.querySelector(selector),
        notify: notifications.notify,
        reducedMotion: () => window.matchMedia("(prefers-reduced-motion: reduce)").matches,
        theme: () => workspace.dataset.theme,
    });
    window.acornChrome = { theme, notifications, tour };
    preference.addEventListener("change", (event) => theme.update(event.matches));
    window.addEventListener("resize", () => {
        const toggle = document.getElementById("nav-toggle");
        if (toggle?.getAttribute("aria-expanded") === "true" && window.getComputedStyle(toggle.parentElement).display === "none") {
            // Close through Dioxus so drawer state, aria-expanded and main inertness agree.
            toggle.click();
        }
    });
    window.addEventListener("pagehide", () => { tour.cancel(); notifications.dismiss(); });
    theme.sync();
}
installChrome();
