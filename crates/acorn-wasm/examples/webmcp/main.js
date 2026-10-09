import init, {
    assessFairMetadata,
    checkReadability,
    crosswalkMetadata,
    diffContent,
    discoverPersistentIdentifiers,
    inspectPersistentIdentifier,
    validateMetadataStandard,
    validateResearchActivity,
} from "../pkg/acorn.js";
import { createAcornWebMcpAdapter } from "../pkg/webmcp.js";

const CAPABILITY_MESSAGES = Object.freeze({
    disposed: "WebMCP installation was disposed; the manual controls still use the same ACORN adapter.",
    failed: "WebMCP tool registration failed; the manual controls still use the same ACORN adapter.",
    installed: "WebMCP tools are installed for this page.",
    unsupported: "WebMCP is unavailable; the manual controls still use the same ACORN adapter.",
});
const capabilityStatus = document.querySelector("#capability-status");
const submitButtons = [...document.querySelectorAll('form button[type="submit"]')];
const tabs = [...document.querySelectorAll('[role="tab"]')];
const toolViews = Object.freeze({
    acorn_assess_fair_metadata: {
        output: document.querySelector("#fair-output"),
        status: document.querySelector("#fair-status"),
    },
    acorn_check_readability: {
        output: document.querySelector("#readability-output"),
        status: document.querySelector("#readability-status"),
    },
    acorn_crosswalk_metadata: {
        output: document.querySelector("#crosswalk-output"),
        status: document.querySelector("#crosswalk-status"),
    },
    acorn_diff_content: {
        output: document.querySelector("#diff-output"),
        status: document.querySelector("#diff-status"),
    },
    acorn_discover_persistent_identifiers: {
        output: document.querySelector("#discovery-output"),
        status: document.querySelector("#discovery-status"),
    },
    acorn_inspect_persistent_identifier: {
        output: document.querySelector("#identifier-output"),
        status: document.querySelector("#identifier-status"),
    },
    acorn_validate_metadata_standard: {
        output: document.querySelector("#metadata-output"),
        status: document.querySelector("#metadata-status"),
    },
    acorn_validate_research_activity: {
        output: document.querySelector("#validation-output"),
        status: document.querySelector("#validation-status"),
    },
});

function activateTab(selected) {
    tabs.forEach((tab) => {
        const active = tab === selected;
        tab.setAttribute("aria-selected", String(active));
        tab.tabIndex = active ? 0 : -1;
        document.querySelector(`#${tab.getAttribute("aria-controls")}`).hidden = !active;
    });
}

function render(tool, output, message) {
    const view = toolViews[tool];
    view.status.textContent = message;
    view.output.value = JSON.stringify(output, null, 2);
}

function tabIndexForKey(key, index) {
    return {
        ArrowLeft: (index - 1 + tabs.length) % tabs.length,
        ArrowRight: (index + 1) % tabs.length,
        End: tabs.length - 1,
        Home: 0,
    }[key] ?? null;
}

tabs.forEach((tab, index) => {
    tab.addEventListener("click", () => activateTab(tab));
    tab.addEventListener("keydown", (event) => {
        const targetIndex = tabIndexForKey(event.key, index);
        if (targetIndex !== null) {
            event.preventDefault();
            activateTab(tabs[targetIndex]);
            tabs[targetIndex].focus();
        }
    });
});

async function initialize() {
    await init();
    const adapter = createAcornWebMcpAdapter(
        {
            assessFairMetadata,
            checkReadability,
            crosswalkMetadata,
            diffContent,
            discoverPersistentIdentifiers,
            inspectPersistentIdentifier,
            validateMetadataStandard,
            validateResearchActivity,
        },
        {
            onInvocation: (event) => {
                if (event.phase === "started") {
                    toolViews[event.tool].status.textContent = `Running ${event.tool}…`;
                } else {
                    render(event.tool, event.output, `${event.tool} completed.`);
                }
            },
        },
    );
    submitButtons.forEach((button) => {
        button.disabled = false;
    });
    document.querySelector("#fair-form").addEventListener("submit", async (event) => {
        event.preventDefault();
        const resource = document.querySelector("#fair-resource").value.trim();
        const input = {
            json: document.querySelector("#fair-json").value,
            standard: document.querySelector("#fair-standard").value,
            ...(resource.length === 0 ? {} : { resource }),
        };
        render("acorn_assess_fair_metadata", await adapter.assessFairMetadata(input), "Manual FAIR assessment completed.");
    });
    document.querySelector("#readability-form").addEventListener("submit", async (event) => {
        event.preventDefault();
        const input = {
            metric: document.querySelector("#metric").value,
            text: document.querySelector("#prose").value,
        };
        render("acorn_check_readability", await adapter.checkReadability(input), "Manual readability check completed.");
    });
    document.querySelector("#crosswalk-form").addEventListener("submit", async (event) => {
        event.preventDefault();
        const [source, target] = document.querySelector("#crosswalk-pair").value.split(":");
        const contactText = document.querySelector("#crosswalk-contact").value.trim();
        const localIdentifier = document.querySelector("#crosswalk-identifier").value.trim();
        let contact;
        try {
            contact = contactText.length === 0 ? undefined : JSON.parse(contactText);
        } catch {
            render(
                "acorn_crosswalk_metadata",
                { error: { code: "invalid_arguments", message: "Contact seed must be valid JSON." }, ok: false },
                "Manual crosswalk rejected invalid contact JSON.",
            );
            return;
        }
        const input = {
            content: document.querySelector("#crosswalk-content").value,
            format: document.querySelector("#crosswalk-format").value,
            source,
            target,
            ...(contact === undefined ? {} : { contact }),
            ...(localIdentifier.length === 0 ? {} : { localIdentifier }),
        };
        render("acorn_crosswalk_metadata", await adapter.crosswalkMetadata(input), "Manual metadata crosswalk completed.");
    });
    document.querySelector("#diff-form").addEventListener("submit", async (event) => {
        event.preventDefault();
        const input = {
            after: document.querySelector("#diff-after").value,
            before: document.querySelector("#diff-before").value,
            mode: document.querySelector("#diff-mode").value,
        };
        render("acorn_diff_content", await adapter.diffContent(input), "Manual content diff completed.");
    });
    document.querySelector("#discovery-form").addEventListener("submit", async (event) => {
        event.preventDefault();
        const input = { text: document.querySelector("#discovery-text").value };
        render("acorn_discover_persistent_identifiers", await adapter.discoverPersistentIdentifiers(input), "Manual identifier discovery completed.");
    });
    document.querySelector("#identifier-form").addEventListener("submit", async (event) => {
        event.preventDefault();
        const kind = document.querySelector("#identifier-kind").value;
        const input = {
            value: document.querySelector("#identifier-value").value,
            ...(kind.length === 0 ? {} : { kind }),
        };
        render("acorn_inspect_persistent_identifier", await adapter.inspectPersistentIdentifier(input), "Manual identifier inspection completed.");
    });
    document.querySelector("#metadata-form").addEventListener("submit", async (event) => {
        event.preventDefault();
        const input = {
            content: document.querySelector("#metadata-content").value,
            format: document.querySelector("#metadata-format").value,
            standard: document.querySelector("#metadata-standard").value,
        };
        render("acorn_validate_metadata_standard", await adapter.validateMetadataStandard(input), "Manual metadata validation completed.");
    });
    document.querySelector("#validation-form").addEventListener("submit", async (event) => {
        event.preventDefault();
        const input = { json: document.querySelector("#research-json").value };
        render("acorn_validate_research_activity", await adapter.validateResearchActivity(input), "Manual validation completed.");
    });
    const installation = adapter.install();
    window.addEventListener("pagehide", () => installation.dispose(), { once: true });
    const status = await installation.ready.catch(() => "failed");
    capabilityStatus.textContent = CAPABILITY_MESSAGES[status];
    return { adapter, installation };
}

export const demoReady = initialize().catch((error) => {
    capabilityStatus.textContent = "ACORN WebAssembly initialization failed; no tools are available.";
    Object.values(toolViews).forEach(({ status }) => {
        status.textContent = "Initialization failed. Rebuild the package and reload this page.";
    });
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
