# Feature Plans

## In progress

- [ModelKit and ModelPack OCI interoperability](oci-model-artifact-downloads.md) - Consume native ModelKits and standards-conformant ModelPacks through a normalized, safely materializing OCI plan while preserving GGUF inference policy.
- [Portable ModelKit and Kitfile schema model](modelkit-schema-model.md) - Match KitOps Kitfile 1.0.0 (`pkg/artifact/kitfile.md` + `pkg/artifact/kitfile.go` at the pinned commit) with explicit authoring-YAML and resolved-JSON boundaries and portable validation.
- [Unified import and download with BagIt archives](unified-import-download-bagit.md) - Finish the shared bucket transfer pipeline and add inferred ZIP, TAR, TAR.GZ, and 7z BagIt import and export.
- [SWHID persistent-identifier hardening](swhid-persistent-identifier.md) - Add pinned conformance, resolver-failure, filesystem-edge-case, and Git object-format coverage to the implemented SWHID lifecycle.

## Planned

- [Agent inference and verified-fix roadmap](opencode-acp-inference.md) - Stage prompt hydration, ACP/OpenAI-compatible inference, structured fix suggestions, and verified deterministic remediation.
- [RAD metadata standards alignment](rad-metadata-standards-alignment.md) - Align activity metadata with RAiD and output interchange with DataCite, DCAT, persistent identifiers, and contributor-role vocabularies.
- [First-class RRID research-resource support](rrid-research-resource-support.md) - Add portable RRID parsing and discovery, typed RAD relations, standards interchange, and contract-gated SciCrunch resolution and search.
- [Research activity changelog](research-activity-changelog.md) - Add a manually authored, timestamped timeline of code and other research developments with explicit progress states and provenance for future dynamically discovered entries.
- [Open Compute infrastructure schema alignment](ocp-infrastructure-schema-alignment.md) - Add portable infrastructure and OCP contribution, recognition, and management-profile models while consolidating schema validation and standard module layout.
- [ZON serialization support](zon-format-support.md) - Add a pinned strict ZON codec and integrate `.zonf` with RAD I/O, CLI workflows, crosswalks, discovery, and GitLab review.
