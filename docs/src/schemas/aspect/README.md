# ASPECT Framework
> **A** **S**cientific **P**rescription for the **E**fficient **C**lassification of **T**echnology

The **ASPECT** framework is a standardized methodology for classifying and describing technology components within the ACORN ecosystem. It provides a structured approach to defining the attributes and relationships of various technological elements, ensuring consistency and interoperability across different systems and applications.

**ASPECT** was designed with the goal of unifying our understanding of automation, AI/ML technology, and "classical" software. We focus on "technology" instead of "AI/ML technology" because the latter is a subset of the former. Furthermore, focusing on AI/ML as the end goal is not fruitful or correct. In fact, doing so is backwards. AI/ML software is not novel in any meaningful sense. Even if it was, it would still be 100% predicated on the scientific principles of software.

In the context of technology, AI/ML and automation are the same.

[Read docs.rs documentation for ASPECT](https://docs.rs/acorn-lib/latest/acorn_lib/schema/research_activity/aspect/index.html)

### Key Components
- 💾 [Data](./data.md)
- 💼 [Portability](./portability.md)
- ⚡ [Efficacy](./efficacy.md)
- 🤝 [Human-machine teaming](./human-machine-teaming.md)
- 🖥️ [(Hardware) Resources](./resources.md)
- 📈 [Maturity](./maturity.md)
- 🎯 [Task classification](./task-classification.md)

### Real-world Example
The ASPECT framework can be applied in various scenarios

- [AI-driven Gravity Mapping](https://research.ornl.gov/gravity/)
    - Data
        - ***Real***
        - ***Unavailable*** to public
        - ***Silver*** quality
        - ***Textual*** modality training data
    - Software Portability
        - ***Source*** (Level 1)
            - Has restrictive hardware requirements (e.g., high-end GPU with CUDA support)
            - Provided as a software repository and requires significant manual setup to run
            - Common with research software that pushes the boundaries of current technology
    - Efficacy
        - ***Type 2***
            - Ingests training data
            - Creates machine learning models using PyTorch
            - Creates a global map of gravity anomalies
    - Human-machine teaming
        - ***Machine-assisted*** (Level 1)
    - Maturity
        - ***Developed*** (<span title="Technology Readiness Level">TRL</span> 5)
    - Task classification
        - ***Perceive***
        - ***Reason***
