# 🦾 Motivity

## Discussion
"Motivity" is a technology's ability to exert power over its environment. In the context of ASPECT, this involves how a given technology interacts with its environment and its degree of autonomy in performing tasks. As a model of interaction, **motivity** is closely related to the [autonomy attribute](./autonomy.md). **Motivity** can also be viewed in terms of data binding: no binding, one-way binding, and two-way binding.

**Motivity** is a somewhat uncommon word. It was chosen for the ASPECT framework in part because deliberate ambiguity can foster conceptual depth. Rarity minimizes external preconceptions, enabling custom meaning layering without the baggage of a widely used term.

**Motivity** has been defined in various contexts across philosophy, biology, and psychology, often emphasizing intrinsic capacity for motion or change, which makes sense considering its etymology, emphasizing an intrinsic ability rather than external force. **Motivity** uniquely captures an inherent "motive power" or self-initiating force for change, aligning with a data model's bidirectional synchronization as an active, propulsive property rather than passive reactivity (which implies response) or linkage (structural connection).

Similar niche terms like "affordance" in <span title="Human Computer Interaction">HCI</span> gained traction despite initial obscurity and today offer rich, nuanced meanings. 

## Levels

**Motivity** is partitioned into six ordinal levels that describe how a technology interacts with its environment. The framework is built around three primitives: **perception** (one-way input binding), **projection** (one-way output binding), and **comprehension** (internal state and computation).

> [!NOTE]
> Throughout this document, "comprehension" refers to a technology's ability to receive, store, and act on data — not to general intelligence, consciousness, or AGI. A hash function, a state machine, and a lookup table all exhibit comprehension in this sense: they maintain internal state and their behavior depends on it.

| Level | Name | Perception | Comprehension | Projection | Stateless/Stateful | Examples |
|-------|------|------------|--------------|------------|--------------------|---------|
| 0 | **Inert** | ✗ | ✗ | ✗ | — | A hammer or simple pulley system |
| 1 | **Computational** | ✗ | ✓ | ✗ | Stateful (internal) | Digital model[^twin], simulation |
| 2 | **Perceptive** | ✓ | ✗ | ✗ | Stateless | Data shadow[^twin], sensor log |
| 3 | **Projective** | ✗ | ✗ | ✓ | Stateless | Cron job that emails status |
| 4 | **Reactive** | ✓ | ✗ | ✓ | Stateless | Smart light switch, relay |
| 5 | **Adaptive** | ✓ | ✓ | ✓ | Stateful | Digital twin[^twin], autonomous agent |

### Inert (Level 0)
Technology at this level has no computational capability and no interaction with the environment. It is purely passive. Examples include a hammer, a simple pulley system, or any purely physical tool with no embedded intelligence.

```pikchr
scale = 0.7
circle "Self" fill white
```

### Computational (Level 1)
Technology at this level performs self-contained computation but has no input from or output to the environment. Comprehension (internal processing) is present, but neither perception nor projection are. Examples include a digital model or simulation that runs in isolation.

```pikchr
scale = 0.7
circle "Self" fill white
arrow thick <-> from last circle.s down 2cm "    Comprehension" ljust color 0x00662C
circle "Self" fill white
```

### Perceptive (Level 2)
Technology at this level receives input from the environment (one-way input binding) but produces no output and has no internal comprehension. The technology observes but does not process or act. Examples include a data shadow or a sensor that logs readings to storage without analyzing them.

```pikchr
scale = 0.7
E1: circle "Env" fill white width 0.5
    move 2cm
S1: circle "Self" fill white
    arrow thick from E1.e to S1.w "Perception" above "" above color 0x00662C
```

### Projective (Level 3)
Technology at this level produces output to the environment (one-way output binding) but receives no input and has no internal comprehension. Examples include a cron job that emails a status report — it projects information without sensing or reasoning about its environment.

```pikchr
scale = 0.7
S1: circle "Self" fill white
    move 2cm
E1: circle "Env" fill white width 0.5
    arrow thick from S1.e right 2cm "Projection" above "" above color 0x00662C
```

### Reactive (Level 4)
Technology at this level has two-way binding with the environment (perception and projection) but is stateless — it responds to input with output in a direct stimulus-response fashion with no internal model or memory of past interactions. Comprehension is absent. Examples include a smart light switch that turns lights on when motion is detected, or a relay that closes a circuit when a threshold is crossed.

```pikchr
scale = 0.7
E1: circle "Env" fill white
    arrow thick -> from E1.e right 2cm "Perception" above "" above color 0x00662C
S1: circle "Self" fill white
    arrow thick -> from S1.e right 2cm "Projection" above "" above color 0x00662C
E2: circle "Env" fill white
```

### Adaptive (Level 5)
Technology at this level has full two-way binding (perception and projection) with internal comprehension — it maintains state, builds an internal model of its environment, and changes its behavior based on accumulated experience. Examples include a digital twin that continuously synchronizes with a physical system and predicts future states, or an autonomous agent that plans and adapts.

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
