export type ErrorCode =
    | "busy"
    | "cancelled"
    | "execution_failed"
    | "input_too_large"
    | "invalid_arguments"
    | "result_too_large";

export type ToolResult<T> = { ok: true; result: T } | { error: { code: ErrorCode; message: string }; ok: false };
export type ReadabilityMetric = "ari" | "cli" | "fkgl" | "fres" | "gfi" | "lix" | "smog";
export type InstallationStatus = "disposed" | "installed" | "unsupported";

export interface Diagnostic {
    code: string | null;
    kind: "parse" | "validation";
    message: string;
    params: Record<string, unknown>;
    path: string | null;
}

export interface ReadabilityCheck {
    category: string;
    message: string;
    metric: string;
    score: number;
    severity: string;
    success: boolean;
    threshold: number;
    uri?: string | null;
}

export interface ReadabilityResult {
    checks: ReadabilityCheck[];
    summary: string[][];
}

export interface ValidationResult {
    diagnosticCount: number;
    diagnostics: Diagnostic[];
    truncated: boolean;
    valid: boolean;
}

export interface ExecutionOptions {
    signal?: AbortSignal;
}

export interface InvocationEvent {
    output?: ToolResult<ReadabilityResult | ValidationResult>;
    phase: "finished" | "started";
    tool: "acorn_check_readability" | "acorn_validate_research_activity";
}

export interface AcornBindings {
    checkReadability(text: string, metric: string): unknown;
    validateResearchActivity(json: string): unknown;
}

export interface ModelContextTool {
    annotations: {
        consequentialHint: boolean;
        readOnlyHint: boolean;
        untrustedContentHint: boolean;
    };
    description: string;
    execute(input: object, options: ExecutionOptions): Promise<unknown>;
    inputSchema: object;
    name: string;
    title: string;
}

export interface ModelContext {
    registerTool(tool: ModelContextTool, options?: { signal?: AbortSignal }): Promise<void>;
}

export interface WebMcpInstallation {
    dispose(): void;
    ready: Promise<InstallationStatus>;
}

export interface AcornWebMcpAdapter {
    checkReadability(
        input: { metric: ReadabilityMetric; text: string },
        options?: ExecutionOptions,
    ): Promise<ToolResult<ReadabilityResult>>;
    install(modelContext?: ModelContext): WebMcpInstallation;
    validateResearchActivity(
        input: { json: string },
        options?: ExecutionOptions,
    ): Promise<ToolResult<ValidationResult>>;
}

/** Create a side-effect-free adapter around initialized ACORN WASM bindings. */
export function createAcornWebMcpAdapter(
    bindings: AcornBindings,
    options?: { onInvocation?: (event: InvocationEvent) => void },
): AcornWebMcpAdapter;
