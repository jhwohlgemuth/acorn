export type CrosswalkSource = "raid" | "research-activity-data";
export type CrosswalkTarget = "datacite" | "raid" | "research-activity-data";
export type DiffMode = "json-patch" | "unified";
export type ErrorCode =
    | "busy"
    | "cancelled"
    | "execution_failed"
    | "input_too_large"
    | "invalid_arguments"
    | "result_too_large";
export type IdentifierInspectionStatus = "ambiguous" | "invalid" | "valid";
export type IdentifierKind = "ark" | "arxiv" | "doi" | "handle" | "isbn" | "isni" | "orcid" | "patent" | "raid" | "ror" | "rrid" | "swhid";
export type InstallationStatus = "disposed" | "installed" | "unsupported";
export type MetadataFormat = "cff" | "json" | "jsonc" | "markdown" | "text" | "yaml" | "zon";
export type MetadataStandard =
    | "citation-file-format"
    | "ckan"
    | "datacite"
    | "document"
    | "invenio"
    | "kitfile"
    | "modelkit"
    | "raid"
    | "research-activity-data"
    | "text";
export type ReadabilityMetric = "ari" | "cli" | "fkgl" | "fres" | "gfi" | "lix" | "smog";
export type ToolName =
    | "acorn_assess_fair_metadata"
    | "acorn_check_readability"
    | "acorn_crosswalk_metadata"
    | "acorn_diff_content"
    | "acorn_discover_persistent_identifiers"
    | "acorn_inspect_persistent_identifier"
    | "acorn_validate_metadata_standard"
    | "acorn_validate_research_activity";
export type ToolResult<T> = { ok: true; result: T } | { error: { code: ErrorCode; message: string }; ok: false };

export interface ContentDiffJsonPatchResult {
    changed: boolean;
    mode: "json-patch";
    operations: unknown[];
}

export interface ContentDiffUnifiedResult {
    changed: boolean;
    diff: string;
    mode: "unified";
}

export type ContentDiffResult = ContentDiffJsonPatchResult | ContentDiffUnifiedResult;

export interface CrosswalkInput {
    contact?: Record<string, unknown>;
    content: string;
    format: MetadataFormat;
    localIdentifier?: string;
    source: CrosswalkSource;
    target: CrosswalkTarget;
}

export interface CrosswalkResult {
    content: string;
    format: "json";
    source: CrosswalkSource;
    target: CrosswalkTarget;
    warnings: CrosswalkWarning[];
}

export interface CrosswalkWarning {
    details: string | null;
    field: string;
    from: string;
    lossType: string;
    to: string;
}

export interface Diagnostic {
    code: string | null;
    kind: "parse" | "validation";
    message: string;
    params: Record<string, unknown>;
    path: string | null;
}

export interface DiscoveredIdentifier {
    kind: IdentifierKind;
    normalized: string;
}

export interface FairAssessmentResult {
    findings: Array<Record<string, unknown>>;
    profile: Record<string, unknown>;
}

export interface IdentifierDiscoveryResult {
    count: number;
    identifiers: DiscoveredIdentifier[];
    truncated: boolean;
}

export interface IdentifierInspectionResult {
    candidates: IdentifierKind[];
    kind: IdentifierKind | null;
    normalized: string | null;
    status: IdentifierInspectionStatus;
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
    output?: ToolResult<
        ContentDiffResult
        | CrosswalkResult
        | FairAssessmentResult
        | IdentifierDiscoveryResult
        | IdentifierInspectionResult
        | ReadabilityResult
        | ValidationResult
    >;
    phase: "finished" | "started";
    tool: ToolName;
}

export interface AcornBindings {
    assessFairMetadata(json: string, standard: string, resource?: string): unknown;
    checkReadability(text: string, metric: string): unknown;
    crosswalkMetadata(
        content: string,
        format: string,
        source: string,
        target: string,
        contactJson?: string,
        localIdentifier?: string,
    ): unknown;
    diffContent(before: string, after: string, mode: string): unknown;
    discoverPersistentIdentifiers(text: string): unknown;
    inspectPersistentIdentifier(value: string, kind?: string): unknown;
    validateMetadataStandard(content: string, standard: string, format: string): unknown;
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
    assessFairMetadata(
        input: { json: string; resource?: string; standard: MetadataStandard },
        options?: ExecutionOptions,
    ): Promise<ToolResult<FairAssessmentResult>>;
    checkReadability(
        input: { metric: ReadabilityMetric; text: string },
        options?: ExecutionOptions,
    ): Promise<ToolResult<ReadabilityResult>>;
    crosswalkMetadata(input: CrosswalkInput, options?: ExecutionOptions): Promise<ToolResult<CrosswalkResult>>;
    diffContent(
        input: { after: string; before: string; mode: DiffMode },
        options?: ExecutionOptions,
    ): Promise<ToolResult<ContentDiffResult>>;
    discoverPersistentIdentifiers(
        input: { text: string },
        options?: ExecutionOptions,
    ): Promise<ToolResult<IdentifierDiscoveryResult>>;
    inspectPersistentIdentifier(
        input: { kind?: IdentifierKind; value: string },
        options?: ExecutionOptions,
    ): Promise<ToolResult<IdentifierInspectionResult>>;
    install(modelContext?: ModelContext): WebMcpInstallation;
    validateMetadataStandard(
        input: { content: string; format: MetadataFormat; standard: MetadataStandard },
        options?: ExecutionOptions,
    ): Promise<ToolResult<ValidationResult>>;
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
