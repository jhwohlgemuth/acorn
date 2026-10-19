# Commands

- [✅ Check](./check.md)
- [🏗️ Create](./create.md)
- [👨‍⚕️ Doctor](./doctor.md)
- [🔀 Diff](./diff.md)
- [📥 Download](./download.md)
- [📤 Export](./export.md)
- [🤖 Format](./format.md)
- [🌾 Gather](./gather.md)
- [📥 Import](./import.md)
- [🧠 Infer](./infer.md)
- [📥 Intake](./intake.md)
- [🕸️ Link](./link.md)
- [🔌 MCP](./mcp.md)
- [📡 Serve](./serve.md)
- [🧩 Skill](./skill.md)
- [🧬 SWHID](./swhid.md)
- [🔄 Sync](./sync.md)
- [🖥️ TUI](./tui.md)

## Command workflows

### Research activity data

```pikchr
scale = 0.8
Start: circle radius 0.07in fill 0x00662C thickness 0
arrow right 0.532in thick color 0x00662C
box "Draft RAD" wid 0.9in ht 0.42in rad 0.08in fill white thickness 0
arrow right 0.532in thick color 0x00662C
box "check" wid 0.72in ht 0.42in rad 0.08in fill white thickness 0
arrow right 0.532in thick color 0x00662C
Format: box "format" wid 0.78in ht 0.42in rad 0.08in fill white thickness 0
arrow right 0.732in thick color 0x00662C
Link: box "link" wid 0.68in ht 0.42in rad 0.08in fill white thickness 0
arrow right 0.732in thick color 0x00662C
End: circle radius 0.1in fill white thickness 0
circle radius 0.05in fill 0x00662C thickness 0 at End
Export: box "export" wid 0.78in ht 0.42in rad 0.08in fill white thickness 0 at Link+(0,-0.65in)
arrow from Format.e right 0.266in then down until even with Export then to Export.w thick color 0x00662C
arrow from Export.e right 0.266in then up until even with End then to End.w thick color 0x00662C
```

### Persistent identifiers

```pikchr
scale = 0.8
Start: circle radius 0.07in fill 0x00662C thickness 0
arrow right 0.802in thick color 0x00662C
box "Documents | URLs" wid 1.25in ht 0.42in rad 0.08in fill white thickness 0
arrow right 0.802in thick color 0x00662C
box "gather" wid 0.78in ht 0.42in rad 0.08in fill white thickness 0
arrow right 0.802in thick color 0x00662C
box "Draft RAD" wid 0.9in ht 0.42in rad 0.08in fill white thickness 0
arrow right 0.802in thick color 0x00662C
End: circle radius 0.1in fill white thickness 0
circle radius 0.05in fill 0x00662C thickness 0 at End
```

### Local AI and agents

ACORN keeps its core research-data commands deterministic while offering optional tools for local AI and agent workflows:

- [`import model`](./import.md#model-metadata) catalogs GGUF metadata without downloading weights.
- [`download model`](./download.md#models) selects and downloads local GGUF weights.
- [`sync`](./sync.md) configures supported agent and inference applications to use those models.
- [`skill`](./skill.md) gives compatible AI agents ACORN-specific operating instructions.
- [`infer`](./infer.md) uses Needle for free-form tool selection; `infer run` hydrates a known embedded prompt and runs it through ACP or an OpenAI-compatible endpoint.
- [`mcp`](./mcp.md) discovers and invokes tools explicitly approved in application configuration.
