# API Refactor Plans

## Recommended order

- [GitLab repository automation](gitlab-webhook-bot-workflows.md) - Process typed repository flat files through bot branches and draft merge requests, including citation workflows.
- [Protobuf/gRPC transport](protobuf-grpc-transport.md) - Expose the shared operation registry through generated tonic/prost server and client adapters.
- [API client roadmap](roadmap.md) - Coordinate the shared contract and extraction sequence.
- [Hugging Face request readiness](huggingface-request-readiness.md) - Enforce repository and download inputs through operation-specific request types.
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
