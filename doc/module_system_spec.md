# NEPLg2 Module System Specification v1.0

This document defines the NEPLg2 module system, emphasizing the orthogonality between physical files and logical modules.

## 1. Core Concepts

NEPLg2 distinguishes between three layers of organization:

1.  **Physical Layer (Files)**: Actual `.nepl` files on disk. Attributes: Path, Hash, Spans.
2.  **Syntax Layer (AST)**: Parsed content containing `merge`, `use`, and `module` blocks.
3.  **Logical Layer (Modules)**: The actual namespace tree used for name resolution and visibility.

## 2. Module Definition and Composition

### 2.1 Intra-file Modules
A file can contain nested modules using the `module <name>:` syntax with indentation.
```nepl
module parser:
    fn parse ...:
        ...
```

### 2.2 Inter-file Composition (`merge`)
`merge` is **Source Part Composition**. It merges another file into the *current module scope*.
- **Not** a simple string include.
- The operand is a **file path string**, not an identifier: `merge "./impl.nepl";`
- Merged files share the same logical module and `private` visibility.
- Partials are combined into a declaration multiset before resolution.

```nepl
#module
#indent 4

merge "./editor_ops.nepl";
merge "./editor_util.nepl";

pub fn run ...:
    ...
```

### 2.3 Module Dependencies (`use`)
`use` introduces a **fully-qualified identifier alias** into the current scope.

- The path uses `::` as separator (not `/` or `""`).
- The alias introduced is the **last segment** of the path.
- `use` acts as a visibility and cache boundary.

```nepl
use std::streamio;         // introduces `streamio` as alias for std::streamio
use core::math;            // introduces `math` as alias for core::math
use core::math::gcd;       // introduces `gcd` as alias for core::math::gcd
```

#### Glob import (`*`)
`*` may only be used when the target is a **module**. It imports all public items of that module directly into the current scope.

```nepl
use std::streamio::*;      // OK: streamio is a module
use core::math::*;         // OK: math is a module
```

`*` is **not valid** for non-module targets (functions, types, etc.):

```nepl
use core::math::gcd::*;    // ERROR: gcd is a function, not a module
```

## 3. Anchor Parts and Canonical Paths

Every logical module has one **Anchor Part** (the primary file).

### 3.1 Canonical Path Strategy
The **Canonical Module Path** is determined by:
`[Anchor File Path] + [Nested Module Path Segments]`

Module path segments are joined with `::`.

Example:
- `./editor.nepl` (Anchor)
- `module parser:` (Nested)
- Canonical Path: `./editor::parser`

### 3.2 Resolution Rules
1.  **Explicit Module Path**: Use the identifier defined in `#module`.
2.  **Anchor Path**: The path of the primary file.
3.  **Ambiguity Error**: If a `use` path refers to multiple distinct modules, it is a compile-time error.
4.  **Non-canonical Warning**: If `use` points to a non-anchor part (e.g., a merged file), the compiler warns and normalizes to the canonical anchor path.

## 4. Name Resolution

1.  **Module Resolution**: Purely path-based. Modules are **not** overloaded. `use` must resolve to exactly one module; zero matches → unresolved error, two or more → ambiguous module error.
2.  **Item Resolution**: Context-aware. Items (functions, traits) **can** be overloaded if unambiguous.

## 5. Visibility

- `private`: Visible within the entire logical module (including merged parts).
- `pub`: Visible to modules that `use` this module.
- `fileprivate`: **Avoided** to maintain file-module orthogonality.
