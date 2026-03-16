# NEPL Reboot - Design and Implementation Plan (2026-02-03)

> **[廃止済み / Superseded]**
> このドキュメントは 2026-02-03 時点の旧設計案であり、現在の仕様とは異なる。
> - `#import` は廃止。現在は `use` (モジュール依存) と `merge` (ソース結合) を使う。
> - `fn`/`struct`/`enum`/`trait`/`impl` キーワードは廃止。現在は `let` で統一。
> - 型記法は `%TypeExpr` 前置記法に変更済み。
> 現行仕様は `doc/module_system_spec.md` / `doc/type_notation_spec.md` を参照。

This document is a clean-slate specification and implementation plan for a
backward-incompatible reboot of NEPL. It is based on a review of the current
repository (nepl-core, nepl-cli, stdlib, examples) and highlights the gaps that
prevent a robust module system, name resolution, and long-term maintenance.

## 1. Goals
- Preserve prefix + offside-rule surface syntax but remove legacy shortcuts that
  undermine soundness.
- Make modules first-class: explicit package graph, explicit imports, re-export,
  deterministic name resolution, and prelude control.
- Move all later phases (typecheck, monomorphize, codegen) to DefId-based
  references instead of string names.
- Define a stable WASM ABI and keep the runtime boundary in stdlib, not in the
  compiler.
- Keep CLI behavior explicit: target/profile must be declared, no heuristics.

## 2. Current implementation snapshot (observed 2026-02-03)
- Language surface (nepl-core/src/lexer.rs, parser.rs):
  - Prefix expressions with colon blocks, `;` stack guards, pipe (`|>`) injection.
  - Constructs: `fn`, `struct`, `enum`, `match`, `trait`/`impl` (parsed), type
    params `.T`, effects (`->` pure, `*>` impure), references/boxes/tuples.
  - Directives: `#entry`, `#indent`, `#target`, `#import`, `#include`, `#use`,
    `#prelude`, `#no_prelude`, `#if[target=...]`, `#if[profile=...]`, `#extern`.
  - Import clauses are parsed from a directive string, not tokenized grammar.
- Modules/imports:
  - loader.rs flattens modules by splicing items/directives into the parent AST.
  - Paths are resolved to filesystem locations (stdlib root + relative paths).
  - `#use` is parsed but not used in name resolution.
- Name resolution:
  - name_resolve.rs is a stub; typecheck resolves via a lexical Env after
    flattening.
  - module_graph.rs and resolve.rs contain Phase2/3/4 scaffolding (graph,
    export table, import scope), but are not integrated into the pipeline.
- Type inference and checking (typecheck.rs, types.rs, monomorphize.rs):
  - Stack-driven reduction with unification; generics via `.T` params.
  - Monomorphize functions only, using mangled strings.
  - Move check and drop insertion run on HIR with copy-ness from TypeCtx.
- WASM backend (codegen_wasm.rs):
  - Emits exported memory, `main` and `_start`, string layout `[len][bytes]` with
    an allocator header, validates via wasmparser.
- CLI and stdlib boundary (nepl-cli/src/main.rs, stdlib/*):
  - CLI auto-upgrades target to `wasi` if `std/stdio` is imported.
  - Stdlib uses `#extern` WASI for IO and inline wasm for memory operations.

## 3. Language surface to keep
- Prefix expressions, colon blocks, and `;` stack guards remain core syntax.
- `fn`, `struct`, `enum`, `match`, `trait`/`impl` syntax stay, but traits can be
  staged in later phases.
- Effects `->` / `*>` and type params `.T` are preserved.
- `#wasm` and `#intrinsic` remain available, but only after name resolution and
  type/effect checks succeed.

## 4. Module system (new spec)

### 4.1 Module identity and manifests
- Each module is identified by `(package, module_path)`.
- A package is defined by a `nepl.toml` at its root:
  - `[package] name = "..."`
  - `[deps] std = "/path/to/stdlib", kp = "/path/to/kp"`
  - Optional `[prelude]` list of module specs used as defaults.
- Module spec rules:
  - "std/math" -> package `std`, module path `math`.
  - "./ui/layout" -> relative to the importing file inside the same package.
  - "../shared/log" -> relative path inside the same package.

### 4.2 Import grammar
```
#import "<module_spec>" [as <alias>]
#import "<module_spec>" as *
#import "<module_spec>" as { a, b as c, ns::* }
#import "<module_spec>" as @merge
pub #import "<module_spec>" <clause>

#prelude "<module_spec>"
#no_prelude
```

- Omitted clause means `as <last_segment>`.
- `as { .. }` supports:
  - `name`
  - `name as alias`
  - `ns::*` (open a namespace exported by the module)
- `pub #import` re-exports imported names (subject to the clause).

### 4.3 Visibility and exports
- Definitions are private by default. `pub` is allowed on `fn/struct/enum/trait`
  and on `#import`.
- Re-export collects public names from the imported module (or only the selected
  names if the clause is selective).
- Duplicate export names are an error.

### 4.4 Merge import (`as @merge`)
- Purpose: split a single module across files without losing "same module"
  semantics.
- Semantics:
  - The imported module is still a node in the dependency graph.
  - During definition collection, items from `@merge` modules are merged into the
    importing module's item list.
  - All diagnostics keep original file spans.
  - Duplicate definitions inside a merged module are errors.

### 4.5 Conditional imports
- `#if[target=...]` and `#if[profile=...]` are evaluated when building the module
  graph. Disabled branches do not participate in the graph or cycle detection.

## 5. Name resolution (new spec)
- Scopes in priority order:
  1) local bindings (function params, let bindings, pattern binds)
  2) module/namespace items
  3) selective imports (`as { name }`)
  4) open imports (`as *`)
  5) prelude modules (unless `#no_prelude` is set)
- Path resolution:
  - `alias::name` resolves via import alias.
  - `crate::...` resolves to the current package root.
  - `pkg::...` resolves to a dependency package if that package is listed in
    `[deps]`.
- Ambiguity:
  - If more than one candidate exists at the same priority tier, emit a
    deterministic error with an import stack.

## 6. Type system and monomorphization
- AST is lowered to HIR with DefId references and explicit TypeId nodes.
- Type annotations are ascriptions, not identity functions.
- Effects are part of function types and enforced at call sites.
- Generics:
  - Type params are `.T` only.
  - Instantiated on use; unify with call arguments.
  - Monomorphize functions and ADTs based on DefId + concrete TypeId list.
- Move check and drop insertion operate on resolved HIR/MIR, not on strings.

## 7. WASM ABI and runtime boundary
- ABI decisions are centralized and documented:
  - Memory layout reserves a fixed header for allocator metadata.
  - `str` layout: `[len:i32][bytes...]`.
  - `enum` layout: `[tag:i32][payload...]`.
  - `struct` layout: field order as declared.
- The compiler does not auto-inject allocator imports; stdlib provides `alloc`
  and related helpers.
- `#wasm` and `#intrinsic` require resolved types/effects and are rejected
  otherwise.

## 8. CLI and stdlib boundary
- CLI loads `nepl.toml` and resolves dependencies through the module graph.
- Target and profile must be explicit (CLI or directive). If both are provided
  and disagree, emit a diagnostic.
- Stdlib is restructured into packages with explicit preludes:
  - `std/prelude_base`: core types and math
  - `std/prelude_wasi`: prelude_base + stdio
- `nepl test` uses manifest information and runs stdlib tests without relying on
  hard-coded paths.

## 9. Implementation roadmap
1) Syntax and AST reset
   - Tokenize directives instead of parsing directive strings.
   - Introduce visibility and import clauses in AST.
   - Tests: parse fixtures for all import forms and prelude directives.
2) Module graph + manifest
   - Implement `nepl.toml` loader and dependency resolver.
   - Build module graph with cycle detection and import stacks.
   - Tests: cycle detection, stdlib vs relative resolution, conditional pruning.
3) Definition collection and export table
   - Build per-module export tables with visibility and re-exports.
   - Detect duplicate exports and merge conflicts.
   - Tests: re-export chains, duplicate names, merge conflicts.
4) Name resolution
   - Resolve identifiers to DefId; implement scope and prelude priority.
   - Lower AST to HIR with resolved IDs.
   - Tests: alias resolution, open/selective ambiguity, prelude opt-out.
5) Typecheck, monomorphize, MIR
   - Port stack-based typing onto HIR; enforce effects and annotations.
   - Implement generic instantiation and monomorphization for ADTs + functions.
   - Run move check/drop insertion on MIR.
   - Tests: generics, effect violations, move errors, match exhaustiveness.
6) WASM backend
   - Codegen from MIR; formalize ABI and runtime symbols.
   - Remove host shims; validate with wasmparser.
7) CLI/stdlib/test tooling
   - Switch CLI to manifest-based loading and explicit targets.
   - Restructure stdlib into prelude modules; update stdlib tests.
   - Web runner uses provider-based loader.
8) Documentation and migration
   - Update README/cli/runtime docs; add "breaking migration" notes.

## 10. Test strategy
- Unit tests per crate: syntax, module graph, resolver, typecheck.
- Integration tests: compile + run sample programs for wasm and wasi targets.
- Diagnostics tests: ambiguity, missing exports, cycle detection, effect errors.
- Stdlib tests: run via `nepl test` in both debug and release profiles.
- ABI tests: validate wasm layout expectations using golden wasm dumps.

## 11. Open questions
- Trait/impl resolution rules and coherence are not yet specified.
- The exact MIR shape and drop model should be locked before codegen work.
- Decide whether namespaces are explicit (`namespace X:`) or inferred by module
  nesting.
