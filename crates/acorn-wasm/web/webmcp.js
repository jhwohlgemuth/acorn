const CROSSWALK_PAIRS = Object.freeze([
    "raid:research-activity-data",
    "research-activity-data:datacite",
    "research-activity-data:raid",
]);
const CROSSWALK_TOOL_NAME = "acorn_crosswalk_metadata";
const DIAGNOSTIC_LIMIT = 100;
const DIFF_INPUT_LIMIT = 128 * 1024;
const DIFF_MODES = Object.freeze(["json-patch", "unified"]);
const DIFF_TOOL_NAME = "acorn_diff_content";
const DISCOVERY_TOOL_NAME = "acorn_discover_persistent_identifiers";
const FAIR_TOOL_NAME = "acorn_assess_fair_metadata";
const FORMATS = Object.freeze(["cff", "json", "jsonc", "markdown", "text", "yaml", "zon"]);
const IDENTIFIER_INPUT_LIMIT = 2 * 1024;
const IDENTIFIER_KINDS = Object.freeze(["ark", "arxiv", "doi", "handle", "isbn", "isni", "orcid", "patent", "raid", "ror", "rrid", "swhid"]);
const IDENTIFIER_TOOL_NAME = "acorn_inspect_persistent_identifier";
const JSON_INPUT_LIMIT = 256 * 1024;
const METADATA_STANDARDS = Object.freeze([
    "citation-file-format",
    "ckan",
    "datacite",
    "document",
    "invenio",
    "kitfile",
    "modelkit",
    "raid",
    "research-activity-data",
    "text",
]);
const OUTPUT_LIMIT = 256 * 1024;
const READABILITY_INPUT_LIMIT = 16 * 1024;
const READABILITY_METRICS = Object.freeze(["ari", "cli", "fkgl", "fres", "gfi", "lix", "smog"]);
const READABILITY_TOOL_NAME = "acorn_check_readability";
const VALIDATE_METADATA_TOOL_NAME = "acorn_validate_metadata_standard";
const VALIDATION_TOOL_NAME = "acorn_validate_research_activity";
const encoder = new TextEncoder();

class AdapterError extends Error {
    constructor(code, message) {
        super(message);
        this.code = code;
    }
}

const annotations = Object.freeze({
    consequentialHint: false,
    readOnlyHint: true,
    untrustedContentHint: true,
});

const fairInputSchema = Object.freeze({
    additionalProperties: false,
    properties: {
        json: { description: "Raw JSON metadata to assess.", maxLength: JSON_INPUT_LIMIT, type: "string" },
        resource: { description: "Optional stable resource label or URI.", maxLength: IDENTIFIER_INPUT_LIMIT, type: "string" },
        standard: { description: "Metadata standard represented by the JSON.", enum: METADATA_STANDARDS, type: "string" },
    },
    required: ["json", "standard"],
    type: "object",
});

const crosswalkInputSchema = Object.freeze({
    additionalProperties: false,
    properties: {
        contact: { description: "ContactPoint seed required only for RAiD to RAD.", type: "object" },
        content: { description: "Complete source metadata document.", maxLength: JSON_INPUT_LIMIT, type: "string" },
        format: { description: "Source document format.", enum: FORMATS, type: "string" },
        localIdentifier: { description: "Optional local RAD identifier for RAiD imports.", maxLength: IDENTIFIER_INPUT_LIMIT, type: "string" },
        source: { description: "Source metadata standard.", enum: ["raid", "research-activity-data"], type: "string" },
        target: { description: "Target metadata standard.", enum: ["datacite", "raid", "research-activity-data"], type: "string" },
    },
    required: ["content", "format", "source", "target"],
    type: "object",
});

const diffInputSchema = Object.freeze({
    additionalProperties: false,
    properties: {
        after: { description: "Content after the change.", maxLength: DIFF_INPUT_LIMIT, type: "string" },
        before: { description: "Content before the change.", maxLength: DIFF_INPUT_LIMIT, type: "string" },
        mode: { description: "Unified text diff or RFC 6902 JSON Patch.", enum: DIFF_MODES, type: "string" },
    },
    required: ["after", "before", "mode"],
    type: "object",
});

const discoveryInputSchema = Object.freeze({
    additionalProperties: false,
    properties: {
        text: { description: "Text in which to discover persistent identifiers.", maxLength: READABILITY_INPUT_LIMIT, minLength: 1, type: "string" },
    },
    required: ["text"],
    type: "object",
});

const identifierInputSchema = Object.freeze({
    additionalProperties: false,
    properties: {
        kind: {
            description: "Optional persistent identifier kind to validate strictly.",
            enum: IDENTIFIER_KINDS,
            type: "string",
        },
        value: {
            description: "Research identifier to classify, validate, and normalize without resolving it.",
            maxLength: IDENTIFIER_INPUT_LIMIT,
            minLength: 1,
            type: "string",
        },
    },
    required: ["value"],
    type: "object",
});

const readabilityInputSchema = Object.freeze({
    additionalProperties: false,
    properties: {
        metric: {
            description: "Readability metric to calculate.",
            enum: READABILITY_METRICS,
            type: "string",
        },
        text: {
            description: "UTF-8 prose to assess without reading external resources.",
            maxLength: READABILITY_INPUT_LIMIT,
            minLength: 1,
            type: "string",
        },
    },
    required: ["metric", "text"],
    type: "object",
});

const validationInputSchema = Object.freeze({
    additionalProperties: false,
    properties: {
        json: {
            description: "Raw Research Activity JSON to parse and validate.",
            maxLength: JSON_INPUT_LIMIT,
            type: "string",
        },
    },
    required: ["json"],
    type: "object",
});

const metadataValidationInputSchema = Object.freeze({
    additionalProperties: false,
    properties: {
        content: { description: "Complete metadata document to validate.", maxLength: JSON_INPUT_LIMIT, type: "string" },
        format: { description: "Document format.", enum: FORMATS, type: "string" },
        standard: { description: "Portable metadata standard to validate against.", enum: METADATA_STANDARDS, type: "string" },
    },
    required: ["content", "format", "standard"],
    type: "object",
});

function byteLength(value) {
    return encoder.encode(value).byteLength;
}

function cancelled(signal) {
    if (signal?.aborted) {
        throw new AdapterError("cancelled", "The ACORN operation was cancelled.");
    }
}

function emit(onInvocation, event) {
    try {
        onInvocation?.(event);
    } catch {
        // Consumer status reporting must not change tool execution.
    }
}

function failure(code, message) {
    return { error: { code, message }, ok: false };
}

function isRecord(value) {
    return value !== null && typeof value === "object" && !Array.isArray(value);
}

function normalize(value, seen = new WeakSet()) {
    if (value === null || typeof value === "boolean" || typeof value === "string") {
        return value;
    }
    if (typeof value === "number") {
        return Number.isFinite(value) ? value : null;
    }
    if (value === undefined) {
        return undefined;
    }
    if (typeof value !== "object") {
        throw new AdapterError("execution_failed", "ACORN returned a value that cannot be represented as JSON.");
    }
    if (seen.has(value)) {
        throw new AdapterError("execution_failed", "ACORN returned a cyclic value that cannot be represented as JSON.");
    }
    seen.add(value);
    const normalized = Array.isArray(value)
        ? value.map((item) => normalize(item, seen) ?? null)
        : Object.fromEntries(
              [...(value instanceof Map ? value.entries() : Object.entries(value))]
                  .map(([key, item]) => {
                      if (typeof key !== "string") {
                          throw new AdapterError("execution_failed", "ACORN returned a map with a non-string key.");
                      }
                      return [key, normalize(item, seen)];
                  })
                  .filter(([, item]) => item !== undefined),
          );
    seen.delete(value);
    return normalized;
}

function resultEnvelope(result) {
    const output = { ok: true, result };
    const serialized = JSON.stringify(output);
    return byteLength(serialized) <= OUTPUT_LIMIT
        ? output
        : failure("result_too_large", "The ACORN result exceeded the 256 KiB output limit.");
}

function validateKeys(input, allowed, required = allowed) {
    if (!isRecord(input)) {
        throw new AdapterError("invalid_arguments", "Tool arguments must be an object.");
    }
    const keys = Object.keys(input);
    if (keys.some((key) => !allowed.includes(key)) || required.some((key) => !keys.includes(key))) {
        throw new AdapterError("invalid_arguments", `Tool arguments must contain required fields and only: ${allowed.join(", ")}.`);
    }
}

function identifierResult(value) {
    const normalized = normalize(value);
    const fields = ["candidates", "kind", "normalized", "status"];
    if (!isRecord(normalized) || Object.keys(normalized).length !== fields.length || fields.some((field) => !(field in normalized))) {
        throw new AdapterError("execution_failed", "ACORN returned an invalid identifier result.");
    }
    if (!Array.isArray(normalized.candidates) || !normalized.candidates.every((kind) => IDENTIFIER_KINDS.includes(kind))) {
        throw new AdapterError("execution_failed", "ACORN returned an invalid identifier result.");
    }
    const valid = normalized.status === "valid"
        && IDENTIFIER_KINDS.includes(normalized.kind)
        && typeof normalized.normalized === "string"
        && normalized.normalized.length > 0
        && normalized.candidates.length === 0;
    const invalid = normalized.status === "invalid"
        && normalized.kind === null
        && normalized.normalized === null
        && normalized.candidates.length === 0;
    const ambiguous = normalized.status === "ambiguous"
        && normalized.kind === null
        && normalized.normalized === null
        && normalized.candidates.length === 2
        && normalized.candidates[0] === "isni"
        && normalized.candidates[1] === "orcid";
    if (!(valid || invalid || ambiguous)) {
        throw new AdapterError("execution_failed", "ACORN returned an invalid identifier result.");
    }
    return normalized;
}

function validationResult(value) {
    if (!isRecord(value) || typeof value.valid !== "boolean" || !Array.isArray(value.diagnostics)) {
        throw new AdapterError("execution_failed", "ACORN returned an invalid validation result.");
    }
    const diagnostics = value.diagnostics.slice(0, DIAGNOSTIC_LIMIT).map((diagnostic) => normalize(diagnostic));
    return {
        diagnosticCount: value.diagnostics.length,
        diagnostics,
        truncated: value.diagnostics.length > DIAGNOSTIC_LIMIT,
        valid: value.valid,
    };
}

function readabilityResult(value) {
    const normalized = normalize(value);
    if (!isRecord(normalized) || !Array.isArray(normalized.checks) || !Array.isArray(normalized.summary)) {
        throw new AdapterError("execution_failed", "ACORN returned an invalid readability result.");
    }
    if (normalized.checks.some((check) => !isRecord(check) || !Number.isFinite(check.score) || !Number.isFinite(check.threshold))) {
        throw new AdapterError("execution_failed", "Readability is unavailable for the supplied text.");
    }
    return normalized;
}

function assessFairArguments(input) {
    validateKeys(input, ["json", "resource", "standard"], ["json", "standard"]);
    if (typeof input.json !== "string" || !METADATA_STANDARDS.includes(input.standard)) {
        throw new AdapterError("invalid_arguments", "FAIR assessment requires JSON and a supported metadata standard.");
    }
    if (input.resource !== undefined && (typeof input.resource !== "string" || input.resource.trim().length === 0)) {
        throw new AdapterError("invalid_arguments", "FAIR resource must be a non-empty string when supplied.");
    }
    if (byteLength(input.json) > JSON_INPUT_LIMIT || (input.resource !== undefined && byteLength(input.resource) > IDENTIFIER_INPUT_LIMIT)) {
        throw new AdapterError("input_too_large", "FAIR assessment input exceeded its UTF-8 byte limit.");
    }
    return input;
}

function crosswalkArguments(input) {
    validateKeys(input, ["contact", "content", "format", "localIdentifier", "source", "target"], ["content", "format", "source", "target"]);
    if (typeof input.content !== "string" || !FORMATS.includes(input.format) || !CROSSWALK_PAIRS.includes(`${input.source}:${input.target}`)) {
        throw new AdapterError("invalid_arguments", "Crosswalk requires content, a supported format, and a supported source-target pair.");
    }
    if (input.contact !== undefined && !isRecord(input.contact)) {
        throw new AdapterError("invalid_arguments", "Crosswalk contact must be an object when supplied.");
    }
    if (input.source === "raid" && input.target === "research-activity-data" && !isRecord(input.contact)) {
        throw new AdapterError("invalid_arguments", "RAiD to RAD conversion requires a contact seed.");
    }
    if (input.localIdentifier !== undefined && (typeof input.localIdentifier !== "string" || input.localIdentifier.trim().length === 0)) {
        throw new AdapterError("invalid_arguments", "Crosswalk localIdentifier must be a non-empty string when supplied.");
    }
    let contactJson;
    try {
        contactJson = input.contact === undefined ? undefined : JSON.stringify(input.contact);
    } catch {
        throw new AdapterError("invalid_arguments", "Crosswalk contact must be representable as JSON.");
    }
    if (
        byteLength(input.content) > JSON_INPUT_LIMIT
        || (contactJson !== undefined && byteLength(contactJson) > JSON_INPUT_LIMIT)
        || (input.localIdentifier !== undefined && byteLength(input.localIdentifier) > IDENTIFIER_INPUT_LIMIT)
    ) {
        throw new AdapterError("input_too_large", "Crosswalk input exceeded its UTF-8 byte limit.");
    }
    return { ...input, contactJson };
}

function crosswalkResult(value) {
    const normalized = normalize(value);
    if (!isRecord(normalized) || typeof normalized.content !== "string" || normalized.format !== "json" || !Array.isArray(normalized.warnings)) {
        throw new AdapterError("execution_failed", "ACORN returned an invalid crosswalk result.");
    }
    return normalized;
}

function diffArguments(input) {
    validateKeys(input, ["after", "before", "mode"]);
    if (typeof input.after !== "string" || typeof input.before !== "string" || !DIFF_MODES.includes(input.mode)) {
        throw new AdapterError("invalid_arguments", "Diff requires before and after strings and a supported mode.");
    }
    if (byteLength(input.after) > DIFF_INPUT_LIMIT || byteLength(input.before) > DIFF_INPUT_LIMIT) {
        throw new AdapterError("input_too_large", "Diff content exceeded the 128 KiB per-value UTF-8 input limit.");
    }
    return input;
}

function diffResult(value) {
    const normalized = normalize(value);
    const validUnified = normalized?.mode === "unified" && typeof normalized.changed === "boolean" && typeof normalized.diff === "string";
    const validPatch = normalized?.mode === "json-patch" && typeof normalized.changed === "boolean" && Array.isArray(normalized.operations);
    if (!(validUnified || validPatch)) {
        throw new AdapterError("execution_failed", "ACORN returned an invalid diff result.");
    }
    return normalized;
}

function discoveryArguments(input) {
    validateKeys(input, ["text"]);
    if (typeof input.text !== "string" || input.text.trim().length === 0) {
        throw new AdapterError("invalid_arguments", "Identifier discovery requires non-empty text.");
    }
    if (byteLength(input.text) > READABILITY_INPUT_LIMIT) {
        throw new AdapterError("input_too_large", "Identifier discovery text exceeded the 16 KiB UTF-8 input limit.");
    }
    return input;
}

function discoveryResult(value) {
    const normalized = normalize(value);
    const valid = isRecord(normalized)
        && Number.isSafeInteger(normalized.count)
        && normalized.count >= 0
        && typeof normalized.truncated === "boolean"
        && Array.isArray(normalized.identifiers)
        && normalized.count >= normalized.identifiers.length
        && normalized.identifiers.length <= 100
        && normalized.truncated === (normalized.count > normalized.identifiers.length)
        && normalized.identifiers.every((identifier) => isRecord(identifier)
            && IDENTIFIER_KINDS.includes(identifier.kind)
            && typeof identifier.normalized === "string");
    if (!valid) {
        throw new AdapterError("execution_failed", "ACORN returned an invalid identifier discovery result.");
    }
    return normalized;
}

function fairResult(value) {
    const normalized = normalize(value);
    if (!isRecord(normalized) || !Array.isArray(normalized.findings) || normalized.findings.length !== 41 || !isRecord(normalized.profile)) {
        throw new AdapterError("execution_failed", "ACORN returned an invalid FAIR assessment result.");
    }
    return normalized;
}

function identifierArguments(input) {
    validateKeys(input, ["kind", "value"], ["value"]);
    if (typeof input.value !== "string" || input.value.trim().length === 0) {
        throw new AdapterError("invalid_arguments", "Identifier inspection requires a non-empty value.");
    }
    if (input.kind !== undefined && (typeof input.kind !== "string" || !IDENTIFIER_KINDS.includes(input.kind))) {
        throw new AdapterError("invalid_arguments", "Unsupported persistent identifier kind.");
    }
    if (byteLength(input.value) > IDENTIFIER_INPUT_LIMIT) {
        throw new AdapterError("input_too_large", "Persistent identifier exceeded the 2 KiB UTF-8 input limit.");
    }
    return input;
}

function metadataValidationArguments(input) {
    validateKeys(input, ["content", "format", "standard"]);
    if (typeof input.content !== "string" || !FORMATS.includes(input.format) || !METADATA_STANDARDS.includes(input.standard)) {
        throw new AdapterError("invalid_arguments", "Metadata validation requires content, a supported format, and a supported standard.");
    }
    if (byteLength(input.content) > JSON_INPUT_LIMIT) {
        throw new AdapterError("input_too_large", "Metadata content exceeded the 256 KiB UTF-8 input limit.");
    }
    return input;
}

function readabilityArguments(input) {
    validateKeys(input, ["metric", "text"]);
    if (typeof input.text !== "string" || input.text.trim().length === 0 || typeof input.metric !== "string") {
        throw new AdapterError("invalid_arguments", "Readability requires non-empty text and a supported metric.");
    }
    if (!READABILITY_METRICS.includes(input.metric)) {
        throw new AdapterError("invalid_arguments", "Unsupported readability metric.");
    }
    if (byteLength(input.text) > READABILITY_INPUT_LIMIT) {
        throw new AdapterError("input_too_large", "Readability text exceeded the 16 KiB UTF-8 input limit.");
    }
    return input;
}

function validationArguments(input) {
    validateKeys(input, ["json"]);
    if (typeof input.json !== "string") {
        throw new AdapterError("invalid_arguments", "Validation requires a JSON string.");
    }
    if (byteLength(input.json) > JSON_INPUT_LIMIT) {
        throw new AdapterError("input_too_large", "Research Activity JSON exceeded the 256 KiB UTF-8 input limit.");
    }
    return input;
}

function defaultModelContext() {
    return typeof document === "object" ? document.modelContext : undefined;
}

/**
 * Create an ACORN WebMCP adapter around initialized WASM bindings.
 *
 * Importing this module has no registration or network side effects.
 */
export function createAcornWebMcpAdapter(bindings, options = {}) {
    if (
        typeof bindings?.assessFairMetadata !== "function"
        || typeof bindings?.checkReadability !== "function"
        || typeof bindings?.crosswalkMetadata !== "function"
        || typeof bindings?.diffContent !== "function"
        || typeof bindings?.discoverPersistentIdentifiers !== "function"
        || typeof bindings?.inspectPersistentIdentifier !== "function"
        || typeof bindings?.validateMetadataStandard !== "function"
        || typeof bindings?.validateResearchActivity !== "function"
    ) {
        throw new TypeError("Initialized ACORN WASM bindings are required.");
    }
    if (options.onInvocation !== undefined && typeof options.onInvocation !== "function") {
        throw new TypeError("onInvocation must be a function when supplied.");
    }
    let active = false;
    let installation;
    const run = async (tool, signal, operation) => {
        if (active) {
            return failure("busy", "Another ACORN operation is already running.");
        }
        active = true;
        emit(options.onInvocation, { phase: "started", tool });
        let output;
        try {
            cancelled(signal);
            output = resultEnvelope(await operation());
            cancelled(signal);
        } catch (error) {
            output = error instanceof AdapterError
                ? failure(error.code, error.message)
                : failure("execution_failed", "ACORN could not complete the operation.");
        } finally {
            active = false;
        }
        emit(options.onInvocation, { output, phase: "finished", tool });
        return output;
    };
    const assessFairMetadata = (input, execution = {}) => run(FAIR_TOOL_NAME, execution.signal, () => {
        const { json, resource, standard } = assessFairArguments(input);
        return fairResult(bindings.assessFairMetadata(json, standard, resource));
    });
    const checkReadability = (input, execution = {}) => run(READABILITY_TOOL_NAME, execution.signal, () => {
        const { metric, text } = readabilityArguments(input);
        return readabilityResult(bindings.checkReadability(text, metric));
    });
    const crosswalkMetadata = (input, execution = {}) => run(CROSSWALK_TOOL_NAME, execution.signal, () => {
        const { contactJson, content, format, localIdentifier, source, target } = crosswalkArguments(input);
        return crosswalkResult(bindings.crosswalkMetadata(
            content,
            format,
            source,
            target,
            contactJson,
            localIdentifier,
        ));
    });
    const diffContent = (input, execution = {}) => run(DIFF_TOOL_NAME, execution.signal, () => {
        const { after, before, mode } = diffArguments(input);
        return diffResult(bindings.diffContent(before, after, mode));
    });
    const discoverPersistentIdentifiers = (input, execution = {}) => run(DISCOVERY_TOOL_NAME, execution.signal, () => {
        const { text } = discoveryArguments(input);
        return discoveryResult(bindings.discoverPersistentIdentifiers(text));
    });
    const inspectPersistentIdentifier = (input, execution = {}) => run(IDENTIFIER_TOOL_NAME, execution.signal, () => {
        const { kind, value } = identifierArguments(input);
        return identifierResult(bindings.inspectPersistentIdentifier(value, kind));
    });
    const validateMetadataStandard = (input, execution = {}) => run(VALIDATE_METADATA_TOOL_NAME, execution.signal, () => {
        const { content, format, standard } = metadataValidationArguments(input);
        return validationResult(bindings.validateMetadataStandard(content, standard, format));
    });
    const validateResearchActivity = (input, execution = {}) => run(VALIDATION_TOOL_NAME, execution.signal, () => {
        const { json } = validationArguments(input);
        return validationResult(bindings.validateResearchActivity(json));
    });
    const install = (modelContext = defaultModelContext()) => {
        if (installation !== undefined) {
            throw new Error("This ACORN WebMCP adapter is already installed.");
        }
        const controller = new AbortController();
        let disposed = false;
        let handle;
        const dispose = () => {
            if (!disposed) {
                disposed = true;
                controller.abort();
                if (installation === handle) {
                    installation = undefined;
                }
            }
        };
        const ready = (async () => {
            if (typeof modelContext?.registerTool !== "function") {
                return disposed ? "disposed" : "unsupported";
            }
            const tools = [
                {
                    annotations,
                    description: "Assess supplied JSON metadata against ACORN's 41 portable RDA FAIR indicators.",
                    execute: (input, execution) => assessFairMetadata(input, execution),
                    inputSchema: fairInputSchema,
                    name: FAIR_TOOL_NAME,
                    title: "Assess FAIR metadata",
                },
                {
                    annotations,
                    description: "Assess supplied prose with an ACORN readability metric.",
                    execute: (input, execution) => checkReadability(input, execution),
                    inputSchema: readabilityInputSchema,
                    name: READABILITY_TOOL_NAME,
                    title: "Check readability",
                },
                {
                    annotations,
                    description: "Convert supplied metadata through an existing typed, loss-aware ACORN crosswalk.",
                    execute: (input, execution) => crosswalkMetadata(input, execution),
                    inputSchema: crosswalkInputSchema,
                    name: CROSSWALK_TOOL_NAME,
                    title: "Crosswalk metadata",
                },
                {
                    annotations,
                    description: "Compare supplied content as a unified text diff or RFC 6902 JSON Patch.",
                    execute: (input, execution) => diffContent(input, execution),
                    inputSchema: diffInputSchema,
                    name: DIFF_TOOL_NAME,
                    title: "Diff content",
                },
                {
                    annotations,
                    description: "Discover and normalize persistent identifiers in supplied text without resolving them.",
                    execute: (input, execution) => discoverPersistentIdentifiers(input, execution),
                    inputSchema: discoveryInputSchema,
                    name: DISCOVERY_TOOL_NAME,
                    title: "Discover persistent identifiers",
                },
                {
                    annotations,
                    description: "Classify, validate, and normalize one supplied research identifier without resolving it.",
                    execute: (input, execution) => inspectPersistentIdentifier(input, execution),
                    inputSchema: identifierInputSchema,
                    name: IDENTIFIER_TOOL_NAME,
                    title: "Inspect persistent identifier",
                },
                {
                    annotations,
                    description: "Parse and validate supplied content against a portable metadata standard.",
                    execute: (input, execution) => validateMetadataStandard(input, execution),
                    inputSchema: metadataValidationInputSchema,
                    name: VALIDATE_METADATA_TOOL_NAME,
                    title: "Validate metadata standard",
                },
                {
                    annotations,
                    description: "Parse and validate supplied ACORN Research Activity JSON.",
                    execute: (input, execution) => validateResearchActivity(input, execution),
                    inputSchema: validationInputSchema,
                    name: VALIDATION_TOOL_NAME,
                    title: "Validate research activity",
                },
            ];
            try {
                for (const tool of tools) {
                    await modelContext.registerTool(
                        tool,
                        { signal: controller.signal },
                    );
                    if (disposed) {
                        return "disposed";
                    }
                }
                return disposed ? "disposed" : "installed";
            } catch (error) {
                controller.abort();
                if (disposed) {
                    return "disposed";
                }
                installation = undefined;
                throw new Error("ACORN WebMCP tool registration failed.", { cause: error });
            }
        })();
        handle = Object.freeze({ dispose, ready });
        installation = handle;
        return handle;
    };
    return Object.freeze({
        assessFairMetadata,
        checkReadability,
        crosswalkMetadata,
        diffContent,
        discoverPersistentIdentifiers,
        inspectPersistentIdentifier,
        install,
        validateMetadataStandard,
        validateResearchActivity,
    });
}
