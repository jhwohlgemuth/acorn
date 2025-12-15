# 🤝 Human-Machine Teaming

In the context of **ASPECT**, human-machine teaming describes the adaptive bi-directional team interaction among humans and machines that augments human capabilities for improved outcomes. This attribute incorporates on prior work by the [Society of Automative Engineers' six levels of driving automation](https://www.sae.org/standards/j3016_202104-taxonomy-definitions-terms-related-driving-automation-systems-road-motor-vehicles) and expands on <span title="International Organization for Standardization">ISO</span> definitions to partition technology into distinct and employable categories.

> [!TIP]
> ISO 22989[^ISO] defines human-machine teaming as "*integration of human interaction with machine intelligence capabilities*."

## Levels
### Manual (HMT 0)
```pikchr
scale = 0.7
circle "Human" fill white
move 2cm
diamond "Task" fill white width 0.8
arrow thick thick from first circle.e to first diamond.w color 0x00662C
```

### Machine-assisted (HMT 1)
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