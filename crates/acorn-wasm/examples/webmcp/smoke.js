const READABILITY_TOOL = "acorn_check_readability";
const VALIDATION_TOOL = "acorn_validate_research_activity";
let serializeToolInputs = false;

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
    assert(await installation.ready === "installed", "WebMCP installation is unavailable");
    assert(typeof document.modelContext?.getTools === "function", "getTools is unavailable");
    assert(typeof document.modelContext?.executeTool === "function", "executeTool is unavailable");
    const tools = await document.modelContext.getTools();
    const acornTools = tools.filter(({ name }) => name === READABILITY_TOOL || name === VALIDATION_TOOL);
    assert(acornTools.length === 2, "ACORN tools were not discovered");
    assert(acornTools[0].name === READABILITY_TOOL && acornTools[1].name === VALIDATION_TOOL, "ACORN tools are not ordered by name");
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
    assert(maximumValidationMs < 100, `Validation exceeded the 100 ms gate: ${maximumValidationMs.toFixed(2)} ms`);
    assert(maximumReadabilityMs < 100, `Readability exceeded the 100 ms gate: ${maximumReadabilityMs.toFixed(2)} ms`);
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
    assert(!remaining.some(({ name }) => name === READABILITY_TOOL || name === VALIDATION_TOOL), "Disposal left ACORN tools registered");
    capability.textContent = "Native WebMCP smoke passed; its ACORN tool registrations were disposed.";
    output.textContent = [
        "PASS: registration, discovery, schemas, annotations, execution, serialization, manual parity, permissions policy, bounds, timing, and cleanup.",
        `Browser: ${navigator.userAgent}`,
        `Maximum validation time (20 × 256 KiB): ${maximumValidationMs.toFixed(2)} ms`,
        `Maximum readability time (20 × 16 KiB): ${maximumReadabilityMs.toFixed(2)} ms`,
    ].join("\n");
}
