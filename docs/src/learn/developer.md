# Developer trail

Goal: integrate ACORN's data model, validation, and optional agent tooling into your own tool.

<div class="acorn-growth emoji-growth five-stages" aria-label="Developer trail from seed to tree"><span>Install</span><span>Model</span><span>Agents</span><span>Python</span><span>Web</span></div>

1. [Install ACORN](../getting-started.md) and inspect `acorn help`.
2. Learn the [ACORN research activity model](../concepts/acorn.md).
3. Use the [`acorn-lib` Rust crate](../packages/acorn-lib.md) for native and agent-enabled applications, and extract ACORN's operating instructions with [`acorn skill`](../cli-reference/commands/skill.md).
4. For Python workflows, follow the standalone [`acorn-py` project](https://code.ornl.gov/research-enablement/acorn-py).
5. Review the [WebAssembly-supported surface](../packages/acorn-web.md) before targeting a browser or edge runtime.

When adding a CLI integration, prefer the generated `CommandLineHelp.md` in the repository as the exact option reference and the curated [command pages](../cli-reference/commands/) for workflow guidance.

<p class="acorn-next"><strong>Next stop:</strong>&nbsp;Start building with the <a href="../packages/acorn-lib.md"><code>acorn-lib</code> crate</a>.</p>
