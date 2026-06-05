# Security Policy

Thank you for helping keep ACORN secure.

If you discover a security vulnerability, please report it privately and do not open a public issue.

## Supported Versions

We release patches for security vulnerabilities for the following versions:

| Version | Supported          |
| ------- | ------------------ |
| Latest  | :white_check_mark: |
| < Latest| :x:                |

## Reporting a Vulnerability

- Email: research@ornl.gov
- Include: affected component, reproduction steps, impact, and suggested fix (if known)
- Please provide enough detail for us to reproduce and verify the issue quickly

## Response Expectations

- We will acknowledge receipt as soon as possible
- We will investigate and prioritize based on severity and impact
- We will coordinate disclosure timing with the reporter when appropriate

## Scope

This policy applies to this repository and officially maintained ACORN releases.

## Security Considerations

When using `acorn-cli` or `acorn-lib`, please be aware of the following security considerations:

- Treat YAML input as untrusted. Workspace crates now use `serde_norway`/`unsafe-libyaml-norway` for YAML parsing, but untrusted YAML can still trigger parser edge cases. Prefer trusted input sources and avoid exposing YAML parsing to arbitrary public input.
- Run ACORN with least privilege. Some commands read from the filesystem, fetch remote content, or generate output artifacts; avoid running the CLI against sensitive directories or with elevated permissions unless required.
- Avoid placing secrets on the command line. Command arguments can be exposed through shell history and process listings; prefer environment variables, keyring-backed storage, or local configuration files with appropriate permissions.
- Review downloaded or gathered metadata before redistribution. ACORN can aggregate external content and links; validate provenance and sanitize outputs before publishing generated documents.
- Keep dependencies current and re-run security audits regularly. Several advisories listed below are addressed by upstream version upgrades and should be prioritized as part of routine maintenance.

## Known Related RustSec Security Advisories

| Advisory | CVSS[^cvss] | Crate | Issue | Impact | Patched[^upgrade-planning] / Notes |
|---|---|---|---|---|---|
| [RUSTSEC-2020-0071](https://rustsec.org/advisories/RUSTSEC-2020-0071.html) | 6.2 (MEDIUM) | `time` | Potential segfault from unsynchronized environment-variable access in local-offset functions. | Crash / denial of service. | Patched in `>=0.2.23`; no patch for `0.1.x` (upgrade to `0.2.23+` or `0.3.x`). |
| [RUSTSEC-2023-0071](https://rustsec.org/advisories/RUSTSEC-2023-0071.html) | 5.9 (MEDIUM) | `rsa` | "Marvin Attack" timing side-channel in RSA operations. | Potential private-key leakage / cryptographic compromise. | No patched versions currently listed by RustSec. |
| [RUSTSEC-2024-0388](https://rustsec.org/advisories/RUSTSEC-2024-0388.html) | N/A | `derivative` | Crate is unmaintained[^unmaintained]. | Security maintenance risk. | Migrate to a maintained alternative. |
| [RUSTSEC-2024-0436](https://rustsec.org/advisories/RUSTSEC-2024-0436.html) | N/A | `paste` | Crate is unmaintained[^unmaintained]. | Security maintenance risk. | Migrate to a maintained alternative. |
| [RUSTSEC-2025-0056](https://rustsec.org/advisories/RUSTSEC-2025-0056.html) | N/A | `adler` | Crate is no longer actively maintained[^unmaintained]. | Security maintenance risk. | Switch to a maintained alternative. |
| [RUSTSEC-2025-0134](https://rustsec.org/advisories/RUSTSEC-2025-0134.html) | N/A | `rustls-pemfile` | Crate is unmaintained[^unmaintained]; repository archived. | Security maintenance risk. | No patched versions listed; migrate to `rustls-pki-types` PEM APIs. |
| [RUSTSEC-2025-0141](https://rustsec.org/advisories/RUSTSEC-2025-0141.html) | N/A | `bincode` | Crate is unmaintained[^unmaintained] (development ceased). | Security maintenance risk. | No patched versions listed; migrate to a maintained alternative. |
| [RUSTSEC-2026-0009](https://rustsec.org/advisories/RUSTSEC-2026-0009.html) | 6.8 (MEDIUM) | `time` | Stack exhaustion on malformed / adversarial RFC 2822 parsing. | Denial of service. | Affected `>=0.3.6, <0.3.47`; patched in `>=0.3.47`. |
| [RUSTSEC-2026-0037](https://rustsec.org/advisories/RUSTSEC-2026-0037.html) | 8.7 (HIGH) | `quinn-proto` | Panic from invalid QUIC transport parameters. | Denial of service. | Patched in `>=0.11.14`. |
| [RUSTSEC-2026-0044](https://rustsec.org/advisories/RUSTSEC-2026-0044.html) | N/A | `aws-lc-sys` | Common Name validation bypass for wildcard / raw UTF-8 CN values. | Certificate name-validation bypass. | Patched in `>=0.39.0`. |
| [RUSTSEC-2026-0045](https://rustsec.org/advisories/RUSTSEC-2026-0045.html) | 5.9 (MEDIUM) | `aws-lc-sys` | Timing side-channel in AES-CCM tag verification. | Potential secret leakage via timing analysis. | Patched in `>=0.38.0`. |
| [RUSTSEC-2026-0046](https://rustsec.org/advisories/RUSTSEC-2026-0046.html) | 7.5 (HIGH) | `aws-lc-sys` | `PKCS7_verify()` certificate-chain validation bypass with multiple signers. | Signature/certificate validation bypass risk. | Patched in `>=0.38.0`. |
| [RUSTSEC-2026-0047](https://rustsec.org/advisories/RUSTSEC-2026-0047.html) | 7.5 (HIGH) | `aws-lc-sys` | `PKCS7_verify()` signature validation bypass for authenticated attributes. | Signature verification bypass / integrity risk. | Patched in `>=0.38.0`. |
| [RUSTSEC-2026-0048](https://rustsec.org/advisories/RUSTSEC-2026-0048.html) | 7.4 (HIGH) | `aws-lc-sys` | CRL distribution-point matching logic error can bypass revocation checks. | Revoked certificate may be accepted. | Patched in `>=0.39.0`. |
| [RUSTSEC-2026-0049](https://rustsec.org/advisories/RUSTSEC-2026-0049.html) | N/A | `rustls-webpki` | Faulty CRL distribution-point matching can ignore valid revocation data. | Revoked certificate may be accepted (policy-dependent). | Patched in `>=0.103.10`. |
| [RUSTSEC-2026-0097](https://rustsec.org/advisories/RUSTSEC-2026-0097.html) | N/A | `rand` | Unsoundness with custom logger + `thread_rng`/`rng` reseed path. | Undefined behavior in specific logging/reseeding scenarios. | Patched in `>=0.10.1`, `>=0.9.3 <0.10.0`, and `>=0.8.6 <0.9.0`. |
| [RUSTSEC-2026-0098](https://rustsec.org/advisories/RUSTSEC-2026-0098.html) | N/A | `rustls-webpki` | URI name constraints were incorrectly accepted/ignored. | Name-constraints enforcement bypass in affected cert-validation flows. | Patched in `>=0.103.12 <0.104.0-alpha.1` and `>=0.104.0-alpha.6`. |
| [RUSTSEC-2026-0099](https://rustsec.org/advisories/RUSTSEC-2026-0099.html) | N/A | `rustls-webpki` | Wildcard certificates were incorrectly accepted under permitted subtree constraints. | Name-constraints enforcement bypass in affected cert-validation flows. | Patched in `>=0.103.12 <0.104.0-alpha.1` and `>=0.104.0-alpha.6`. |
| [RUSTSEC-2026-0104](https://rustsec.org/advisories/RUSTSEC-2026-0104.html) | N/A | `rustls-webpki` | Reachable panic in CRL parsing (`onlySomeReasons` empty BIT STRING handling). | Denial of service via panic. | Patched in `>=0.103.13 <0.104.0-alpha.1` and `>=0.104.0-alpha.7`. |

## Repository Remediation Status (Current Lockfiles)

The items below are based on currently locked versions in `Cargo.lock`.

### Addressable by Version Upgrades

- `RUSTSEC-2026-0009`: `time` is locked at `0.3.40`; upgrade to `>=0.3.47`.
- `RUSTSEC-2026-0037`: `quinn-proto` is locked at `0.11.13`; upgrade to `>=0.11.14`.
- `RUSTSEC-2026-0044`, `RUSTSEC-2026-0045`, `RUSTSEC-2026-0046`, `RUSTSEC-2026-0047`, `RUSTSEC-2026-0048`: `aws-lc-sys` is locked at `0.37.1`; upgrade to `>=0.39.0`.
- `RUSTSEC-2026-0049`, `RUSTSEC-2026-0098`, `RUSTSEC-2026-0099`, `RUSTSEC-2026-0104`: `rustls-webpki` is locked at `0.103.9`; upgrade to `>=0.103.13`.
- `RUSTSEC-2026-0097`: `rand` is locked at `0.8.5`, `0.9.2`, and `0.10.1`; update `0.8.x` to `>=0.8.6` and `0.9.x` to `>=0.9.3` (`0.10.1` is already patched).

### Requires Migration / Risk Acceptance

- `RUSTSEC-2023-0071`: `rsa` is locked at `0.9.10`; no patched RustSec version is listed.
- `RUSTSEC-2024-0388`: `derivative` is locked at `2.2.0` and is unmaintained[^unmaintained].
- `RUSTSEC-2024-0436`: `paste` is locked at `1.0.15` and is unmaintained[^unmaintained].
- `RUSTSEC-2025-0056`: `adler` is locked at `1.0.2` and is unmaintained[^unmaintained].
- `RUSTSEC-2025-0134`: `rustls-pemfile` is locked at `1.0.4` and is unmaintained[^unmaintained].
- `RUSTSEC-2025-0141`: `bincode` is locked at `1.3.3` and `2.0.1` and is unmaintained[^unmaintained].
- `RUSTSEC-2020-0071`: `time 0.1.45` is still present transitively and has no `0.1.x` patch.

## Preferred Languages

We prefer all communications to be in English.

## Policy Updates

This security policy may be updated from time to time. Please check back regularly for any changes.

---

[^cvss]: CVSS = [Common Vulnerability Scoring System](https://nvd.nist.gov/vuln-metrics/cvss)
[^unmaintained]: "Unmaintained" advisories are not always immediate exploitability issues, but they do indicate that security fixes may stop arriving.
[^upgrade-planning]: For upgrade planning, treat advisories with patched versions as direct remediation items and unmaintained[^unmaintained] crates as migration items.