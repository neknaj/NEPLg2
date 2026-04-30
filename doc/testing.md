# Testing and doctest workflow

> **対象実装**: このドキュメントは NEPLg2.0（現行 `nepl-core`）のテストワークフローを記述する。
> NEPLg3 では `tests-g3/` を新設して並行開発し、Stage 6 で `tests/` と切り替える計画。
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

`nodesrc/tests.js` uses a 60 second default timeout per doctest case. This is
intended to cover heavier self-host parser / stdlib cases while still catching
real hangs. Override it with `NEPL_TEST_CASE_TIMEOUT_MS` when a local run needs a
stricter or looser bound:

```bash
NEPL_TEST_CASE_TIMEOUT_MS=20000 node nodesrc/tests.js -i tests/stdlib --no-tree -o /tmp/tests.json -j 4
```

Run only the executable examples and inspect the aggregated JSON:

```bash
node nodesrc/tests.js -i examples --no-tree -o /tmp/examples-tests.json -j 8
```

Verify that tracked text files are UTF-8 without BOM:

```bash
node nodesrc/check_utf8.js
node nodesrc/check_utf8.js tutorials/getting_started
```

Run source policy regressions for stdlib implementation style:

```bash
node nodesrc/run_source_policy_regressions.js
```

CI runs the same source-policy set with `--warn-only`. A source-policy drift is
reported as a GitHub warning and in the step summary, but it does not prevent
compile, doctest, LLVM, or Pages deployment jobs from running:

```bash
node nodesrc/run_source_policy_regressions.js --warn-only
```

Individual policy checks can still be run directly when investigating a
specific warning:

```bash
node nodesrc/test_stdlib_match_decision_trees.js
node nodesrc/test_stdlib_sha256_no_unsafe_unwraps.js
node nodesrc/test_run_test_wasi_tmp_dir.js
node nodesrc/test_run_test_wasix_missing_wasmer_fallback.js
node nodesrc/test_stdlib_sort_merge_no_unsafe_unwraps.js
node nodesrc/test_stdlib_btree_insert_no_unsafe_grow_unwraps.js
node nodesrc/test_stdlib_bytebuf_utf8_boundary.js
node nodesrc/test_stdlib_fs_no_unsafe_unwraps.js
node nodesrc/test_stdlib_cliarg_no_unsafe_unwraps.js
node nodesrc/test_stdlib_streamio_no_unsafe_unwraps.js
node nodesrc/test_stdlib_no_unsafe_helpers.js
node nodesrc/test_stdlib_cast_doc_no_boilerplate.js
node nodesrc/test_stdlib_json_doc_no_boilerplate.js
node nodesrc/test_stdlib_nm_parser_doc_no_boilerplate.js
node nodesrc/test_stdlib_string_doc_no_boilerplate.js
node nodesrc/test_stdlib_nm_parser_no_inline_unwraps.js
node nodesrc/test_stdlib_nm_parser_no_block_unwraps.js
node nodesrc/test_stdlib_diag_error_no_unsafe_unwraps.js
node nodesrc/test_stdlib_std_test_no_unsafe_unwraps.js
node nodesrc/test_stdlib_kpgraph_no_unsafe_unwraps.js
node nodesrc/test_stdlib_wasix_tui_no_unsafe_unwraps.js
node nodesrc/test_stdlib_binary_heap_no_unsafe_unwraps.js
node nodesrc/test_stdlib_queue_deque_no_unsafe_unwraps.js
node nodesrc/test_stdlib_bitset_no_unsafe_unwraps.js
node nodesrc/test_stdlib_bitset_update_error_owner.js
node nodesrc/test_stdlib_adjacency_matrix_no_unsafe_unwraps.js
node nodesrc/test_stdlib_adjacency_matrix_update_error_owner.js
node nodesrc/test_stdlib_bloom_filter_no_unsafe_unwraps.js
node nodesrc/test_stdlib_counting_bloom_filter_no_unsafe_unwraps.js
node nodesrc/test_stdlib_fenwick_no_unsafe_unwraps.js
node nodesrc/test_stdlib_fenwick_add_error_owner.js
node nodesrc/test_stdlib_ringbuffer_no_unsafe_unwraps.js
node nodesrc/test_stdlib_disjoint_set_no_unsafe_unwraps.js
node nodesrc/test_stdlib_disjoint_set_borrowed_observers.js
node nodesrc/test_stdlib_disjoint_set_union_error_owner.js
node nodesrc/test_stdlib_list_no_unsafe_unwraps.js
node nodesrc/test_stdlib_sparse_set_no_unsafe_unwraps.js
node nodesrc/test_stdlib_segment_tree_no_unsafe_unwraps.js
node nodesrc/test_stdlib_segment_tree_borrowed_observers.js
node nodesrc/test_stdlib_segment_tree_update_error_owner.js
node nodesrc/test_stdlib_stack_no_unsafe_unwraps.js
node nodesrc/test_stdlib_hashmap_storage_contract.js
node nodesrc/test_stdlib_hashset_storage_contract.js
node nodesrc/test_stdlib_vec_no_unsafe_unwraps.js
node nodesrc/test_stdlib_byte_scanner_helpers_boundary.js
node nodesrc/test_selfhost_cli_args_no_owner_field_reads.js
node nodesrc/test_selfhost_cli_args_types_split.js
node nodesrc/test_selfhost_string_helpers_boundary.js
node nodesrc/test_selfhost_cli_driver_boundary.js
node nodesrc/test_selfhost_cli_file_io_boundary.js
node nodesrc/test_selfhost_cli_reporter_boundary.js
node nodesrc/test_selfhost_outcome_no_raw_result_cell.js
node nodesrc/test_llvm_runner_return_value.js
node nodesrc/test_stdlib_builder_owner_boundary.js
node nodesrc/test_stdlib_io_bytebuf_owner_boundary.js
node nodesrc/test_stdlib_string_no_unsafe_unwraps.js
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

Run playground editor CLI snapshots after the web TypeScript bundle is updated:

```bash
npm --prefix web run build:ts
node nodesrc/cli.js -i tests/playground_editor --playground-editor-tests -o json=/tmp/playground-editor-tests.json
```

Run the worker-backed terminal regression after shell/runtime changes:

```bash
node nodesrc/playground_shell_worker_test_runner.js
```

## Where tests live

### `tests/compiler/*.n.md`

Compiler-facing regression cases.

- parse / name resolution / typecheck / diagnostics
- `compile_fail` cases with `diag_code`
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

Tutorial documents should also stay UTF-8 without BOM so ruby annotations and
Japanese text render consistently across the CLI, generated HTML, and the web
playground.

## Runtime model used by `nodesrc`

`nodesrc/run_test.js` chooses the execution path from `#target`:

- `#target wasi` / `wasip1`-style cases run through Node's WASI support
- `#target wasix` cases first try `wasmer run`; when `wasmer` is not installed, or when Wasmer lacks the `wasix_32v1.tty_get` / `tty_set` imports, the runner falls back to Node's WASI support with minimal WASIX TTY host imports

This matters for features such as TUI, which require WASIX imports and cannot be
executed by the preview1-only Node WASI runtime without those fallback imports.
The runner prepares a `tmp/` scratch directory inside each preopen root before
execution. File-write doctests can use `tmp/...` paths without depending on an
untracked repository directory being present in a clean checkout.

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
- `diag_code:`

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

## Playground editor CLI tests

`tests/playground_editor/` is reserved for browser-free editor verification.

- Each case lives under its own directory.
- The standard fixture shape is `source.nepl`, `commands.json`, and `expected.json`.
- Analysis-model cases use `source.nepl`, `analysis.json`, `requests.json`, and `expected.json`.
- `commands.json` may contain pure editor commands plus `keyboard_event` steps for shortcut coverage.
- Cursor/navigation cases should prefer pure commands such as `move_cursor`, `move_cursor_vertical`, `move_cursor_line_boundary`, and `move_cursor_page` so they stay runnable without DOM events.
- `requests.json` can verify `update_payload`, `token_insight`, `hover`, `definition`, and `occurrences` without starting a browser.
- The formal entrypoint is `nodesrc/cli.js --playground-editor-tests`, which writes an aggregate JSON summary.
- `nodesrc/playground_editor_test_runner.js` remains the lower-level runner used by the CLI path.
- Current coverage includes shortcut/history cases, text insert/delete, left-right movement, up-down movement, Home/End, PageUp/PageDown, and analysis snapshots.
- Terminal/process regressions are covered separately by `nodesrc/playground_shell_worker_test_runner.js`, which verifies that compile/run requests stay on the worker side and that compile outputs/stdout are routed back correctly.

## Current guidance

- Prefer `tests/stdlib/*.n.md` for new reboot regressions.
- Prefer `stdlib/**/*.nepl` doctests for short examples attached to one API.
- Use `run_doctest.js` for the fastest reproduction loop.
- Use `nodesrc/tui_regression.js` for TUI end-to-end checks after WASIX-facing changes.
- Keep docs, doctests, and implementation in sync in the same change.
