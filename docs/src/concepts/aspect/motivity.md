# 🦾 Motivity

## Discussion
"Motivity" describes a technology's ability to exert power over its environment. In the context of **ASPECT**, this involves how a given technology interacts with its environment and the degree of autonomy it possesses in performing tasks. As a model of interaction, motivity is built on [task classification](./task-classification.md) categories and is closely related to the [human-machine teaming level](./human-machine-teaming.md). Motivity can also be viewed in terms of data binding - no binding, one-way binding, and two-way binding.

Motivity is a somewhat uncommon word. It was chosen to for the ASPECT framework in part because deliberate ambiguity can foster conceptual depth. Rarity minimizes external preconceptions, enabling custom layering of meanings without the baggage of a widely used term.

Motivity has been defined in various contexts across philosophy, biology, and psychology, often emphasizing intrinsic capacity for motion or change, which makes sense considering its etymology, emphasizing an intrinsic ability rather than external force. Motivity uniquely captures an inherent "motive power" or self-initiating force for change, aligning with a data model's bidirectional synchronization as an active, propulsive property rather than passive reactivity (which implies response) or linkage (structural connection).

Similar niche terms like "affordance" in <span title="Human Computer Interaction">HCI</span> gained traction despite initial obscurity and today offer rich, nuanced meanings.

## Types
Motivity is logically partitioned into four distinct types based on the level of interaction with the environment. Motivity is built around the same tasks in [task classification](./task-classification.md) - perception, projection, and comprehension, which essentially act as proxies for one-way data binding (input), one-way data binding (output), and computation, respectively.

### Type 0
> [!NOTE]
> *Example* Digital model

Type 0 has no interaction with the environment. Type 0 agency can involve "comprehension" (e.g., simulations) but also includes technology that does not (e.g., a hammer). Although comprehension is optional in every type, including Type 0, in the case of Type 0 technologies it is not a very interesting thing to consider.

```pikchr
scale = 0.7
circle "Env" fill white width 0.5
move 2cm
circle "Self" fill white
arrow dashed thick <-> from last circle.s down 2cm "    Comprehension" ljust color 0x00662C
circle "Self" fill white
```

### Type 1A
> [!NOTE]
> *Example* Data shadow[^twin]

Type 1A is argubly the most common form of technology, at least in the context of research. Although it is mildly ambiguous in interpration, the use of an interactive notebook (e.g., Jupyter) to visualize the distribution of some data read from a <span title="Comma Separated Values">CSV</span> file would be considered as Type 1A since the technology reads input from the environment (a CSV file) and presents an aspect of the data visually. Technically, the associated image is a created artifact stored with zeroes and ones, but for the purpose of classification via purpose of agency, we consider such ephemeral artifacts as insufficient to justify Types 1B or 2.

```pikchr
scale = 0.7
E1: circle "Env" fill white width 0.5
    move 2cm
S1: circle "Self" fill white
    arrow dashed thick <-> from S1.s down 2cm "    Comprehension" ljust color 0x00662C
S2: circle "Self" fill white
    arrow thick from E1.e to S1.w "Perception" above "" above color 0x00662C
```

### Type 1B
> [!NOTE]
> *Example* Cron job that emails status of long running calculation

```pikchr
scale = 0.7
S1: circle "Self" fill white
    arrow dashed thick <-> from S1.n up 2cm "Comprehension    " rjust color 0x00662C
S2: circle "Self" fill white
    arrow thick -> from S2.e right 2cm "Projection" above "" above color 0x00662C
E1: circle "Env" fill white
```

### Type 2
> [!NOTE]
> *Example* Digital twin[^twin]

```pikchr
scale = 0.7
E1: circle "Env" fill white
    arrow thick -> from E1.e right 2cm color 0x00662C "Perception" above "" above
S1: circle "Self" fill white
    arrow dashed thick <-> from S1.s down 2cm "Comprehension    " rjust color 0x00662C
S2: circle "Self" fill white
    arrow thick -> from S1.e right 2cm "Projection" above "" above color 0x00662c
E2: circle "Env" fill white
```

[^twin]: Y. K. Liu, S. K. Ong, and A. Y. C. Nee, "State-of-the-art survey on digital twin implementations," Adv. Manuf., vol. 10, no. 1, pp. 1-23, Mar. 2022, doi: 10.1007/s40436-021-00375-w.
