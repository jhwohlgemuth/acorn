# Sponsor Highlights demo bundle

[Download the demo bundle](_assets/sponsor-highlights-demo.zip). Archive paths are relative to `.wiki/features/`; extract there in a fresh ACORN checkout. Start with the [walkthrough](walkthrough.md); its rehearsal command uses the local `_assets/rehearse.py` and requires the current ACORN CLI and Chrome.

The [manifest](_assets/demo-package-manifest.json) resolves its `files[].path` entries from `pathBase`, relative to the manifest's own directory. It lists the current packaged files and their hashes, excluding itself and the ZIP. The bundle carries the contributor/reviewer and developer PDFs, retained demo evidence and notification walkthrough across the three plan folders.

The recorded JSONL bytes and historical execution records are unchanged. Paths shown inside captured PDFs, screenshots or transcripts are historical; use the walkthrough for current invocation paths.
