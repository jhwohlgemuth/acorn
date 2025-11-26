# Commands
- [🕵️‍♂️ Audit](./audit.md)
- [✅ Check](./check.md)
- [👨‍⚕️ Doctor](./doctor.md)
- [📥 Download](./download.md)
- [📤 Export](./export.md)
- [🤖 Format](./format.md)
- [🕸️ Link](./link.md)

## ACORN Command Workflow
```mermaid
graph LR
    User -.->|create| A
    A[RAD] --> B[check]
    B --> A
    B --> C[format]
    C --> D[link]
    C --> E[export]
```