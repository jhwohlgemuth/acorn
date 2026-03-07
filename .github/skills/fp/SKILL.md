---
name: fp
description: Functional programming (FP) principles for use whenever code is involved. FP emphasizes immutability, composability, and purity to create clearer, more predictable code. This skill should be applied whenever writing or modifying code, especially when it can lead to simpler and more maintainable solutions. *use this skill whenever code is involved*
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

## Unique applications to Rust
- Avoid adding `?` to the code — always handle errors properly. This promotes better error handling and prevents silent failures.
- Import modules at the top of the file, not inside functions. This keeps dependencies clear and avoids unnecessary imports.
- Avoid using `mut` unless necessary. Emphasize immutability to enhance code safety and readability.
- Use iterators and functional combinators (like `map`, `filter`, `fold`) instead of loops and mutable state. This promotes a more declarative style of programming.
- Leverage Rust's powerful type system to create abstractions that promote immutability and composability, such as using `Option` and `Result` types for error handling instead of mutable state.
- Always prefer `core` over `std` when possible, as it is more lightweight and promotes a functional programming style by avoiding unnecessary dependencies and side effects.