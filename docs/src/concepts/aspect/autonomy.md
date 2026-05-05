# 🤝 Autonomy

In the context of ASPECT, "autonomy" characterizes a technology's level of human-machine teaming (HMT) and describes the adaptive, bi-directional interaction among humans and machines that augments human capabilities for improved outcomes. This attribute builds on prior work by the Society of Automotive Engineers' six levels of driving automation[^sae] and expands on <span title="International Organization for Standardization">ISO</span> definitions to partition technology into distinct and employable categories.

> [!TIP]
> ISO 22989[^ISO] defines human-machine teaming as "*integration of human interaction with machine intelligence capabilities*."

This attribute is important for understanding the degree to which a technology can operate independently of human intervention, as well as the level of human oversight and control required for its operation. By categorizing technologies based on their level of autonomy, we can better understand their capabilities and limitations, and make informed decisions about their use in various applications. The Visidata project popularized its own similar "level of machine assistance" characterizing LLM involvement in code changes[^visidata], and ASPECT's autonomy attribute builds on this work to provide a more comprehensive framework for understanding the relationship between humans and machines in technology.

> [!TIP]
> The [`acorn-lib`](../../packages/acorn-lib.html) crate provides conversion utilities to seemlessly integrate Visidata's AI collaboration levels into ASPECT's autonomy attribute.

## Levels
### Manual (HMT 0)
This level is characterized by the execution of a script where the deterministic outcome is fully known and controlled by the human operator.

```pikchr
scale = 0.7
circle "Human" fill white
move 2cm
diamond "Task" fill white width 0.8
arrow thick thick from first circle.e to first diamond.w color 0x00662C
```

### Machine-assisted (HMT 1)
In this level, the machine assists the human operator in executing a task. The human remains in full control of the task execution, with the machine offering support or suggestions as needed. This might include an iterative script that augments the input during each iteration based on prior outputs.

```pikchr
scale = 0.7
box "Machine" fill white width 75%
move 2cm
circle "Human" fill white
move 2cm
diamond "Task" fill white width 0.8
arrow dashed <-> from first box.e to first circle.w color 0x00662C
arrow thick thick from first circle.e to first diamond.w color 0x00662C
```

### Human as primary (HMT 2)

```pikchr
scale = 0.7
box "Machine" fill white width 75%
move 2cm
circle "Human" fill white
move 2cm
diamond "Task" fill white width 0.8
arrow <-> from first box.e to first circle.w color 0x00662C
arrow thick thick from first circle.e to first diamond.w color 0x00662C
arrow dashed from first box.s down 2cm then right until even with first diamond then to first diamond.s color 0x00662C
```

### Machine as primary (HMT 3)

```pikchr
scale = 0.7
circle "Human" fill white
move 2cm
box "Machine" fill white width 75%
move 2cm
diamond "Task" fill white width 0.8
arrow <-> from first circle.e to first box.w color 0x00662C
arrow thick thick from first box.e to first diamond.w color 0x00662C
arrow dashed from first circle.s down 2cm then right until even with first diamond then to first diamond.s color 0x00662C
```

### Human supervisor (HMT 4)

```pikchr
scale = 0.7
circle "Human" fill white
move 2cm
box "Machine" fill white width 75%
move 2cm
diamond "Task" fill white width 0.8
arrow dashed <-> from first circle.e to first box.w color 0x00662C
arrow thick thick from first box.e to first diamond.w color 0x00662C
```

### Machine only (HMT 5)

```pikchr
scale = 0.7
box "Machine" fill white width 75%
move 2cm
diamond "Task" fill white width 0.8
arrow thick thick from first box.e to first diamond.w color 0x00662C
```

[^ISO]: ISO/IEC 22989:2022 Information Technology --- Artificial Intelligence Concepts and Terminology
[^sae]: SAE J3016 Taxonomy and Definitions for Terms Related to Driving Automation Systems: https://www.sae.org/standards/j3016_202104-taxonomy-definitions-terms-related-driving-automation-systems-road-motor-vehicles
[^visidata]: Visidata AI Collaboration Levels: https://www.visidata.org/blog/2026/ai/