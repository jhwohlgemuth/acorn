# Commands
- [🕵️‍♂️ Audit](./audit.md)
- [✅ Check](./check.md)
- [👨‍⚕️ Doctor](./doctor.md)
- [📥 Download](./download.md)
- [📤 Export](./export.md)
- [🤖 Format](./format.md)
- [📥 Import](./import.md)
- [🕸️ Link](./link.md)

## Command Workflow
```mermaid
stateDiagram
    direction LR
    [*] --> RAD: create
    RAD --> check
    check --> RAD
    check --> format
    format --> link
    format --> export
```
