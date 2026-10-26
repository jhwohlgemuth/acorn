# Sponsor Highlights demo bundle

[Download the demo bundle](_assets/sponsor-highlights-demo.zip). Archive paths are relative to `.wiki/features/`; extract there in a fresh ACORN checkout. Start with the [walkthrough](walkthrough.md) or its [PDF](_assets/demo-walkthrough.pdf), refreshed October 26 with SharePoint and GitLab branches, current commands, confirmation controls, the two-branch notification map and captured screens; its rehearsal command uses the local `_assets/rehearse.py` and requires the current ACORN CLI and Chrome.

The [manifest](_assets/demo-package-manifest.json) resolves its `files[].path` entries from `pathBase`, relative to the manifest's own directory. It lists the current packaged files and their hashes, excluding itself and the ZIP. The bundle carries the contributor/reviewer and developer PDFs, retained demo evidence and notification walkthrough across the three plan folders.

The recorded JSONL bytes and historical execution records are unchanged. Paths shown inside captured PDFs, screenshots or transcripts are historical; use the walkthrough for current invocation paths.

The SharePoint receipt/export and Teams tests are separate from the retained GitLab-to-local-report rehearsal. The PDFs label the unaccepted SharePoint download-to-CLI and standard-user access steps. This documentation refresh does not change or rerun the frozen source evidence.

SharePoint now provides a combined `logbook.jsonl`, refreshed from list rows every five minutes. Individual UUID files are receipt evidence. The frozen demo still uses its retained GitLab fixture; independent SharePoint download-to-CLI acceptance remains open.
