---
name: fp
description: Functional programming (FP) principles for use whenever code is involved. FP emphasizes immutability, composability, and purity to create clearer, more predictable code. This skill should be applied whenever writing or modifying code, especially when it can lead to simpler and more maintainable solutions.
license: MIT
---

## Always prefer functional programming style when possible

**Minimize moving parts. Avoid mutable state and side effects.**

- Prioritize the 3 T's of functional programming:
  1. Immutability
  2. Composability
  3. Purity
- Avoid mutable state and side effects.
- Minimize use of `return` statements and early exits. Instead, structure code to flow naturally from inputs to outputs.
- Favor pure functions that take inputs and return outputs without modifying external state.
- Use immutable data structures and avoid shared mutable state.
- This leads to clearer, more predictable code that is easier to test and debug.

OOP makes code understandable by encapsulating moving parts. Functional Programming makes code understandable by minimizing moving parts.