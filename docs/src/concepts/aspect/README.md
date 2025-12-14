# ASPECT
> **A** **S**cientific **P**rescription for the **E**fficient **C**lassification of **T**echnology

The **ASPECT** framework is a standardized methodology for classifying and describing technology components within the ACORN ecosystem. It provides a structured approach to defining the attributes and relationships of various technological elements, ensuring consistency and interoperability across different systems and applications.

**ASPECT** was designed with the goal of unifying our understanding of automation, AI/ML technology, and "classical" software. We focus on "technology" instead of "AI/ML technology" because the latter is a subset of the former. Furthermore, focusing on AI/ML as the end goal is not fruitful or correct. In fact, doing so is backwards. AI/ML software is not novel in any meaningful sense. Even if it was, it would still be 100% predicated on the scientific principles of software.

In the context of technology, AI/ML and automation are the same.

> [!TIP]
> [Read docs.rs documentation for ASPECT](https://docs.rs/acorn-lib/latest/acorn_lib/schema/research_activity/aspect/index.html)

### Key Components
*click arrow to expand or collapse*
<details>
  <summary>💼 <a href="./portability.md">Software Portability</a></summary>

- Limited
- Source
- Containerized
- Installer
- Automated Installer
- WebAssembly
</details>
<details>
  <summary>🤝 <a href="./human-machine-teaming.md">Human-machine teaming</a></summary>

- Manual
- Machine-assisted
- Human-as-primary
- Machine-as-primary
- Human-supervised
- Machine-only
</details>
<details>
  <summary>📈 <a href="./maturity.md">Maturity</a></summary>

Maturity uses an augmented version of technology readiness levels (TRL) and includes levels 1 through 9
</details>
<details>
  <summary>💾 <a href="./data.md">Data</a></summary>

- Real or Synthetic
- Availability
- Modality
- Quality
</details>
<details>
  <summary>🖥️ <a href="./resources.md">(Hardware) Resources</a></summary>

- <span title="Central Processing Unit">CPU</span>
- <span title="Graphics Processing Unit">GPU</span>
- <span title="Tensor Processing Unit">TPU</span>
- <span title="Field-programmable Gate Array">FPGA</span>
- Quantum
- Neuromorphic
- Other
</details>
<details>
  <summary>⚡ <a href="./agency.md">Agency</a></summary>

- Type 0
- Type 1A
- Type 1B
- Type 2
</details>
<details>
  <summary>🎯 <a href="./task-classification.md">Task Classification</a></summary>

- Percieve
- Reason
- Project
</details>

### Real-world Example
The ASPECT framework can be applied in various scenarios

- [AI-driven Gravity Mapping](https://research.ornl.gov/gravity/)
    - 💼 ***Source*** (Level 1)
    - 🤝 ***Machine-assisted*** (Level 1)
    - 🖥️ ***Developed*** (<span title="Technology Readiness Level">TRL</span> 5)
    - 💾 Trained on ***Real***, ***Unavailable***, ***Silver*** quality, ***Textual*** modality data
    - 🖥️ ***GPU***
    - ⚡ ***Type 2***
    - 🎯 ***Perceive***, ***Reason***
