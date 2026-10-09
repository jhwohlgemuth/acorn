import init, { checkReadability, validateResearchActivity } from "../pkg/acorn.js";
import { createAcornWebMcpAdapter } from "../pkg/webmcp.js";

const CAPABILITY_MESSAGES = Object.freeze({
    disposed: "WebMCP installation was disposed; the manual controls still use the same ACORN adapter.",
    failed: "WebMCP tool registration failed; the manual controls still use the same ACORN adapter.",
    installed: "WebMCP tools are installed for this page.",
    unsupported: "WebMCP is unavailable; the manual controls still use the same ACORN adapter.",
});
const capabilityStatus = document.querySelector("#capability-status");
const metric = document.querySelector("#metric");
const operationStatus = document.querySelector("#operation-status");
const prose = document.querySelector("#prose");
const readabilityButton = document.querySelector("#readability");
const researchJson = document.querySelector("#research-json");
const result = document.querySelector("#result");
const validateButton = document.querySelector("#validate");

function render(output, message) {
    operationStatus.textContent = message;
    result.textContent = JSON.stringify(output, null, 2);
}

async function initialize() {
    await init();
    const adapter = createAcornWebMcpAdapter(
        { checkReadability, validateResearchActivity },
        {
            onInvocation: (event) => {
                if (event.phase === "started") {
                    operationStatus.textContent = `Running ${event.tool}…`;
                } else {
                    render(event.output, `${event.tool} completed.`);
                }
            },
        },
    );
    readabilityButton.disabled = false;
    validateButton.disabled = false;
    readabilityButton.addEventListener("click", async () => {
        const output = await adapter.checkReadability({ metric: metric.value, text: prose.value });
        render(output, "Manual readability check completed.");
    });
    validateButton.addEventListener("click", async () => {
        const output = await adapter.validateResearchActivity({ json: researchJson.value });
        render(output, "Manual validation completed.");
    });
    const installation = adapter.install();
    window.addEventListener("pagehide", () => installation.dispose(), { once: true });
    const status = await installation.ready.catch(() => "failed");
    capabilityStatus.textContent = CAPABILITY_MESSAGES[status];
    return { adapter, installation };
}

export const demoReady = initialize().catch((error) => {
    capabilityStatus.textContent = "ACORN WebAssembly initialization failed; no tools are available.";
    operationStatus.textContent = "Initialization failed. Rebuild the package and reload this page.";
    throw error;
});

if (new URLSearchParams(window.location.search).has("smoke")) {
    document.querySelector("#smoke").hidden = false;
    demoReady
        .then(async (demo) => {
            const { runNativeSmoke } = await import("./smoke.js");
            return runNativeSmoke(demo);
        })
        .catch((error) => {
            document.querySelector("#smoke-output").textContent = `FAIL: ${error.message}`;
        });
}
