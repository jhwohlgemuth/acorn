import assert from "node:assert/strict";
import test from "node:test";

import { createAcornWebMcpAdapter } from "./webmcp.js";

const readabilityValue = (overrides = {}) => ({
    checks: [
        {
            category: "readability",
            message: "FKGL = 4/12",
            metric: "fkgl",
            score: 4,
            severity: "info",
            success: true,
            threshold: 12,
            uri: undefined,
            ...overrides,
        },
    ],
    summary: [["Readability items found", "1"]],
});

const bindings = (overrides = {}) => ({
    checkReadability: () => readabilityValue(),
    validateResearchActivity: () => ({ diagnostics: [], valid: true, value: { private: true } }),
    ...overrides,
});

class FakeModelContext {
    constructor(rejectedTool) {
        this.rejectedTool = rejectedTool;
        this.tools = new Map();
    }

    async registerTool(tool, { signal } = {}) {
        if (tool.name === this.rejectedTool) {
            throw new Error("registration rejected");
        }
        if (signal?.aborted) {
            throw signal.reason;
        }
        this.tools.set(tool.name, tool);
        signal?.addEventListener("abort", () => this.tools.delete(tool.name), { once: true });
    }
}

class PendingModelContext extends FakeModelContext {
    constructor() {
        super();
        this.pending = new Promise((resolve) => {
            this.release = resolve;
        });
    }

    async registerTool(tool, options) {
        await this.pending;
        return super.registerTool(tool, options);
    }
}

test("argument validation enforces exact fields and UTF-8 byte limits", async () => {
    let calls = 0;
    const adapter = createAcornWebMcpAdapter(
        bindings({
            checkReadability: () => {
                calls += 1;
                return readabilityValue();
            },
        }),
    );
    assert.equal((await adapter.checkReadability({ metric: "fkgl" })).error.code, "invalid_arguments");
    assert.equal((await adapter.checkReadability({ extra: true, metric: "fkgl", text: "Text." })).error.code, "invalid_arguments");
    assert.equal((await adapter.checkReadability({ metric: "unknown", text: "Text." })).error.code, "invalid_arguments");
    assert.equal((await adapter.checkReadability({ metric: "fkgl", text: "é".repeat(8 * 1024) })).ok, true);
    assert.equal((await adapter.checkReadability({ metric: "fkgl", text: `${"é".repeat(8 * 1024)}x` })).error.code, "input_too_large");
    assert.equal(calls, 1);
});

test("binding failures are redacted", async () => {
    const adapter = createAcornWebMcpAdapter(
        bindings({
            validateResearchActivity: () => {
                throw new Error("private input and /local/path");
            },
        }),
    );
    assert.deepEqual(await adapter.validateResearchActivity({ json: "{}" }), {
        error: { code: "execution_failed", message: "ACORN could not complete the operation." },
        ok: false,
    });
});

test("busy and cancellation policies avoid overlapping binding calls", async () => {
    const controller = new AbortController();
    const adapter = createAcornWebMcpAdapter(
        bindings({
            checkReadability: () => {
                controller.abort();
                return readabilityValue();
            },
        }),
    );
    const first = adapter.validateResearchActivity({ json: "{}" });
    assert.equal((await adapter.validateResearchActivity({ json: "{}" })).error.code, "busy");
    assert.equal((await first).ok, true);
    assert.equal((await adapter.checkReadability({ metric: "fkgl", text: "Text." }, { signal: controller.signal })).error.code, "cancelled");
});

test("installation cleans up partial registration and permits retry", async () => {
    const adapter = createAcornWebMcpAdapter(bindings());
    const failing = new FakeModelContext("acorn_validate_research_activity");
    await assert.rejects(adapter.install(failing).ready, /registration failed/);
    assert.equal(failing.tools.size, 0);
    assert.equal((await adapter.validateResearchActivity({ json: "{}" })).ok, true);
    const succeeding = new FakeModelContext();
    const installation = adapter.install(succeeding);
    assert.equal(await installation.ready, "installed");
    assert.deepEqual([...succeeding.tools.keys()], ["acorn_check_readability", "acorn_validate_research_activity"]);
    installation.dispose();
    assert.equal(succeeding.tools.size, 0);
});

test("installation handles unsupported, duplicate, and disposed states", async () => {
    const adapter = createAcornWebMcpAdapter(bindings());
    const unsupported = adapter.install(undefined);
    assert.equal(await unsupported.ready, "unsupported");
    assert.throws(() => adapter.install(undefined), /already installed/);
    unsupported.dispose();
    const context = new FakeModelContext();
    const installed = adapter.install(context);
    assert.equal(await installed.ready, "installed");
    installed.dispose();
    installed.dispose();
    assert.equal(context.tools.size, 0);
});

test("installation can be disposed while registration is pending", async () => {
    const adapter = createAcornWebMcpAdapter(bindings());
    const context = new PendingModelContext();
    const installation = adapter.install(context);
    installation.dispose();
    context.release();
    assert.equal(await installation.ready, "disposed");
    assert.equal(context.tools.size, 0);
});

test("installation registers strict schemas and safety annotations", async () => {
    const context = new FakeModelContext();
    const adapter = createAcornWebMcpAdapter(bindings());
    const installation = adapter.install(context);
    assert.equal(await installation.ready, "installed");
    const readability = context.tools.get("acorn_check_readability");
    const validation = context.tools.get("acorn_validate_research_activity");
    assert.deepEqual(readability.annotations, {
        consequentialHint: false,
        readOnlyHint: true,
        untrustedContentHint: true,
    });
    assert.equal(readability.inputSchema.additionalProperties, false);
    assert.deepEqual(readability.inputSchema.required, ["metric", "text"]);
    assert.deepEqual(readability.inputSchema.properties.metric.enum, ["ari", "cli", "fkgl", "fres", "gfi", "lix", "smog"]);
    assert.equal(validation.inputSchema.additionalProperties, false);
    assert.deepEqual(validation.inputSchema.required, ["json"]);
    installation.dispose();
});

test("invocation events report equivalent manual results", async () => {
    const events = [];
    const adapter = createAcornWebMcpAdapter(bindings(), { onInvocation: (event) => events.push(event) });
    const output = await adapter.checkReadability({ metric: "fkgl", text: "A complete sentence." });
    assert.equal(output.ok, true);
    assert.deepEqual(events.map(({ phase, tool }) => ({ phase, tool })), [
        { phase: "started", tool: "acorn_check_readability" },
        { phase: "finished", tool: "acorn_check_readability" },
    ]);
    assert.deepEqual(events.at(-1).output, output);
});

test("non-finite readability scores become bounded errors", async () => {
    const adapter = createAcornWebMcpAdapter(bindings({ checkReadability: () => readabilityValue({ score: null }) }));
    assert.deepEqual(await adapter.checkReadability({ metric: "fkgl", text: "No terminal punctuation" }), {
        error: { code: "execution_failed", message: "Readability is unavailable for the supplied text." },
        ok: false,
    });
});

test("readability preserves binding results for every supported metric", async () => {
    const metrics = ["ari", "cli", "fkgl", "fres", "gfi", "lix", "smog"];
    const calls = [];
    const adapter = createAcornWebMcpAdapter(
        bindings({
            checkReadability: (text, metric) => {
                calls.push([text, metric]);
                return readabilityValue({ metric });
            },
        }),
    );
    for (const metric of metrics) {
        const output = await adapter.checkReadability({ metric, text: "Unicode prose: café." });
        assert.equal(output.result.checks[0].metric, metric);
    }
    assert.deepEqual(calls, metrics.map((metric) => ["Unicode prose: café.", metric]));
});

test("normalization converts maps, omits undefined values, and rejects cycles", async () => {
    const params = new Map([["nested", new Map([["value", 1]])], ["omitted", undefined]]);
    const adapter = createAcornWebMcpAdapter(
        bindings({
            validateResearchActivity: () => ({
                diagnostics: [{ code: "invalid", kind: "validation", message: "Invalid", params, path: "title" }],
                valid: false,
            }),
        }),
    );
    const output = await adapter.validateResearchActivity({ json: "{}" });
    assert.deepEqual(output.result.diagnostics[0].params, { nested: { value: 1 } });
    const cyclic = {};
    cyclic.self = cyclic;
    const cyclicAdapter = createAcornWebMcpAdapter(
        bindings({
            validateResearchActivity: () => ({
                diagnostics: [{ code: "invalid", kind: "validation", message: "Invalid", params: cyclic, path: null }],
                valid: false,
            }),
        }),
    );
    assert.equal((await cyclicAdapter.validateResearchActivity({ json: "{}" })).error.code, "execution_failed");
});

test("output bounds reject oversized normalized results", async () => {
    const adapter = createAcornWebMcpAdapter(
        bindings({
            validateResearchActivity: () => ({
                diagnostics: [{ code: "invalid", kind: "validation", message: "x".repeat(256 * 1024), params: {}, path: null }],
                valid: false,
            }),
        }),
    );
    assert.equal((await adapter.validateResearchActivity({ json: "{}" })).error.code, "result_too_large");
});

test("tool execution returns direct structured envelopes", async () => {
    const context = new FakeModelContext();
    const adapter = createAcornWebMcpAdapter(bindings());
    const installation = adapter.install(context);
    await installation.ready;
    const tool = context.tools.get("acorn_validate_research_activity");
    assert.deepEqual(await tool.execute({ json: "{}" }, {}), {
        ok: true,
        result: { diagnosticCount: 0, diagnostics: [], truncated: false, valid: true },
    });
    installation.dispose();
});

test("validation preserves order, caps diagnostics, and never echoes the value", async () => {
    const diagnostics = Array.from({ length: 101 }, (_, index) => ({
        code: "invalid",
        kind: "validation",
        message: `Issue ${index}`,
        params: {},
        path: `items.${index}`,
    }));
    const adapter = createAcornWebMcpAdapter(
        bindings({ validateResearchActivity: () => ({ diagnostics, valid: false, value: { secret: true } }) }),
    );
    const output = await adapter.validateResearchActivity({ json: "{}" });
    assert.equal(output.result.diagnosticCount, 101);
    assert.equal(output.result.diagnostics.length, 100);
    assert.equal(output.result.diagnostics[0].path, "items.0");
    assert.equal(output.result.diagnostics.at(-1).path, "items.99");
    assert.equal(output.result.truncated, true);
    assert.equal("value" in output.result, false);
});

test("validation enforces the 256 KiB byte limit before invoking WASM", async () => {
    let calls = 0;
    const adapter = createAcornWebMcpAdapter(
        bindings({
            validateResearchActivity: () => {
                calls += 1;
                return { diagnostics: [], valid: true };
            },
        }),
    );
    assert.equal((await adapter.validateResearchActivity({ json: "x".repeat(256 * 1024) })).ok, true);
    assert.equal((await adapter.validateResearchActivity({ json: "x".repeat(256 * 1024 + 1) })).error.code, "input_too_large");
    assert.equal(calls, 1);
});
