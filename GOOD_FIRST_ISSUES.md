# Functional Programming Improvements for ACORN

This document catalogs opportunities to improve the ACORN codebase by applying functional programming principles: immutability, composability, and purity.

## Guiding Principles

- Minimize moving parts. Avoid mutable state and side effects.
- Favor pure functions that take inputs and return outputs without modifying external state.
- Use immutable data structures and avoid shared mutable state.
- Minimize use of `return` statements; structure code to flow naturally.

---

## Priority 1: Easy Refactoring (Good for New Contributors)

### 1.1 Remove Debug `println!` Statements

**Locations:**
- `acorn-lib/src/schema/pid/raid.rs:986-987` - Debug print in serialization
- `acorn-lib/src/schema/research_activity/mod.rs:525-526` - Debug JSON output

**Change:** Replace with proper logging (`tracing::debug!`/`info!`) or remove entirely.

---

### 1.2 Replace Nested `for` Loops with Iterator Chains

**Locations:**
- `acorn-lib/src/schema/mod.rs:748-759` - Nested `for_each` in `to_graph()` method

**Current pattern:**
```rust
organization.member.iter().for_each(|directorate| {
    let a = graph.add_node(directorate.name.clone());
    graph.add_edge(root, a, 0);
    directorate.member.iter().for_each(|division| {
        let b = graph.add_node(division.name.clone());
        graph.add_edge(a, b, 0);
        division.member.iter().for_each(|group| {
            // ...
        });
    });
});
```

**Suggested refactor:**
```rust
use petgraph::graph::NodeIndex;
use std::collections::HashMap;

let mut indices: HashMap<_, NodeIndex<u32>> = HashMap::new();
indices.insert(organization.name.clone(), root);

organization.member.iter().flat_map(|d| {
    std::iter::once(d).chain(d.member.iter().flat_map(|div| {
        std::iter::once(div).chain(div.member.iter())
    }))
}).for_each(|org| {
    let parent = // find parent index
    let idx = graph.add_node(org.name.clone());
    if let Some(p) = parent {
        graph.add_edge(p, idx, 0);
    }
    indices.insert(org.name.clone(), idx);
});
```

---

### 1.3 Replace `.push()` in Loops with Immutable Collection Building

**Locations:**
- `acorn-lib/src/io/api/mod.rs:679` - Template key extraction loop

**Change:** Use `.iter().flat_map().collect()` or iterator chains instead when possible.

---

### 1.4 Use `Option::map_or` Instead of Match

**Change:** Replace patterns like:
```rust
match x {
    Some(y) => ...,
    None => default,
}
```

With:
```rust
x.map_or(default, |y| ...)
```

---

### 1.5 Refactor Match Expressions to Option Methods

**Locations:**
- `acorn-lib/src/schema/mod.rs:712-740` - Nested match blocks in `nearest()` method

**Change:** Use `and_then()`, `map()`, or `map_or()` to flatten nested matches.

---

## Priority 2: Moderate Complexity

### 2.1 Eliminate Unnecessary Mutable Variables in Loops

**Locations:**
- `acorn-lib/src/io/mod.rs:366` - `let mut success = true` mutated in for loop
- `acorn-lib/src/schema/research_activity/mod.rs:415,425,550,562,636,652`

**Current:**
```rust
let mut success = true;
for index in 0..archive.len() {
    // ...
    if error_occurred {
        success = false;
    }
}
```

**Refactor to:**
```rust
let success = (0..archive.len()).all(|index| {
    // returns true if all succeed, false otherwise
});
```

---

### 2.2 Replace Mutable File Handles

**Locations:**
- `acorn-lib/src/analyzer/mod.rs:653`
- `acorn-lib/src/io/mod.rs:345,919-920,957-958`
- `acorn-cli/src/commands/link/mod.rs:53`
- `acorn-cli/src/commands/format/mod.rs:65`

**Change:** Consider using atomic file writing crates like `tempfile` or `atomicwrites`.

---

### 2.3 Refactor Early Returns to Match Expressions

**Locations:**
- `acorn-lib/src/io/mod.rs:950`
- `acorn-lib/src/io/api/mod.rs:489,653,658`

**Change:** Restructure to use `?` operator or match arms that return values naturally.

---

## Priority 3: Advanced

### 3.1 Extract Pure Functions from Validation Logic

**Location:** `acorn-lib/src/schema/research_activity/mod.rs:818-936`

Large validation block with many `let mut err = ValidationError::new()`.

**Change:** Extract into pure functions returning `Result` or `Vec<ValidationError>`.

---

### 3.2 Replace `.clone() + Mutation` with Functional Updates

**Locations:**
- `acorn-lib/src/schema/research_activity/mod.rs:415` - `let mut clone = self.clone().copy();`
- `acorn-lib/src/schema/research_activity/aspect.rs:290,298`

**Change:** Use builder pattern or `Struct.update()` pattern.

---

## Summary Table

| # | Location | Type | Difficulty |
|---|----------|------|------------|
| 1.1 | schema/pid/raid.rs, schema/research_activity/mod.rs | Remove println! | Easy |
| 1.2 | schema/mod.rs:748-759 | Iterator chains | Easy |
| 1.3 | io/api/mod.rs:679 | Remove .push() | Easy |
| 1.4 | Throughout | Use map_or | Easy |
| 1.5 | schema/mod.rs:712-740 | Flatten matches | Easy |
| 2.1 | io/mod.rs:366, schema/* | Remove let mut | Moderate |
| 2.2 | io/mod.rs, commands/* | File operations | Moderate |
| 2.3 | io/mod.rs, io/api/mod.rs | Early returns | Moderate |
| 3.1 | schema/research_activity/mod.rs | Extract pure fn | Advanced |
| 3.2 | schema/* | Clone+mute pattern | Advanced |
