# Testing and doctest workflow

> **対象実装**: このドキュメントは NEPLg2.0（現行 `nepl-core`）のテストワークフローを記述する。
> NEPLg2.1 では `tests-2.1/` を新設して並行開発し、Stage 6 で `tests/` と切り替える計画。
> 詳細は [migration/index.md](./migration/index.md) を参照。

This document describes the current NEPLg2 test workflow and where each kind of
test lives.

## Overview

The repository currently uses three main layers of verification:

- Rust-side compiler tests under `nepl-core/tests/`
- doctests and focused behavioral tests under `tests/`, `tutorials/`, and `stdlib/`
- end-to-end sample regressions such as `nodesrc/tui_regression.js`

The main day-to-day workflow for stdlib reboot work is based on `nodesrc/`.

## Recommended commands

Run focused doctests and test files with JSON output:

```bash
node nodesrc/tests.js -i tests/compiler -i tests/stdlib --no-tree -o /tmp/tests.json -j 15
```

Run one doctest directly:

```bash
node nodesrc/run_doctest.js -i tests/stdlib/streamio.n.md -n 3
node nodesrc/run_doctest.js -i stdlib/core/traits/deserialize.nepl -n 1
```

Generate HTML from `.n.md` / `.nepl` documents:

```bash
node nodesrc/cli.js -i stdlib/features/tui.nepl -i tests/stdlib/features_tui.n.md -o html=/tmp/doc-html
```

When Rust-side code changes, rebuild the web compiler bundle first:

```bash
NO_COLOR=false trunk build
```

## Where tests live

### `tests/compiler/*.n.md`

Compiler-facing regression cases.

- parse / name resolution / typecheck / diagnostics
- `compile_fail` cases with `diag_id`
- target-specific behavior such as LLVM / WASM / WASIX checks

### `tests/stdlib/*.n.md`

User-facing stdlib behavior and reboot migration regressions.

- facade behavior
- end-user API expectations
- focused reproductions for bugs found during reboot work

These are the main place to add a regression when an API contract breaks.

### `stdlib/tests/*.n.md`

Library-adjacent requirement and support tests that are still useful as
standalone fixtures.

These are kept when they are still meaningful, but new public-facing behavior
checks should usually go to `tests/stdlib/*.n.md`.

### `stdlib/**/*.nepl`

Doc comments may contain `neplg2:test` doctests.

Use these for:

- small usage examples
- contract examples that should stay close to the definition

Do not use doc comments for large regression matrices or heavy edge-case suites.
Those belong in `tests/`.

### `tutorials/**/*.n.md`

Tutorials also contain doctests.

These act as executable documentation and should reflect the current stdlib
layout and API style.

## Runtime model used by `nodesrc`

`nodesrc/run_test.js` chooses the execution path from `#target`:

- `#target wasi` / `wasip1`-style cases run through Node's WASI support
- `#target wasix` cases run through `wasmer run`

This matters for features such as TUI, which require WASIX imports and cannot be
executed by the preview1-only Node WASI runtime.

You can override the `wasmer` binary with:

```bash
WASMER_BIN=/path/to/wasmer node nodesrc/run_doctest.js -i tests/stdlib/features_tui.n.md -n 1
```

## Output expectations

`nodesrc/tests.js` and `run_doctest.js` both understand doctest metadata such as:

- `stdin:`
- `stdout:`
- `stderr:`
- `ret:`
- `diag_id:`

If a doctest includes `stdout:` or `stderr:`, `nodesrc/tests.js` checks those
expectations by default. `--assert-io` is optional and only makes the intent
explicit.

## `std/test`

`std/test` is the basic assertion module used in NEPL-side tests.

Typical helpers include:

- `assert`
- `assert_eq_i32`
- `assert_str_eq`
- `test_checked`
- `test_fail`

Use the smallest assertion that makes failures obvious.

## Current guidance

- Prefer `tests/stdlib/*.n.md` for new reboot regressions.
- Prefer `stdlib/**/*.nepl` doctests for short examples attached to one API.
- Use `run_doctest.js` for the fastest reproduction loop.
- Use `nodesrc/tui_regression.js` for TUI end-to-end checks after WASIX-facing changes.
- Keep docs, doctests, and implementation in sync in the same change.
