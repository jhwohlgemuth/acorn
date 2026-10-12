import assert from "node:assert/strict";
import test from "node:test";

import { createAcornWebMcpAdapter } from "./webmcp.js";

const identifierValue = (overrides = {}) => ({
    candidates: [],
    kind: "doi",
    normalized: "10.1000/182",
    status: "valid",
    ...overrides,
});

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

const fairValue = () => ({
    findings: Array.from({ length: 41 }, (_, index) => ({ indicator: { id: `RDA-${index}` }, state: "indeterminate" })),
    profile: { resource: "memory://metadata", total: { indeterminate: 41, notApplicable: 0, supported: 0, unsupported: 0 } },
});

const bindings = (overrides = {}) => ({
    assessFairMetadata: () => fairValue(),
    checkReadability: () => readabilityValue(),
    crosswalkMetadata: (_content, _format, source, target) => ({ content: "{}", format: "json", source, target, warnings: [] }),
    diffContent: () => ({ changed: true, diff: "--- before\n+++ after", mode: "unified" }),
    discoverPersistentIdentifiers: () => ({ count: 1, identifiers: [{ kind: "doi", normalized: "10.1000/182" }], truncated: false }),
    inspectPersistentIdentifier: () => identifierValue(),
    validateMetadataStandard: () => ({ diagnostics: [], valid: true }),
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
    assert.equal((await adapter.validateResearchActivity({ fuzzy: "strict", json: "{}" })).ok, true);
    assert.equal((await adapter.validateResearchActivity({ fuzzy: "unknown", json: "{}" })).error.code, "invalid_arguments");
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

test("new portable tools validate arguments and forward normalized values", async () => {
    const calls = [];
    const adapter = createAcornWebMcpAdapter(bindings({
        assessFairMetadata: (...args) => {
            calls.push(["fair", ...args]);
            return fairValue();
        },
        crosswalkMetadata: (...args) => {
            calls.push(["crosswalk", ...args]);
            return { content: "{}", format: "json", source: args[2], target: args[3], warnings: [] };
        },
        diffContent: (...args) => {
            calls.push(["diff", ...args]);
            return { changed: true, mode: "json-patch", operations: [{ op: "replace", path: "/title", value: "After" }] };
        },
        discoverPersistentIdentifiers: (...args) => {
            calls.push(["discover", ...args]);
            return { count: 1, identifiers: [{ kind: "doi", normalized: "10.1000/182" }], truncated: false };
        },
        validateMetadataStandard: (...args) => {
            calls.push(["validate", ...args]);
            return { diagnostics: [], valid: true };
        },
    }));
    assert.equal((await adapter.assessFairMetadata({ json: "{}", standard: "datacite" })).result.findings.length, 41);
    assert.equal((await adapter.crosswalkMetadata({ content: "{}", format: "json", source: "research-activity-data", target: "datacite" })).ok, true);
    assert.equal((await adapter.diffContent({ after: "{\"title\":\"After\"}", before: "{\"title\":\"Before\"}", mode: "json-patch" })).result.mode, "json-patch");
    assert.equal((await adapter.discoverPersistentIdentifiers({ text: "doi:10.1000/182" })).result.identifiers[0].kind, "doi");
    assert.equal((await adapter.validateMetadataStandard({ content: "{}", format: "json", standard: "datacite" })).result.valid, true);
    assert.deepEqual(calls, [
        ["fair", "{}", "datacite", undefined],
        ["crosswalk", "{}", "json", "research-activity-data", "datacite", undefined, undefined],
        ["diff", "{\"title\":\"Before\"}", "{\"title\":\"After\"}", "json-patch"],
        ["discover", "doi:10.1000/182"],
        ["validate", "{}", "datacite", "json"],
    ]);
});

test("crosswalk enforces supported pairs and RAD import context", async () => {
    const adapter = createAcornWebMcpAdapter(bindings());
    assert.equal((await adapter.crosswalkMetadata({ content: "{}", format: "json", source: "datacite", target: "raid" })).error.code, "invalid_arguments");
    assert.equal((await adapter.crosswalkMetadata({ content: "{}", format: "json", source: "raid", target: "research-activity-data" })).error.code, "invalid_arguments");
    assert.equal((await adapter.crosswalkMetadata({ content: "{}", format: "json-ld", source: "dublin-core", target: "research-activity-data" })).error.code, "invalid_arguments");
    assert.equal((await adapter.crosswalkMetadata({
        contact: { email: "person@example.org", name: "Example Person" },
        content: "{}",
        format: "json-ld",
        source: "dublin-core",
        target: "research-activity-data",
    })).error.code, "invalid_arguments");
    assert.equal((await adapter.crosswalkMetadata({
        contact: { email: "person@example.org", name: "Example Person" },
        content: "{}",
        format: "turtle",
        localIdentifier: "browser-import",
        source: "dublin-core",
        target: "research-activity-data",
    })).ok, true);
    assert.equal((await adapter.crosswalkMetadata({
        contact: { email: "person@example.org", name: "Example Person" },
        content: "{}",
        format: "json",
        source: "raid",
        target: "research-activity-data",
    })).ok, true);
    const contact = {};
    contact.self = contact;
    assert.equal((await adapter.crosswalkMetadata({
        contact,
        content: "{}",
        format: "json",
        source: "raid",
        target: "research-activity-data",
    })).error.code, "invalid_arguments");
});

test("diff and discovery enforce independent UTF-8 limits", async () => {
    const adapter = createAcornWebMcpAdapter(bindings());
    assert.equal((await adapter.diffContent({ after: "", before: "é".repeat(64 * 1024), mode: "unified" })).ok, true);
    assert.equal((await adapter.diffContent({ after: "", before: `${"é".repeat(64 * 1024)}x`, mode: "unified" })).error.code, "input_too_large");
    assert.equal((await adapter.discoverPersistentIdentifiers({ text: "é".repeat(8 * 1024) })).ok, true);
    assert.equal((await adapter.discoverPersistentIdentifiers({ text: `${"é".repeat(8 * 1024)}x` })).error.code, "input_too_large");
});

test("bindings are required for every installed tool", () => {
    assert.throws(
        () => createAcornWebMcpAdapter({ checkReadability: () => readabilityValue(), validateResearchActivity: () => ({}) }),
        /bindings are required/,
    );
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

test("identifier inspection enforces optional kind and UTF-8 byte limits", async () => {
    const calls = [];
    const adapter = createAcornWebMcpAdapter(
        bindings({
            inspectPersistentIdentifier: (value, kind) => {
                calls.push([value, kind]);
                return identifierValue({ kind: kind ?? "doi" });
            },
        }),
    );
    assert.equal((await adapter.inspectPersistentIdentifier({ value: "10.1000/182" })).ok, true);
    assert.equal((await adapter.inspectPersistentIdentifier({ kind: "doi", value: "10.1000/182" })).ok, true);
    assert.equal((await adapter.inspectPersistentIdentifier({ kind: "unknown", value: "10.1000/182" })).error.code, "invalid_arguments");
    assert.equal((await adapter.inspectPersistentIdentifier({ extra: true, value: "10.1000/182" })).error.code, "invalid_arguments");
    assert.equal((await adapter.inspectPersistentIdentifier({ value: " " })).error.code, "invalid_arguments");
    assert.equal((await adapter.inspectPersistentIdentifier({ value: "é".repeat(1024) })).ok, true);
    assert.equal((await adapter.inspectPersistentIdentifier({ value: `${"é".repeat(1024)}x` })).error.code, "input_too_large");
    assert.deepEqual(calls, [
        ["10.1000/182", undefined],
        ["10.1000/182", "doi"],
        ["é".repeat(1024), undefined],
    ]);
});

test("identifier inspection preserves valid invalid and ambiguous results", async () => {
    const values = [
        identifierValue(),
        identifierValue({ candidates: [], kind: null, normalized: null, status: "invalid" }),
        identifierValue({ candidates: ["isni", "orcid"], kind: null, normalized: null, status: "ambiguous" }),
    ];
    const adapter = createAcornWebMcpAdapter(bindings({ inspectPersistentIdentifier: () => values.shift() }));
    assert.equal((await adapter.inspectPersistentIdentifier({ value: "10.1000/182" })).result.status, "valid");
    assert.equal((await adapter.inspectPersistentIdentifier({ value: "invalid" })).result.status, "invalid");
    assert.deepEqual((await adapter.inspectPersistentIdentifier({ value: "0000-0002-2057-9115" })).result.candidates, ["isni", "orcid"]);
});

test("identifier inspection rejects malformed binding results", async () => {
    const adapter = createAcornWebMcpAdapter(bindings({ inspectPersistentIdentifier: () => ({ status: "valid" }) }));
    assert.equal((await adapter.inspectPersistentIdentifier({ value: "10.1000/182" })).error.code, "execution_failed");
});

test("installation cleans up partial registration and permits retry", async () => {
    const adapter = createAcornWebMcpAdapter(bindings());
    const failing = new FakeModelContext("acorn_inspect_persistent_identifier");
    await assert.rejects(adapter.install(failing).ready, /registration failed/);
    assert.equal(failing.tools.size, 0);
    assert.equal((await adapter.validateResearchActivity({ json: "{}" })).ok, true);
    const succeeding = new FakeModelContext();
    const installation = adapter.install(succeeding);
    assert.equal(await installation.ready, "installed");
    assert.deepEqual([...succeeding.tools.keys()], [
        "acorn_assess_fair_metadata",
        "acorn_check_readability",
        "acorn_crosswalk_metadata",
        "acorn_diff_content",
        "acorn_discover_persistent_identifiers",
        "acorn_inspect_persistent_identifier",
        "acorn_validate_metadata_standard",
        "acorn_validate_research_activity",
    ]);
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
    const identifier = context.tools.get("acorn_inspect_persistent_identifier");
    const metadata = context.tools.get("acorn_validate_metadata_standard");
    const validation = context.tools.get("acorn_validate_research_activity");
    assert.deepEqual(readability.annotations, {
        consequentialHint: false,
        readOnlyHint: true,
        untrustedContentHint: true,
    });
    assert.equal(readability.inputSchema.additionalProperties, false);
    assert.deepEqual(readability.inputSchema.required, ["metric", "text"]);
    assert.deepEqual(readability.inputSchema.properties.metric.enum, ["ari", "cli", "fkgl", "fres", "gfi", "lix", "smog"]);
    assert.equal(identifier.inputSchema.additionalProperties, false);
    assert.deepEqual(identifier.inputSchema.required, ["value"]);
    assert.deepEqual(identifier.inputSchema.properties.kind.enum, [
        "ark",
        "arxiv",
        "doi",
        "handle",
        "isbn",
        "isni",
        "orcid",
        "patent",
        "raid",
        "ror",
        "rrid",
        "swhid",
    ]);
    assert.equal(validation.inputSchema.additionalProperties, false);
    assert.deepEqual(validation.inputSchema.required, ["json"]);
    assert.deepEqual(validation.inputSchema.properties.fuzzy.enum, ["off", "relaxed", "strict"]);
    assert.deepEqual(metadata.inputSchema.required, ["content", "format", "standard"]);
    assert.deepEqual(context.tools.get("acorn_diff_content").inputSchema.properties.mode.enum, ["json-patch", "unified"]);
    assert.deepEqual(context.tools.get("acorn_crosswalk_metadata").inputSchema.required, ["content", "format", "source", "target"]);
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
