# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

**NEPLg2** (Neknaj Expression Prefix Language General-purpose 2) is a prefix-notation, expression-oriented programming language that compiles to WebAssembly (WASM) and LLVM IR. The compiler is written in Rust. A self-hosting compiler (`/stdlib/neplg2/`) is under development.

### NEPLg2 と NEPLg2.1 の区別

| 名称 | 説明 |
|------|------|
| **NEPLg2** | 現行実装（`nepl-core/`、`stdlib/`、`tests/`）の言語仕様。`plan.md` が設計の起点。 |
| **NEPLg2.1** | 新仕様。型記法（`%fn ...`、juxtaposition、`unit`）・`fn` 宣言キーワード廃止・括弧完全廃止を含む大幅な変更。`doc/` 配下の新仕様ドキュメント群（`type_notation_spec.md`、`pattern_spec.md`、`module_system_spec.md`、`language_platform_spec.md`）が対象。 |

NEPLg2.1 の変更は NEPLg2 とは非互換であり、実装は別途移行計画に従って進める。
現行の `nepl-core/` は NEPLg2 の実装であり、NEPLg2.1 の実装は今後の作業になる。

## Build Commands

```bash
# Debug build
cargo build --workspace --locked

# Release build
cargo build --workspace --release --locked

# Compile a NEPL source file (outputs wasm by default)
cargo run -p nepl-cli -- -i examples/counter.nepl -o tmp/counter

# Emit multiple formats
cargo run -p nepl-cli -- -i examples/counter.nepl -o tmp/counter --emit wasm,wat,wat-min,llvm,llvm-min

# Build web playground
trunk build
trunk serve   # http://127.0.0.1:8080
```

## Testing

```bash
# Rust unit tests
cargo test --workspace --locked

# Fast integration tests (changed files only) — run after trunk build
trunk build
NO_COLOR=false node nodesrc/tests.js --changed --changed-base HEAD -o /tmp/tests-changed.json --runner wasm --no-tree -j 2

# Full integration test suite
trunk build
NO_COLOR=false node nodesrc/tests.js -i tests -i stdlib -o /tmp/tests-dual-full.json --runner all --llvm-all --assert-io --strict-dual --no-tree -j 2

# LLVM tests (requires clang 21.1.0)
NEPL_LLVM_CLANG_BIN=/path/to/clang node nodesrc/tests.js -i tests/llvm_target.n.md --runner llvm --llvm-compile-only -j 2
```

Always run `trunk build` before running `nodesrc/tests.js`. Confirm tests pass before committing.

## Architecture

### Workspace Crates

| Crate | Purpose |
|-------|---------|
| `nepl-core` | Core compiler library (`no_std`) — all compilation stages |
| `nepl-cli` | CLI binary — WASI runtime, file I/O, clang invocation for native targets |
| `nepl-lsp` | Language Server Protocol support (in development) |
| `web` | WebAssembly bindings for the web playground |

### Compilation Pipeline

```
Source (.nepl / .n.md)
  → Lexer (indent-aware tokenization)
  → Parser (prefix notation + off-side rules via `:` + indent)
  → Name Resolution
  → Type Checking (stack-based inference)
  → HIR
  → Passes (move_check, drop_insertion, codegen_precheck)
  → Monomorphization
  ├─→ CodeGen WASM → .wasm / .wat
  └─→ CodeGen LLVM → .ll → clang → native binary
```

Key files in `nepl-core/src/`: `compiler.rs` (orchestration), `lexer.rs`, `parser.rs`, `typecheck.rs`, `ast.rs`, `hir.rs`, `types.rs`, `codegen_wasm.rs`, `codegen_llvm.rs`, `monomorphize.rs`, `module_graph.rs`, `passes/`.

### ファイル形式：`.n.md` と `.md` の違い

`.n.md` は **NM 拡張 Markdown** であり、通常の `.md` とは異なる。仕様は `stdlib/nm/README.n.md` に定義されている。

| 形式 | 説明 |
|------|------|
| `.n.md` | NM 拡張 Markdown。フリガナ（ruby）`[漢字/かんじ]`、gloss `{日本語/English}`、Nest（見出しによる節の入れ子）が使える |
| `.md` | 通常の Markdown。フリガナ・gloss・Nest の拡張構文は使えない |

**フリガナの書き方（`.n.md` 専用）**：

- ruby（読み）: `[漢字/かんじ]` → 漢字の上に小さく「かんじ」が表示される
- gloss（対訳）: `{[日本語/にほんご]/English}` → 「日本語」の下に「English」が小さく表示される
- 多言語 gloss: `{A/b/β}` → 3つ以上の言語を重ねられる

`doc/` 配下の `.md` ファイルは通常の Markdown であるため、フリガナ構文を書いても正しく処理されない。フリガナが必要な文書は `.n.md` 拡張子を使うこと。

### Test Format (`.n.md`)

Tests are embedded in Markdown files as fenced code blocks. `nodesrc/` contains the test runner infrastructure:
- `tests.js` — test orchestrator
- `run_test.js` — individual test execution
- `run_doctest.js` — extracts and runs doctests from stdlib

Stdlib doctests use `//: ` comment markers within `.nepl` files.

### Standard Library (`/stdlib/`)

| Directory | Contents |
|-----------|---------|
| `core/` | Primitives: math, option, result, traits |
| `std/` | stdio, streamio, fs, io |
| `alloc/` | Collections: vec, hashmap |
| `platforms/` | Platform-specific (WASIX, TUI) |
| `neplg2/` | Self-hosting compiler (WASI CLI + pure WASM core) |

## Development Guidelines (from AGENTS.md)

- **Before starting work**: Check `plan.md` for what is planned. Do not modify `plan.md` — write notes about deviations in `note.n.md`.
- **After changes**: Update `note.n.md` with implementation status and differences from the plan. Update `todo.md` (completed items are deleted, not marked done). Update `README.md` and `/doc/` if the implementation changed.
- **Comments**: Write abstract explanations of purpose and logic, not descriptions of changes. Do not add "changed here" style comments — describe changes in prose outside the code. Update comments when the code they describe changes.
- **Scope**: Do not make unnecessary changes. Do not change coding style, indentation, or formatting in unrelated code. Do not remove existing features without explicit instruction.
- **Errors**: Identify and fix root causes, not symptoms.
- **Docs**: Keep `/doc/` consistent in style. Create new doc files there as needed.

### Commit Rules

- **Commit at every meaningful work boundary** — after completing a feature, fix, or doc change. Do not accumulate large batches of unrelated changes.
- **Always update `note.n.md` before committing** — record what was implemented, any deviations from plan, and current status. This is mandatory; never skip it.
- Commit message format: `type(scope): description` (e.g. `feat(parser): add table support`, `fix(html_gen): rewrite .md links`).

### Stdlib and Compiler Comments

Both `stdlib/` and the self-hosting compiler (`stdlib/neplg2/`) use **Japanese comments** written in the extended Markdown format supported by `stdlib/nm`. Each function should document: purpose, algorithm, constraints, complexity, and include inline doctests (`//: neplg2:test`).

### Self-Hosting Compiler Structure

Mirrors the Rust compiler's separation: the CLI (`/stdlib/neplg2/cli/`) uses WASI; the core (`/stdlib/neplg2/core/`) is pure WASM (no WASI), analogous to how `nepl-cli/` uses std while `nepl-core/` is `no_std`.

## Key Documentation Files

- `plan.md` — language/feature design specifications (do not modify)
- `note.n.md` — implementation progress and design notes
- `todo.md` — pending work items only
- `doc/` — detailed specs: memory safety, module system, LSP, LLVM setup, move semantics, dependent types
