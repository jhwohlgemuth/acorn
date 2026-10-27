# SharePoint intake capture: implementation contract

This records the SPD-02 contract for a future read-only SharePoint source. The commands and configuration below are **not available yet**. The existing local and GitLab intake paths keep their current behavior.

## Tenant feasibility status

As of October 27, 2026, the operator has an enterprise SharePoint account, but no approved Entra public-client registration or delegated Graph consent has been identified. SPD-01 therefore remains blocked; browser access does not prove CLI Graph access. The tenant administrator or application owner needs to identify or approve a public-client application, its tenant/client IDs, system-browser loopback redirect and device-code policy, and the minimum delegated read scopes for this specific export. A read-only Graph metadata/content probe and same-revision byte comparison must then be recorded before any live-read claim. No Power Platform app, flow or permission was changed by this work.

## Operator boundary

An operator supplies a direct HTTPS document URL to `acorn intake stage --batch --config FILE --source URL`. The configured ACRE source mapping determines whether that URL is SharePoint. An unconfigured HTTPS origin fails before credential lookup or network access; it is never tried as GitLab with a Microsoft token. `--offline` rejects a remote URL before authentication or directory creation. Capture saves exact bytes and a receipt beneath the configuration's `.acorn/intake/sources/`; staging then uses the existing local JSONL validator and authority rules. Inspect, apply, and report continue to use the saved local file offline. Capture never invokes the exporter, applies records, or acknowledges remote submissions.

The CLI keeps machine-readable batch output on stdout. Authentication prompts and capture diagnostics use stderr. A generic `download file` command is outside this release.

## ACRE configuration contract

Add optional top-level `auth` to schema version 2 ACRE. Existing documents without it remain valid. `auth.profiles` is an insertion-ordered mapping of unique names to non-secret profiles. `auth.sources` is a sequence of SharePoint mappings. Use an order-preserving map type during JSON, JSONC, and YAML deserialization and serialization; the first declared profile is the default for `auth login` and `auth cache`. Do not sort profiles before selecting the default. Both nested types deny unknown fields.

```yaml
schemaVersion: 2.0.0
auth:
  profiles:
    ornl:
      provider: entra
      tenantId: 00000000-0000-0000-0000-000000000001
      clientId: 00000000-0000-0000-0000-000000000002
      cloud: public
      flow: browser
      cache: preferred
      resources:
        graph:
          audience: https://graph.microsoft.com
          scopes: [Files.Read]
      defaultResource: graph
  sources:
    - id: combined-logbook
      provider: sharepoint
      profile: ornl
      resource: graph
      host: example.sharepoint.com
      sitePath: /sites/ResearchEnablement
      libraryPath: /Sponsor Highlights JSONL Pilot
      itemPath: logbook.jsonl
```

The example identifiers and host are synthetic. `tenantId` and `clientId` must be UUIDs; `cloud` is one of `public`, `gccHigh`, or `dod` and fixes the allowed Entra authority and Graph audience. No custom token, discovery, or authorization endpoint is accepted from ACRE in this release. The profile's resource names are local selectors: each defines one HTTPS audience and a nonempty deduplicated scope set. `defaultResource` must name an entry. A source may select one resource and a subset of its scopes; omitted source scopes use the selected resource's set. It cannot widen either. `flow` is `browser` or `device` and defaults to browser. `cache` is `preferred`, `required`, or `disabled` and defaults to preferred. Passwords, client secrets, tokens, and cache payloads are invalid configuration keys.

The source mapping must have a unique `id`, a named profile, one canonical HTTPS host, canonical site and library paths, and one document-relative item path. Optional `siteId`, `driveId`, and `itemId` are accepted only as a complete verified identity set; if supplied, metadata must agree with the URL. Prefer discovery with delegated read permission; use pinned IDs only when the approved grant cannot discover the resource. No implicit `Sites.Read.All` escalation. Match one source by canonical host/site/library/item segments; zero or multiple matches fail before credentials. Decode URL path segments exactly once, preserving spaces and Unicode. Reject sharing links, view pages, userinfo, query, fragment, non-HTTPS, encoded separators, dot/traversal segments, control characters, and malformed percent escapes.

The resolver may make at most 16 metadata requests and traverse at most two paginated result pages. An item outside that bound fails with a clear configuration/discovery error; it never widens scopes. Canonical source identity is the normalized HTTPS host plus the exactly-once-decoded site, library and item segments, re-encoded with uppercase percent escapes for non-unreserved bytes.

Use existing ACRE explicit-file and discovery precedence, but load auth configuration with `ApplicationConfiguration::resolve` plus `read` and validation. Do not call `load` or `load_with_path`: they import an adjacent `.env`, which auth commands must not do. Resolve once at the CLI boundary into immutable inputs; host operations never rediscover files, load ACRE, or read environment variables. `--profile NAME` selects an exact name; omitted selects the first declared profile; an absent/empty collection or unknown name fails. Intake instead uses its matching source's profile. Show the selected configuration path and profile before interaction. Cache identity uses validated issuer/tenant/client/endpoints/account, never the profile name alone.

## Host and provider interfaces

The optional `acorn-host::auth` feature exports owned, validated request types for authority, public client, redirect policy, resource audience and normalized scopes, account selector, grant flow, interaction policy, and cache policy. A token provider receives a resource-scoped request and returns a secret-wrapped access token plus verified account and grant metadata. Provider operations receive this capability; they never receive a reusable raw Graph token string. Access tokens remain in memory and are opaque to ACORN.

Silent acquisition either succeeds or returns a typed `InteractionRequired` event containing the selected profile, authority, resource and permission explanation. The CLI presents Confirm with No focused, then resumes the specific browser/device grant only on Yes. No/Cancel issues no interactive authorization request. Host code never opens a browser or renders terminal controls. Browser authorization uses PKCE S256, random state and nonce, a listener bound to 127.0.0.1 on an ephemeral port, and an overall five-minute deadline. Register `http://localhost` as the desktop redirect and send `http://localhost:{port}/` for the active listener, subject to SPD-01 approval. Accept one callback at `/` with the expected Host and state, then close the listener; never log the code or verifier. Browser-launch failure may display the validated authorization URL only on an interactive stderr terminal; it is never persisted. An unavailable TTY produces a sign-in-required error.

`auth login --device` overrides the profile flow; otherwise the profile flow applies. Device authorization asks Confirm before requesting a code, emits a typed display event containing only the provider-validated verification URL, user code and expiry, and polls within the provider interval and expiry. `slow_down` adds five seconds as RFC 8628 specifies; 429/503 and transport retries remain within the overall deadline. Denial, expiry, Ctrl-C and cancellation stop polling and reject late results. There is no automatic browser/device fallback or unattended code emission.

The Entra adapter uses a maintained OIDC library to validate ID-token signature, issuer, client audience, time and browser nonce; device flow follows its grant-specific validation rules. The stable account key is issuer plus subject. A missing verified identity or refresh material cannot be persisted under a guessed account. Wrong tenant/account results fail before replacing cache entries. Only Graph resource-scoped access tokens go to Graph; signed content URLs get no bearer header. Another audience needs a separate compatible grant.

## Protected cache contract

The protected store saves only refresh material and associated metadata. Preferred persistence uses a supported native store when available and reports session-only operation otherwise; required fails if it cannot use the store; disabled never reads or writes it. None writes plaintext. Use non-synchronizing macOS Keychain entries, Windows current-user Credential Manager with Local persistence, and a persistent Linux Secret Service collection over session D-Bus. No backend is available in WASM. The native adapters distinguish missing, locked, denied, unavailable, malformed, unsupported-version, oversized and unresolved-timeout states. They never truncate credentials or trigger a desktop unlock dialog in a noninteractive process.

Version and namespace entries by validated authority, tenant, client and opaque account ID. Resource/audience and normalized granted-scope compatibility are checked at acquisition; profile aliases can share only fully matching identities and endpoints. Bound each serialized credential to the backend's supported size; Windows generic credentials have a 2,560-byte blob limit, so oversize material fails without truncation. A bounded protected-path, per-identity lock uses `std::fs::File::try_lock` with a 30-second wait deadline. Under that lock, re-read the store before refresh, then replace rotated material. A non-secret generation counter and tombstone survive scoped clear, so a refresh/sign-in started before clear cannot restore credentials. Blocking native calls may complete late: serialize through confirmed completion or return an unresolved outcome; never interpret timeout as confirmed deletion. Store no token or user code in lock/index files.

Put the non-secret index, generation files and locks beneath a current-user local state directory: macOS Application Support/ACORN/auth, Windows LocalAppData/ACORN/auth, and Linux XDG_STATE_HOME/acorn/auth (falling back to HOME/.local/state/acorn/auth). Name each identity file by the SHA-256 of a length-prefixed issuer, tenant, client and subject tuple; never put those strings in filenames. Create the directory and files owner-only, reject links/reparse points, and atomically replace index/generation content under the identity lock. Index records include only version, opaque account selector, bound issuer/client/tenant digest, backend and alias names. Cap the index at 128 identities and 64 KiB; overflow and malformed records fail without deleting credentials.

`auth cache status` is bounded, local and offline, with no refresh or provider request. `clear --dry-run` computes an exact ACORN-owned deletion set without mutation; plain clear asks Confirm with No focused, and noninteractive clear requires `--yes`. If more than one account matches, require `--account`; never add `--all`. Clear takes the identity lock, advances generation, removes matching records and verifies absence. It leaves other accounts, applications, GitLab credentials and JSONL snapshots intact. Disabled persistence does not prevent an explicit scoped clear of older entries. The global `--clear-cache` remains unrelated. Local clear does not revoke remote consent, browser cookies or sessions.

## Graph bytes and intake receipt

`acorn-lib::io::api::graph` accepts a drive/item identity and injected token provider, returning exact bounded bytes with site/drive/item IDs, size, modified time, and opaque ETag or version evidence. It knows no JSONL schema. Resolve configured SharePoint URLs through bounded Graph metadata requests; a discovery permission failure is explicit. Content capture prefers an immutable version endpoint when available; otherwise require stable identity, size and revision before and after download. Missing revision evidence fails. Limit content to 10 MiB, a 30-second request timeout, a two-minute overall deadline and at most two retries for changing revisions. A 401 gets one scoped refresh; 429/503 obey bounded Retry-After; a preauthenticated redirect gets no bearer and only validated HTTPS destinations. Never log signed URLs.

The intake adapter validates UTF-8, framing and the existing 64 KiB record and 10 MiB total limits without rewriting bytes. Empty content stages as zero records; CRLF is accepted as input and retained byte-for-byte. HTML login pages, invalid UTF-8, partial lines and malformed JSONL fail without a ready artifact. A valid capture can still produce normal staging errors for duplicate/conflicting IDs or authority. Fresh remote failure never silently falls back to an older snapshot.

Publish `logbook.jsonl` and `source.json` together from a same-filesystem restricted temporary directory into an immutable final directory. Identity is the digest of provider, tenant, canonical source identity, remote revision and byte SHA-256, excluding capture time. The version-1 receipt contains those fields, byte and record counts, source modified time, capture time, latest verification time, and whether revision proof was pinned or before/after. It never contains bearer tokens or signed URLs. Reuse an existing directory only after checking receipt identity and stored byte digest; preserve first capture time. On POSIX use 0700 directories and 0600 files; on Windows restrict the directory and files to the current user. Reject symlink/reparse-point redirection and leave prior verified captures intact after interruption.

Compute the directory name as lowercase hex SHA-256 of the UTF-8 sequence `sharepoint-v1`, provider, tenant ID, canonical source URL, opaque revision value and lowercase byte digest, with each field prefixed by its unsigned 32-bit big-endian byte length. This avoids ambiguous concatenation and never includes capture time. Write receipt timestamps as UTC RFC 3339 strings, use `null` for unknown source modification time, and preserve the receipt's first `capturedAt` when a later verification updates `verifiedAt`. A failed temporary publish is removable on the next attempt only after confirming it is within the selected source directory; never replace a final directory.

## Dependency decision and validation

SPD-03 registers `oauth2` 5.0.0 and `openidconnect` 4.0.1 with default HTTP features disabled. Use their custom async HTTP-client interface over ACORN's existing Reqwest 0.13 transport and redirect policy; do not pull their bundled Reqwest 0.12 client or the alpha `oauth2-reqwest` bridge. Register `keyring-core` 1.0.1, `apple-native-keyring-store` 1.0.2 with only `keychain`, `windows-native-keyring-store` 1.1.0 with defaults disabled, and `zbus-secret-service-keyring-store` 1.0.1 with only `rt-tokio-crypto-rust`. Wire each store only for its native target. Reuse workspace `secrecy` and Tokio; no all-backends `keyring` facade or new TLS stack. These crates declare MSRVs at or below ACORN's 1.98 and MIT/Apache-compatible licenses; native runtime acceptance remains separate. Review the resolved lockfile, advisories and feature trees before considering SPD-03 complete.

| Direct dependency | License; declared MSRV | Selected role and feature boundary |
|---|---|---|
| [oauth2 5.0.0](https://docs.rs/crate/oauth2/5.0.0) | MIT OR Apache-2.0; 1.65 | Host auth only; defaults disabled, custom HTTP client. |
| [openidconnect 4.0.1](https://docs.rs/crate/openidconnect/4.0.1) | MIT; 1.65 | Host auth only; defaults disabled, shares OAuth 5. |
| [keyring-core 1.0.1](https://docs.rs/crate/keyring-core/1.0.1) | MIT OR Apache-2.0; 1.85 | Host auth store interface; sample feature disabled. |
| [apple-native-keyring-store 1.0.2](https://docs.rs/crate/apple-native-keyring-store/1.0.2) | MIT OR Apache-2.0; 1.85 | macOS only; keychain feature. |
| [windows-native-keyring-store 1.1.0](https://docs.rs/crate/windows-native-keyring-store/1.1.0) | MIT OR Apache-2.0; 1.88 | Windows only; default search feature disabled. |
| [zbus-secret-service-keyring-store 1.0.1](https://docs.rs/crate/zbus-secret-service-keyring-store/1.0.1) | MIT OR Apache-2.0; 1.88 | Linux only; Tokio and Rust crypto feature. |

The Linux store's upstream dependency currently also enables ZBus's default async-io feature, despite selecting only the Tokio feature directly. The adapter must use the Tokio path; revisit this transitive cost before native Linux acceptance. The resolved host tree has one OAuth 5 and one Reqwest 0.13, with only the target's store in each target-specific tree. The October 27 offline advisory scan found six vulnerabilities in both the previous and updated workspace lockfiles; none was added by this dependency selection. Cross-target Linux compilation needs a cross C compiler unavailable on this macOS host, so the Linux store still requires its own target build and native acceptance.

Synthetic fixtures in `tests/fixtures/oauth2/`, `tests/fixtures/microsoft-graph/`, and `tests/fixtures/sharepoint-intake/` define two issuers, two audiences/scope sets, one non-JSONL binary file, URL rejection, and receipt semantics. They contain no real tenant IDs, credentials, codes or signed URLs. SPD-03 provides compiling seams and explicit unsupported errors only; it does not expose or enable a new CLI route.
