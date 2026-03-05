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
- `acorn-lib/src/analyzer/mod.rs:349` - Empty println in code
- `acorn-lib/src/schema/research_activity/mod.rs:522` - Debug JSON output
- `acorn-lib/src/schema/pid/raid.rs:985,995,1002` - Debug print statements
- `acorn-lib/src/lib.rs:325` - Rate limit print

**Change:** Replace with proper logging (`tracing::debug!`/`info!`) or remove entirely.

---

### 1.2 Replace Nested `for` Loops with Iterator Chains

**Locations:**
- `acorn-lib/src/schema/mod.rs:693-704` - Triple nested loop building member list
- `acorn-lib/src/schema/mod.rs:751-762` - Similar pattern in `to_graph()` method
- `acorn-lib/src/schema/mod.rs:920-930` - Duplicate of above

**Current pattern:**
```rust
let mut items = vec![organization.clone()];
for directorate in &directorates {
    items.push(directorate.clone());
    for division in &divisions {
        items.push(division.clone());
        // ...
    }
}
```

**Suggested refactor:**
```rust
std::iter::once(organization.clone())
    .chain(directorates.iter().cloned())
    .chain(divisions.iter().flat_map(|d| std::iter::once(d.clone()).chain(groups.iter().cloned())))
    .collect()
```

---

### 1.3 Replace `.push()` in Loops with Immutable Collection Building

**Locations:**
- `acorn-lib/src/schema/mod.rs:694,697,700,921,924`
- `acorn-lib/src/io/api/mod.rs:679`

**Change:** Use `.iter().flat_map().collect()` or iterator chains instead.

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

### 1.5 Consolidate Duplicate Organization Hierarchy Code

**Location:** `acorn-lib/src/schema/mod.rs`

The `members()` method at lines 689-704 and 916-930 are nearly identical.

**Change:** Extract to single pure function, reuse in both places.

---

## Priority 2: Moderate Complexity

### 2.1 Eliminate Unnecessary `let mut` in Iterator Transformations

**Locations:**
- `acorn-lib/src/analyzer/mod.rs:162` - `let mut issues = vec![]` then reassigned
- `acorn-lib/src/schema/research_activity/mod.rs:415,425,550,562,636,652`

**Current:**
```rust
let mut issues = vec![];
if !is_offline {
    issues = join_all(link_futures).await;
}
```

**Refactor to:**
```rust
let issues = if is_offline {
    vec![]
} else {
    join_all(link_futures).await
};
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
| 1.1 | analyzer/mod.rs, schema/*, lib.rs | Remove println! | Easy |
| 1.2 | schema/mod.rs | Iterator chains | Easy |
| 1.3 | schema/mod.rs, io/api/mod.rs | Remove .push() | Easy |
| 1.4 | Throughout | Use map_or | Easy |
| 1.5 | schema/mod.rs | Deduplicate | Easy |
| 2.1 | analyzer/mod.rs, schema/* | Remove let mut | Moderate |
| 2.2 | io/mod.rs, commands/* | File operations | Moderate |
| 2.3 | io/mod.rs, io/api/mod.rs | Early returns | Moderate |
| 3.1 | schema/research_activity/mod.rs | Extract pure fn | Advanced |
| 3.2 | schema/* | Clone+mute pattern | Advanced |
