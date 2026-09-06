# ASPECT
> **A** **S**cientific **P**aradigm for the **E**fficient **C**lassification of **T**echnology

<img src="../../_assets/aspect_attributes.png" alt="ASPECT Attributes" width="200" />

*portability* | *autonomy* | *maturity* | *motivity*

The **ASPECT** framework is a standardized methodology for classifying and describing technology. It defines underlying dimensions of technology and uses them to map projects into a shared space, allowing clearer insight into their capabilities, requirements, and relationships.

**ASPECT** treats automation, AI/ML, and classical software as subsets of technology, providing one shared classification space.

> [!TIP]
> [Read docs.rs documentation for ASPECT](https://docs.rs/acorn-schema/latest/acorn_schema/schema/research_activity/aspect/index.html)

## Key Components

### Primary attributes

*click arrow to expand or collapse*
<details>
  <summary>📦 <a href="./portability.md">Portability</a></summary>

- Limited (level 0)
- Source (level 1)
- Containerized (level 2)
- Installer (level 3)
- Package manager (level 4)
- WebAssembly (level 5)
</details>
<details>
  <summary>🪴 <a href="./maturity.md">Maturity</a></summary>

- Unvalidated (level 0)
- Discovery (level 1)
- Concept (level 2)
- Development (level 3)
- Prototype (level 4)
- Proven (level 5)
</details>
<details>
  <summary>🤝 <a href="./autonomy.md">Autonomy</a></summary>

- Manual (level 0)
- Machine-assisted (level 1)
- Human-as-primary (level 2)
- Machine-as-primary (level 3)
- Human-supervised (level 4)
- Machine-only (level 5)
</details>
<details>
  <summary>🦾 <a href="./motivity.md">Motivity</a></summary>

- Inert (level 0)
- Computational (level 1)
- Perceptive (level 2)
- Projective (level 3)
- Reactive (level 4)
- Adaptive (level 5)
</details>

### 💾 <a href="./data.md">Data</a>

Data is not included in primary four attributes, but is very important when characterizing technology.

Data has multiple dimensions
- Real or Synthetic
- Availability
- Modality
- Quality

## Example

The ASPECT framework can be applied in various scenarios

### Consider a fictional project with the following characterization:
| Attribute | Level | Details |
|---|---|---|
| 📦 Portability | Level 1 | ***Source*** |
| 🪴 Maturity | Level 3 | ***Development*** |
| 🤝 Autonomy | Level 1 | ***Machine-assisted*** |
| 🦾 Motivity | Level 2 | ***Perceptive*** |

#### Badge Visualization
ASPECT can be visualized in a badge-like form for visual project portfolio management

<p align="center">
  <img src="../../_assets/aspect_example.png" alt="ASPECT Example" width="165" />
</p>
<p align="center"><em>ASPECT badge for a project</em></p>

The badge below provides immediate feedback on the state of the project and its technology components. It can be used in project dashboards, reports, and presentations to communicate the current status and future goals of the project in a clear and concise manner. The annotated version clarifies the placement of each attribute.

<p align="center">
  <img src="../../_assets/aspect_example_annotated.png" alt="Annotated ASPECT Example" width="500" style="background: white; padding: 8px; border-radius: 8px;" />
</p>
<p align="center"><em>Annotated ASPECT badge</em></p>
