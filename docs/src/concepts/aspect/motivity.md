# Motivity

Motivity classifies how a technology receives information from its environment, acts on that environment, and retains state. It uses three primitives: perception for input, projection for output, and comprehension for internal state and computation.

This attribute is related to [autonomy](./autonomy.md), but the two measure different properties. Motivity describes a system's interaction and state. Autonomy describes how humans and machines divide responsibility.

> [!NOTE]
> "Motivity" is uncommon in software classification. ASPECT uses it for a system's capacity to produce change through input, output, and internal state.

## Levels

Motivity has six ordinal levels ordered by interaction complexity. Perception is one-way input, projection is one-way output, and comprehension is internal state and computation.

> [!NOTE]
> Throughout this document, "comprehension" refers to a technology's ability to receive, store, and act on data. It does not imply general intelligence, consciousness, or AGI. A hash function, a state machine, and a lookup table exhibit comprehension in this limited sense because they maintain internal state that affects their behavior.

*Stateless* means no memory between interactions; *stateful* means the technology retains internal state that influences future behavior.

| Level | Name | Perception | Projection | Comprehension | Stateless/Stateful | Examples |
|-------|------|------------|------------|---------------|--------------------|---------|
| 0 | **Inert** | ✗ | ✗ | ✗ | None | A hammer or simple pulley system |
| 1 | **Computational** | ✗ | ✗ | ✓ | Stateful (internal) | Digital model[^twin], simulation |
| 2 | **Perceptive** | ✓ | ✗ | ✗ | Stateless | Data shadow[^twin], sensor log |
| 3 | **Projective** | ✗ | ✓ | ✗ | Stateless | Cron job that emails status |
| 4 | **Reactive** | ✓ | ✓ | ✗ | Stateless | Smart light switch, relay |
| 5 | **Adaptive** | ✓ | ✓ | ✓ | Stateful | Digital twin[^twin], autonomous agent |

### Inert (level 0)
Technology at this level has no computational capability and no interaction with the environment. It is purely passive. Examples include a hammer, a simple pulley system, or any purely physical tool with no embedded intelligence.

```pikchr
scale = 0.7
circle "Self" fill white
```

### Computational (level 1)
Technology at this level performs self-contained computation but has no input from or output to the environment. Comprehension (internal processing) is present, but neither perception nor projection are. Examples include a digital model or simulation that runs in isolation.

```pikchr
scale = 0.7
circle "Self" fill white
arrow thick <-> from last circle.s down 2cm "    Comprehension" ljust color 0x00662C
circle "Self" fill white
```

### Perceptive (level 2)
Technology at this level receives input from the environment (one-way input binding) but produces no output and has no internal comprehension. The technology observes but does not process or act. Examples include a data shadow or a sensor that logs readings to storage without analyzing them.

```pikchr
scale = 0.7
E1: circle "Env" fill white width 0.5
    move 2cm
S1: circle "Self" fill white
    arrow thick from E1.e to S1.w "Perception" above "" above color 0x00662C
```

### Projective (level 3)
Technology at this level produces output to the environment (one-way output binding) but receives no input and has no internal comprehension. A cron job that emails a status report projects information without sensing or reasoning about its environment.

```pikchr
scale = 0.7
S1: circle "Self" fill white
    move 2cm
E1: circle "Env" fill white width 0.5
    arrow thick from S1.e right 2cm "Projection" above "" above color 0x00662C
```

### Reactive (level 4)
Technology at this level has two-way binding with the environment (perception and projection) but is stateless. It responds to input without an internal model or memory of past interactions. Examples include a smart light switch that turns lights on when motion is detected and a relay that closes a circuit when a threshold is crossed.

```pikchr
scale = 0.7
E1: circle "Env" fill white
    arrow thick -> from E1.e right 2cm "Perception" above "" above color 0x00662C
S1: circle "Self" fill white
    arrow thick -> from S1.e right 2cm "Projection" above "" above color 0x00662C
E2: circle "Env" fill white
```

### Adaptive (level 5)
Technology at this level has full two-way binding (perception and projection) with internal comprehension. It maintains state, builds an internal model of its environment, and changes its behavior based on accumulated experience. Examples include a digital twin that continuously synchronizes with a physical system and predicts future states and an autonomous agent that plans and adapts.

```pikchr
scale = 0.7
E1: circle "Env" fill white
    arrow thick -> from E1.e right 2cm "Perception" above "" above color 0x00662C
S1: circle "Self" fill white
    arrow thick <-> from S1.s down 2cm "    Comprehension" ljust color 0x00662C
S2: circle "Self" fill white
    arrow thick -> from S1.e right 2cm "Projection" above "" above color 0x00662C
E2: circle "Env" fill white
```

[^twin]: Y. K. Liu, S. K. Ong, and A. Y. C. Nee, "State-of-the-art survey on digital twin implementations," Adv. Manuf., vol. 10, no. 1, pp. 1-23, Mar. 2022, doi: 10.1007/s40436-021-00375-w.
