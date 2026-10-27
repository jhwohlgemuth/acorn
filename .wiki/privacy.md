---
type: Note
title: "Sanitized wiki copies"
description: "Privacy treatment of account identifiers in guides, screenshots, fixtures and recovery packages."
---

# Privacy treatment

On October 27, 2026, the owner requested removal of the three account identifiers used in participant testing from all wiki assets, explicitly including backups and test records. Generic roles replace account labels; addresses use the reserved `example.invalid` domain. Screenshots mask the corresponding text with opaque rectangles. PDFs and nested archives are included. Receipt IDs, flow IDs and historical test outcomes remain as context.

# Evidence and recovery limits

**These are sanitized copies, not byte-identical originals.** Historical success statements and original hashes describe the pre-redaction material. They do not prove the current sanitized payload, native package, screenshot or data snapshot was tested or imported. Sanitization is not a new acceptance run. The file inventory records current hashes separately; original history is not relabeled as new verification.

- Native Power Platform packages, canvas downloads and clipboard definitions may contain placeholder account settings. Resolve them through the approved operator configuration and revalidate an isolated import before reuse. Do not deploy an `example.invalid` recipient or membership rule.
- Exact-byte payloads, digest fixtures, archives and rehearsal manifests may no longer match historical checksums or remote commits. Rebaseline intentionally in an isolated synthetic fixture before execution. Do not rewrite immutable remote history to match these copies.
- Image redaction changes pixels and metadata. Screenshots remain contextual illustrations; covered account text cannot be recovered from the saved image pixels. A sanitized screenshot is not an unmodified capture.
- Historical acceptance manifests retain historical checksums. Use [the file inventory](_assets/privacy-manifest.json) for current outer-file hashes. It preserves the original sanitization checks separately from later maintenance scans; a refreshed hash does not mean every image was OCR-scanned again or a package was reimported. Archive-member hashes can differ even when the outer filename is unchanged.

No live Power Apps, Power Automate, SharePoint, GitLab, account, connection, credential or notification setting was changed by this privacy edit. Repository history, earlier chat messages and files outside the wiki are outside this sanitization pass; distributing an old commit or previously downloaded PDF can still expose earlier content.
