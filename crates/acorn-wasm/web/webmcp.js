const DIAGNOSTIC_LIMIT = 100;
const JSON_INPUT_LIMIT = 256 * 1024;
const OUTPUT_LIMIT = 256 * 1024;
const READABILITY_INPUT_LIMIT = 16 * 1024;
const READABILITY_METRICS = Object.freeze(["ari", "cli", "fkgl", "fres", "gfi", "lix", "smog"]);
const READABILITY_TOOL_NAME = "acorn_check_readability";
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

function validateKeys(input, expected) {
    if (!isRecord(input)) {
        throw new AdapterError("invalid_arguments", "Tool arguments must be an object.");
    }
    const keys = Object.keys(input);
    if (keys.length !== expected.length || keys.some((key) => !expected.includes(key))) {
        throw new AdapterError("invalid_arguments", `Tool arguments must contain only: ${expected.join(", ")}.`);
    }
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
    if (typeof bindings?.checkReadability !== "function" || typeof bindings?.validateResearchActivity !== "function") {
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
    const checkReadability = (input, execution = {}) => run(READABILITY_TOOL_NAME, execution.signal, () => {
        const { metric, text } = readabilityArguments(input);
        return readabilityResult(bindings.checkReadability(text, metric));
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
            try {
                await modelContext.registerTool(
                    {
                        annotations,
                        description: "Assess supplied prose with an ACORN readability metric.",
                        execute: (input, execution) => checkReadability(input, execution),
                        inputSchema: readabilityInputSchema,
                        name: READABILITY_TOOL_NAME,
                        title: "Check readability",
                    },
                    { signal: controller.signal },
                );
                if (disposed) {
                    return "disposed";
                }
                await modelContext.registerTool(
                    {
                        annotations,
                        description: "Parse and validate supplied ACORN Research Activity JSON.",
                        execute: (input, execution) => validateResearchActivity(input, execution),
                        inputSchema: validationInputSchema,
                        name: VALIDATION_TOOL_NAME,
                        title: "Validate research activity",
                    },
                    { signal: controller.signal },
                );
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
    return Object.freeze({ checkReadability, install, validateResearchActivity });
}
