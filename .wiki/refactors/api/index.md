# API Refactor Plans

## Recommended order

- [API client roadmap](roadmap.md) - Coordinate the shared contract and extraction sequence.
- [Endpoint resolution](endpoint-resolution.md) - Centralize template and domain resolution.
- [Authentication headers](authentication-headers.md) - Share bearer and custom-token parameter construction.
- [Request parameter composition](request-parameter-composition.md) - Normalize required and optional inputs.
- [Invoke-and-parse wrappers](invoke-and-parse-wrappers.md) - Combine repeated request execution and parsing.
- [Environment bootstrap](environment-bootstrap.md) - Apply consistent configuration precedence.
- [Health-check helpers](health-check-helpers.md) - Share status interpretation.

## Later plans

- [API error envelopes](api-error-envelopes.md) - Normalize provider error messages.
- [OpenAPI endpoint validation](openapi-endpoint-validation.md) - Detect template drift.
- [Response DTOs](response-dtos.md) - Type stable response subfields.
- [OpenAPI module code generation](openapi-module-codegen.md) - Generate baseline modules and DTOs.
- [Provider capability matrix](provider-capability-matrix.md) - Describe supported provider behaviors.
