# API Refactor Plans

## Recommended order

- [API client roadmap](roadmap.md) - Coordinate the shared contract and extraction sequence.
- [Hugging Face request readiness](huggingface-request-readiness.md) - Enforce repository and download inputs through operation-specific request types.
- [GitLab claimed-operation capabilities](gitlab-claimed-operation-capabilities.md) - Consume queue claims through guarded single-settlement transitions.
- [Reusable webhook transport](webhook-transport-abstraction.md) - Extract provider-neutral verification, delivery, and safe outbound callback contracts.
- [GitLab bot analysis and citation intake](gitlab-webhook-bot-workflows.md) - Finish citation repository writes, completion handling, and bot operations.
- [Endpoint resolution](endpoint-resolution.md) - Centralize template and domain resolution.
- [Request parameter composition](request-parameter-composition.md) - Normalize required and optional inputs.
- [Invoke-and-parse wrappers](invoke-and-parse-wrappers.md) - Combine repeated request execution and parsing.
- [Environment bootstrap](environment-bootstrap.md) - Apply consistent configuration precedence.
- [Health-check helpers](health-check-helpers.md) - Share status interpretation.

## Later plans

- [API error envelopes](api-error-envelopes.md) - Normalize provider error messages.
- [OpenAPI endpoint validation](openapi-endpoint-validation.md) - Detect template drift.
- [RAiD API refinement](raid-api-refinement.md) - Convert the removed CLI prototype into validated service-point and record workflows.
- [Response DTOs](response-dtos.md) - Type stable response subfields.
- [OpenAPI module code generation](openapi-module-codegen.md) - Generate baseline modules and DTOs.
- [Provider capability matrix](provider-capability-matrix.md) - Describe supported provider behaviors.
