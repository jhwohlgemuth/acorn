# Testing Plans

- [CLI coverage](cli-coverage.md) - Close the remaining high-value link, serve, SWHID, binary-error, and test-support gaps.
- [Coverage throughput](coverage-throughput.md) - Make the local test and coverage loop fast enough to run after every change.
- [GitLab shell runner readiness](gitlab-shell-runner-readiness.md) - GSR-02 is blocked. Runner 55645 can start a tagged job, then loses the final status update.
- [Portable PID verification](pid-verification.md) - Prove the shared ORCID/ISNI checksum with Verus first, then connect production callers, consolidate PID helpers, and run public APIs under Miri.
- [Report PowerPoint visual regression](powerpoint-report-visual-regression.md) - Generate report decks, verify slide content, and compare reviewed LibreOffice renders on Linux and native Windows.
- [WebMCP and MCP interoperability](webmcp-evaluations.md) - Layer contract tests, Inspector smoke, conformance, browser evaluations, and independent live-client controls.

Persistent benchmark packages remain an operational test artifact, not a separate implementation plan. Comparisons use matching runner/hardware fingerprints and remain informational.
