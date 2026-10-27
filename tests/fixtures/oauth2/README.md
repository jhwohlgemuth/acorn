# Synthetic OAuth fixtures

All authorities, accounts and client IDs in this directory are invented. No value is a usable credential. `issuers.json` drives the provider-neutral contract: a token for one issuer, client, audience or scope set must never satisfy the other request. The `config/` files exercise first-declared profile selection and order-preserving JSON, JSONC and YAML round-trips when SPD-23 implements ACRE parsing.
