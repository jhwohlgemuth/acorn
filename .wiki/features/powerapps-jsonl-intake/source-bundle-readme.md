# Sponsor Highlights source bundle

[Download the source bundle](_assets/sponsor-highlights-source-bundle.zip). Archive paths are relative to `.wiki/features/`; extract there in a fresh ACORN checkout to retain cross-plan links and access to repository documentation. The bundle contains the same selected source and acceptance records as before the folder migration, organized by their owning plan.

Start with the [pilot runbook](pilot-runbook.md), [notification walkthrough](notification-walkthrough.md), or [manual rehearsal](../manual-jsonl-ingestion-graduation/walkthrough.md). The [manifest](_assets/source-bundle-manifest.json) resolves its `files[].path` entries from `pathBase`, relative to the manifest's own directory. It records SHA-256 hashes of the current bundled files and excludes itself and the ZIP.

This is a source backup, not an importable Power Platform solution. Historical transcripts, screenshots and PDFs retain the paths and commands recorded when they were captured. Current Markdown links and executable commands use plan-local paths.
