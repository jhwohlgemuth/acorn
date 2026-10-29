# Testing Plans

- [CLI coverage](cli-coverage.md) - Close the remaining high-value link, serve, SWHID, binary-error, and test-support gaps.
- [Coverage throughput](coverage-throughput.md) - Make the local test and coverage loop fast enough to run after every change.
- [GitLab shell runner readiness](gitlab-shell-runner-readiness.md) - GSR-02 is blocked. Runner 55645 can start a tagged job, then loses the final status update.
- [Portable PID verification](pid-verification.md) - Consolidate shared PID behavior, run public APIs under Miri, and prove checksum cores with Creusot/Why3.

Persistent benchmark packages remain an operational test artifact, not a separate implementation plan. Comparisons use matching runner/hardware fingerprints and remain informational.
