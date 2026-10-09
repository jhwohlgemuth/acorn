const FAIR_TOOL = "acorn_assess_fair_metadata";
const CROSSWALK_TOOL = "acorn_crosswalk_metadata";
const DIFF_TOOL = "acorn_diff_content";
const DISCOVERY_TOOL = "acorn_discover_persistent_identifiers";
const IDENTIFIER_TOOL = "acorn_inspect_persistent_identifier";
const METADATA_TOOL = "acorn_validate_metadata_standard";
const READABILITY_TOOL = "acorn_check_readability";
const VALIDATION_TOOL = "acorn_validate_research_activity";
const ACORN_TOOL_NAMES = Object.freeze([
    FAIR_TOOL,
    READABILITY_TOOL,
    CROSSWALK_TOOL,
    DIFF_TOOL,
    DISCOVERY_TOOL,
    IDENTIFIER_TOOL,
    METADATA_TOOL,
    VALIDATION_TOOL,
]);
let serializeToolInputs = false;
const DATACITE_PROJECT = '{"meta":{"archive":false,"draft":true,"status":"active","identifier":"datacite-project","doi":"https://doi.org/10.5072/acorn-project","typedDates":{"dateType":"Issued","date":"2026-09-17T12:30:00-04:00"},"organizations":{"name":"Example Repository","identifiers":[{"value":"https://ror.org/01qz5mb56","scheme":"ROR","schemeUri":"https://ror.org"}],"roles":[{"organization":{"value":"https://ror.org/01qz5mb56","scheme":"ROR","schemeUri":"https://ror.org"},"role":"Publisher"}]}},"title":"DataCite Project activity","contact":{"jobTitle":"Researcher","givenName":"Jane","familyName":"Doe","email":"first_last@example.com","telephone":"123-456-7890","url":"https://example.com","organization":"Some Organization"}}';

function assert(condition, message) {
    if (!condition) {
        throw new Error(message);
    }
}

function parseExecutionResult(value) {
    assert(typeof value === "string", "executeTool did not return a serialized result");
    return JSON.parse(value);
}

function registeredSchema(tool) {
    return typeof tool.inputSchema === "string" ? JSON.parse(tool.inputSchema) : tool.inputSchema;
}

async function executeNativeTool(modelContext, tool, input, options) {
    try {
        return await modelContext.executeTool(tool, serializeToolInputs ? JSON.stringify(input) : input, options);
    } catch (error) {
        if (!serializeToolInputs && error?.message?.startsWith("Failed to parse input")) {
            serializeToolInputs = true;
            return modelContext.executeTool(tool, JSON.stringify(input), options);
        }
        throw error;
    }
}

async function maximumExecutionTime(modelContext, tool, input, count = 20) {
    let maximum = 0;
    for (let index = 0; index < count; index += 1) {
        const started = performance.now();
        parseExecutionResult(await executeNativeTool(modelContext, tool, input));
        maximum = Math.max(maximum, performance.now() - started);
    }
    return maximum;
}

async function verifyPermissionsPolicyDenial() {
    const frame = document.createElement("iframe");
    frame.allow = "tools 'none'";
    frame.hidden = true;
    frame.srcdoc = "<!doctype html><title>WebMCP policy probe</title>";
    const loaded = new Promise((resolve) => frame.addEventListener("load", resolve, { once: true }));
    document.body.append(frame);
    await loaded;
    const controller = new AbortController();
    let denied = false;
    try {
        await frame.contentDocument.modelContext.registerTool(
            {
                description: "Verify that an iframe can explicitly deny WebMCP tools.",
                execute: async () => ({ ok: true }),
                name: "acorn_permissions_policy_probe",
            },
            { signal: controller.signal },
        );
    } catch (error) {
        denied = error?.name === "NotAllowedError";
    } finally {
        controller.abort();
        frame.remove();
    }
    assert(denied, "The tools Permissions Policy did not reject registration");
}

/** Run deterministic assertions against a native WebMCP implementation. */
export async function runNativeSmoke({ adapter, installation }) {
    const capability = document.querySelector("#capability-status");
    const output = document.querySelector("#smoke-output");
    const tabs = [...document.querySelectorAll('[role="tab"]')];
    assert(tabs.length === 8, "The demo does not expose one tab per ACORN tool");
    assert(tabs.filter((tab) => tab.getAttribute("aria-selected") === "true").length === 1, "The demo must select exactly one tool tab");
    tabs.forEach((tab) => {
        const panel = document.querySelector(`#${tab.getAttribute("aria-controls")}`);
        assert(panel?.querySelector("form") !== null, `${tab.id} has no form`);
        assert(panel?.querySelector('button[type="submit"]') !== null, `${tab.id} has no submit button`);
        assert(panel?.querySelector(".output") !== null, `${tab.id} has no output field`);
        assert(panel?.querySelector('[role="status"]') !== null, `${tab.id} has no live status`);
    });
    document.querySelector("#validation-tab").dispatchEvent(new KeyboardEvent("keydown", { bubbles: true, key: "ArrowRight" }));
    assert(!document.querySelector("#metadata-panel").hidden, "Arrow-key navigation did not reveal the next tab panel");
    document.querySelector("#identifier-tab").click();
    assert(!document.querySelector("#identifier-panel").hidden, "Identifier tab did not reveal its panel");
    assert(document.querySelector("#validation-panel").hidden, "Identifier tab did not hide the validation panel");
    document.querySelector("#validation-tab").click();
    assert(await installation.ready === "installed", "WebMCP installation is unavailable");
    assert(typeof document.modelContext?.getTools === "function", "getTools is unavailable");
    assert(typeof document.modelContext?.executeTool === "function", "executeTool is unavailable");
    const tools = await document.modelContext.getTools();
    const acornTools = tools.filter(({ name }) => ACORN_TOOL_NAMES.includes(name));
    assert(acornTools.length === 8, "ACORN tools were not discovered");
    assert(JSON.stringify(acornTools.map(({ name }) => name)) === JSON.stringify(ACORN_TOOL_NAMES), "ACORN tools are not ordered by name");
    acornTools.forEach((tool) => {
        assert(registeredSchema(tool)?.additionalProperties === false, `${tool.name} accepts unknown fields`);
        assert(tool.annotations.readOnlyHint === true, `${tool.name} is not marked read-only`);
        assert(tool.annotations.untrustedContentHint === true, `${tool.name} output is not marked untrusted`);
    });
    const validationTool = acornTools.find(({ name }) => name === VALIDATION_TOOL);
    const validationInput = { json: "{\"title\":1}" };
    const validation = parseExecutionResult(await executeNativeTool(document.modelContext, validationTool, validationInput));
    assert(validation.ok && validation.result.valid === false, "Validation execution did not return invalid metadata diagnostics");
    assert(JSON.stringify(validation) === JSON.stringify(await adapter.validateResearchActivity(validationInput)), "Manual and tool validation differ");
    const metadataTool = acornTools.find(({ name }) => name === METADATA_TOOL);
    const metadataInput = { content: "A plain metadata document.", format: "text", standard: "text" };
    const metadata = parseExecutionResult(await executeNativeTool(document.modelContext, metadataTool, metadataInput));
    assert(metadata.ok && metadata.result.valid === true, "Metadata-standard validation did not accept plain text");
    assert(JSON.stringify(metadata) === JSON.stringify(await adapter.validateMetadataStandard(metadataInput)), "Manual and tool metadata validation differ");
    const discoveryTool = acornTools.find(({ name }) => name === DISCOVERY_TOOL);
    const discoveryInput = { text: "See https://doi.org/10.1000/182 and RRID:SCR_003512." };
    const discovery = parseExecutionResult(await executeNativeTool(document.modelContext, discoveryTool, discoveryInput));
    assert(discovery.ok && discovery.result.count === 2, "Identifier discovery did not return both persistent identifiers");
    assert(JSON.stringify(discovery) === JSON.stringify(await adapter.discoverPersistentIdentifiers(discoveryInput)), "Manual and tool identifier discovery differ");
    const identifierTool = acornTools.find(({ name }) => name === IDENTIFIER_TOOL);
    const identifierInput = { value: "https://doi.org/10.1000/182" };
    const identifier = parseExecutionResult(await executeNativeTool(document.modelContext, identifierTool, identifierInput));
    assert(identifier.ok && identifier.result.kind === "doi", "Identifier execution did not classify a DOI");
    assert(JSON.stringify(identifier) === JSON.stringify(await adapter.inspectPersistentIdentifier(identifierInput)), "Manual and tool identifier results differ");
    const ambiguousIdentifier = parseExecutionResult(
        await executeNativeTool(document.modelContext, identifierTool, { value: "0000-0002-2057-9115" }),
    );
    assert(
        ambiguousIdentifier.ok && ambiguousIdentifier.result.status === "ambiguous",
        "Identifier execution did not report an ambiguous name identifier",
    );
    const explicitIdentifier = parseExecutionResult(
        await executeNativeTool(document.modelContext, identifierTool, { kind: "orcid", value: "0000-0002-2057-9115" }),
    );
    assert(explicitIdentifier.ok && explicitIdentifier.result.status === "valid", "Explicit ORCID inspection did not resolve ambiguity");
    const diffTool = acornTools.find(({ name }) => name === DIFF_TOOL);
    const diffInput = { after: '{"title":"After"}', before: '{"title":"Before"}', mode: "json-patch" };
    const diff = parseExecutionResult(await executeNativeTool(document.modelContext, diffTool, diffInput));
    assert(diff.ok && diff.result.operations.length === 1, "JSON Patch diff did not return one operation");
    assert(JSON.stringify(diff) === JSON.stringify(await adapter.diffContent(diffInput)), "Manual and tool diff results differ");
    const fairTool = acornTools.find(({ name }) => name === FAIR_TOOL);
    const fairInput = { json: '{"title":"Example","doi":"10.1000/182"}', resource: "memory://example", standard: "datacite" };
    const fair = parseExecutionResult(await executeNativeTool(document.modelContext, fairTool, fairInput));
    assert(fair.ok && fair.result.findings.length === 41, "FAIR assessment did not return all RDA indicators");
    assert(JSON.stringify(fair) === JSON.stringify(await adapter.assessFairMetadata(fairInput)), "Manual and tool FAIR results differ");
    const crosswalkTool = acornTools.find(({ name }) => name === CROSSWALK_TOOL);
    const crosswalkInput = { content: DATACITE_PROJECT, format: "json", source: "research-activity-data", target: "datacite" };
    const crosswalk = parseExecutionResult(await executeNativeTool(document.modelContext, crosswalkTool, crosswalkInput));
    assert(crosswalk.ok && crosswalk.result.target === "datacite", "Typed RAD to DataCite crosswalk did not return target metadata");
    assert(JSON.stringify(crosswalk) === JSON.stringify(await adapter.crosswalkMetadata(crosswalkInput)), "Manual and tool crosswalk results differ");
    const readabilityTool = acornTools.find(({ name }) => name === READABILITY_TOOL);
    const readabilityInput = { metric: "fkgl", text: "This is a complete sentence." };
    const readability = parseExecutionResult(await executeNativeTool(document.modelContext, readabilityTool, readabilityInput));
    assert(readability.ok && readability.result.checks.length === 1, "Readability execution did not return one check");
    assert(JSON.stringify(readability) === JSON.stringify(await adapter.checkReadability(readabilityInput)), "Manual and tool readability differ");
    for (const metric of ["ari", "cli", "fkgl", "fres", "gfi", "lix", "smog"]) {
        const metricResult = parseExecutionResult(
            await executeNativeTool(document.modelContext, readabilityTool, { metric, text: "Unicode prose is clear: café." }),
        );
        assert(metricResult.ok && metricResult.result.checks[0].metric === metric, `${metric} did not preserve its result`);
    }
    const hostile = parseExecutionResult(
        await executeNativeTool(document.modelContext, validationTool, { json: '{"<img src=x onerror=alert(1)>":' }),
    );
    assert(hostile.ok && hostile.result.valid === false, "Malformed hostile JSON did not return diagnostics");
    assert(document.querySelector("img") === null, "Untrusted content created an HTML element");
    const identifierOverLimit = parseExecutionResult(
        await executeNativeTool(document.modelContext, identifierTool, { value: "x".repeat(2 * 1024 + 1) }),
    );
    assert(!identifierOverLimit.ok && identifierOverLimit.error.code === "input_too_large", "Identifier byte limit was not enforced");
    const maximumIdentifierMs = await maximumExecutionTime(
        document.modelContext,
        identifierTool,
        { value: "x".repeat(2 * 1024) },
    );
    const maximumValidationMs = await maximumExecutionTime(
        document.modelContext,
        validationTool,
        { json: `{}`.padEnd(256 * 1024, " ") },
    );
    const readabilityText = "This is a complete sentence. ".repeat(700).slice(0, 16 * 1024);
    const maximumReadabilityMs = await maximumExecutionTime(
        document.modelContext,
        readabilityTool,
        { metric: "fkgl", text: readabilityText },
    );
    assert(maximumIdentifierMs < 100, `Identifier inspection exceeded the 100 ms gate: ${maximumIdentifierMs.toFixed(2)} ms`);
    assert(maximumValidationMs < 100, `Validation exceeded the 100 ms gate: ${maximumValidationMs.toFixed(2)} ms`);
    assert(maximumReadabilityMs < 100, `Readability exceeded the 100 ms gate: ${maximumReadabilityMs.toFixed(2)} ms`);
    assert(document.querySelector("#identifier-output").value.length > 2, "Identifier output was not routed to its tool tab");
    assert(document.querySelector("#metadata-output").value.length > 2, "Metadata validation output was not routed to its tool tab");
    assert(document.querySelector("#discovery-output").value.length > 2, "Identifier discovery output was not routed to its tool tab");
    assert(document.querySelector("#diff-output").value.length > 2, "Diff output was not routed to its tool tab");
    assert(document.querySelector("#fair-output").value.length > 2, "FAIR output was not routed to its tool tab");
    assert(document.querySelector("#crosswalk-output").value.length > 2, "Crosswalk output was not routed to its tool tab");
    assert(document.querySelector("#readability-output").value.length > 2, "Readability output was not routed to its tool tab");
    assert(document.querySelector("#validation-output").value.length > 2, "Validation output was not routed to its tool tab");
    await verifyPermissionsPolicyDenial();
    const controller = new AbortController();
    controller.abort();
    let cancelled = false;
    try {
        await executeNativeTool(
            document.modelContext,
            readabilityTool,
            { metric: "fkgl", text: "This is a complete sentence." },
            { signal: controller.signal },
        );
    } catch {
        cancelled = true;
    }
    assert(cancelled, "A pre-cancelled execution was not rejected");
    installation.dispose();
    await Promise.resolve();
    const remaining = await document.modelContext.getTools();
    assert(
        !remaining.some(({ name }) => ACORN_TOOL_NAMES.includes(name)),
        "Disposal left ACORN tools registered",
    );
    capability.textContent = "Native WebMCP smoke passed; its ACORN tool registrations were disposed.";
    output.textContent = [
        "PASS: registration, discovery, schemas, annotations, execution, serialization, manual parity, permissions policy, bounds, timing, and cleanup.",
        `Browser: ${navigator.userAgent}`,
        `Maximum identifier time (20 × 2 KiB): ${maximumIdentifierMs.toFixed(2)} ms`,
        `Maximum validation time (20 × 256 KiB): ${maximumValidationMs.toFixed(2)} ms`,
        `Maximum readability time (20 × 16 KiB): ${maximumReadabilityMs.toFixed(2)} ms`,
    ].join("\n");
}
