# Recovered ACORN Hunk comments (1–72)

Recovered from `/private/tmp/acorn-hunk-39945.core` after the Hunk daemon session disconnected. The original Hunk comment number and reviewed line are retained. Comments 1–53 were addressed during the earlier review pass; comments 54–72 were addressed during the follow-up pass.

1. Line 31 — Use Mimetype instead
2. Line 60 — Import `metadata` at top of module
3. Line 997 — Convert to method on ModelDetails impl
4. Line 1076 — Can this be implemented directly on GgufInfo?
5. Line 867 — Create trait for `context_length` and implement it on every element in this diff that implements `context_length` method
6. Line 355 — Combine this with the `impl OciLayer` in io::oci
7. Line 268 — Conflate OciFile and OciFiles impls with impls in io::oci
8. Line 476 — Convert to method on GgufInfo impl
9. Line 13 — Anonymize data in this fixture, do not mention real sponsors, companies, people, or locations
10. Line 254 — Replace project names with random names/words from anime (e.g., gundam, naruto, one-piece, etc.) - the goal is anonymity
11. Line 172 — Add kebabcase rule
12. Line 105 — Move tests to analyzer::host::tests
13. Line 1 — I do not like io_narrow...Can we move it out of acorn-lib and/or rename it?
14. Line 180 — Find a better, one-word name for this feature
15. Line 54 — Add a pikchr diagram that depicts the interconnected crosswalk paths
16. Line 36 — Identify Dublin Core as "in progress"
17. Line 54 — Add background shapes (rounded rectangles) to crosswalk items (similar to other diagrams)
18. Line 62 — Can the `*_with_cancellation` methods be conflated with the base methods by making cancellation argument optional?
19. Line 72 — Can `call_tool_with_cancellation` be conflated with `call_tool` by converting `cancellation` argument to optional? (similar question for `list_tools_with_cancellation`)
20. Line 16 — Remove import alias
21. Line 447 — Import ReadabilityType at top of module
22. Line 1471 — Import OutputFormat at top of module
23. Line 52 — Why do we need to create `render_check`
24. Line 60 — Import ApplicationConfiguration at top of module
25. Line 9 — Remove GatherOutputFormat alias, use context::OutputFormat, if specificity is required
26. Line 282 — Import export_schema at top of module
27. Line 105 — Remove un-used variables
28. Line 115 — Import DurableOperations at top of module
29. Line 116 — Import serve_with_durable at top of module
30. Line 114 — Remove all ? operators and early returns
31. Line 54 — Why is this needed? Remove, if possilbe
32. Line 62 — Import DurableOperations and serve_with_durable at top of module
33. Line 169 — Import `check` at top of module
34. Line 35 — Can we obviate and remove this function by using its content directly and still maintain portability boundaries?
35. Line 561 — Can we replace ariadne_color with impl of From<CheckSeverity> trait for Color and still maintain portability boundaries?
36. Line 97 — Can we obviate and remove this function by using its content directly and still maintain portability boundaries?
37. Line 1 — create folder-format constants module with mod.rs and fair.rs sub modules. Move this module to constants::fair
38. Line 1 — Rename this module to `vale`
39. Line 1 — move this module to constants::readability
40. Line 37 — Do not add uneccessary blank lines. remove blank lines between enums, structs, types, constants, impls, and functions
41. Line 110 — Refactor away ALL use of ? oeprators and early returns
42. Line 238 — Move tests to tests/mod.rs module
43. Line 188 — Move tests to tests/mod.rs module
44. Line 395 — Move tests to tests/mod.rs module
45. Line 41 — Move tests to tests/mod.rs module
46. Line 85 — Refactor away all early returns and ? operators
47. Line 130 — Move tests to tests/mod.rs module
48. Line 7 — Remove all uses of ? operator
49. Line 54 — Refactor out all use of ? operator and early returns
50. Line 23 — Move tests to tests/mod.rs module
51. Line 164 — Refactor away all use of ? operator and early returns
52. Line 29 — Remove ALL early returns and ? operators
53. Line 54 — Remove ALL early returns and ? operators
54. `crates/acorn-cli/src/commands/serve/grpc/mod.rs:105` — Remove un-needed variables.
55. `crates/acorn-lib/src/analyzer/host/context/mod.rs:19` — Can this be replaced/augmented with MimeType? Treat `Console` as `Text`?
56. `crates/acorn-lib/src/analyzer/host/render.rs:478` — Using `&**` feels a little wrong...is there a simple way to refactor this code to avoid `&**` but also avoid `.clone` and messy variable use?
57. `crates/acorn-server/src/config/mod.rs:157` — Move to where ServerResult is defined as a method on the ServerResult impl
58. `crates/acorn-server/src/config/mod.rs:154` — Obviate and remove this function by using its content directly
59. `crates/acorn-server/src/grpc/mod.rs:54` — Refactor away ALL use of ? operator and early returns
60. `crates/acorn-server/src/grpc/mod.rs:4` — Remove ALL import aliases. Use `proto::` to scope imports if specificity is required
61. `crates/acorn-server/src/runtime.rs:24` — Import `ring` at top of module
62. `crates/acorn-server/src/runtime.rs:25` — Import `ring` at top of module
63. `crates/acorn-server/src/runtime.rs:71` — Import TcpListener at top of module
64. `crates/acorn-service/src/registry.rs:167` — Can you either make this item more general (e.g., "acorn.standard.validate") or add new branch for any standard?
65. `crates/acorn-service/src/repository.rs:32` — Refactor match condition so it can comfortably fit on one line
66. `crates/acorn-service/src/repository.rs:64` — Obviate and remove function by using its content directly
67. `crates/acorn-service/src/repository.rs:51` — Either use existing Location capability for this or move this to a method on Location impl
68. `crates/acorn-service/src/repository.rs:97` — Convert to method on RepositoryFileInput impl
69. `crates/acorn-service/src/repository.rs:107` — I would like to abstract as much of the `normalize_*` functionality into the MimeType impl as possible
70. `crates/acorn-service/src/repository.rs:168` — Convert to method on ResearchActivity impl (use shared result struct)
71. `crates/acorn-service/src/repository.rs:177` — Obviate and remove function by using its content directly
72. `crates/acorn-service/src/service.rs:74` — Convert to static method on new Decode trait implemented for ResearchActivity (and other supported standards) - should be generic enough to support RADS, ACRE, datacite, etc.

## Follow-up resolution summary (54–72)

- 54: removed the redundant local `offline` binding.
- 55: renamed the host presentation variant to `Text` and added its `MimeType` conversion while preserving distinct raw-line output.
- 56: destructured directly through `RenderedCheck.0`, with no clone.
- 57–58: removed the one-use `enabled` helper and moved environment-boolean parsing onto `ServerConfig`; `ServerResult` remains a type alias and therefore cannot have an inherent impl.
- 59–60: removed every `?`, explicit early return, and protobuf import alias from the gRPC adapter; wire types are scoped through `proto::`.
- 61–63: imported `ring` and the standard `TcpListener` at module scope; Tokio's listener is scoped through `net::`.
- 64: added `acorn.standard.validate` while preserving `acorn.research-activity.validate` compatibility.
- 65–68: simplified duplicate matching, inlined error construction, added canonical repository-path validation to `Location`, and moved format inference to `RepositoryFileInput::format`.
- 69–71: introduced `MimeType` and `ResearchActivity` normalization traits with a shared `NormalizedContent` result and inlined linked-path derivation.
- 72: added the public static `Decode` trait and implementations for RAD, RAiD, DataCite, CFF, CKAN, Data Catalog Vocabulary, HuWise, Invenio, Kitfile/ModelKit, text, and host-owned ACRE. ACRE schema export remains composed through `acorn export --schema`.
