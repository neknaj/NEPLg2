---
id: ISS-20260429T083822053Z-SELF-HOST-DIAGNOSTICS-USE-STRING-COD-1040C21E
title: "self-host diagnostics use string codes instead of enum parity contract"
area: SELFHOST-DIAG
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-04-29
updated: 2026-04-29
target: "stdlib/neplg2/core/infra/diag.nepl, stdlib/neplg2/core/syntax/parser/module_parser.nepl, stdlib/neplg2/core/module, stdlib/neplg2/cli/reporter.nepl, tests/stdlib/selfhost_cli_reporter.n.md"
---

# ISS-20260429T083822053Z-SELF-HOST-DIAGNOSTICS-USE-STRING-COD-1040C21E: self-host diagnostics use string codes instead of enum parity contract

## 概要

doc/neplg2/compiler_diagnostics_redesign_plan.md Stage D5 requires self-host diagnostics to share the Rust diagnostic code contract, but SelfhostDiagnostic stores code as str and constructors accept arbitrary strings. Parser, loader, import, module graph, and CLI call sites build diagnostics with string literals, so typo or missing variant coverage cannot be caught by match exhaustiveness.

## 対象

- `stdlib/neplg2/core/infra/diag.nepl, stdlib/neplg2/core/syntax/parser/module_parser.nepl, stdlib/neplg2/core/module, stdlib/neplg2/cli/reporter.nepl, tests/stdlib/selfhost_cli_reporter.n.md`

## 根拠

- `doc/neplg2/compiler_diagnostics_redesign_plan.md` Stage D5 は、self-host compiler が Rust core と同じ diagnostic contract を使うことを要求している。
- `SelfhostDiagnostic` の `code` が `str` だったため、call site が `"selfhost.parser.*"` / `"selfhost.loader.*"` などの自由文字列を直接渡せた。
- lexer は `LexErrorCode` enum を持っていたが、parser diagnostic へ写す時点で文字列化していたため、self-host diagnostic value には typed code が残らなかった。

## 問題

doc/neplg2/compiler_diagnostics_redesign_plan.md Stage D5 requires self-host diagnostics to share the Rust diagnostic code contract, but SelfhostDiagnostic stores code as str and constructors accept arbitrary strings. Parser, loader, import, module graph, and CLI call sites build diagnostics with string literals, so typo or missing variant coverage cannot be caught by match exhaustiveness.

## 影響

Self-host diagnostic JSON may drift from the Rust compiler registry and future parser/resolver/checker parity tests can pass string snapshots while internal classification remains unchecked. This also conflicts with the project policy requiring enum-based state so static checks can work.

## 修正方針

Introduce a typed SelfhostDiagnosticCode hierarchy that mirrors the Rust diagnostic categories, keep stable strings only in a selfhost_diag_code_name/as_str conversion implemented with exhaustive match arms, and migrate current self-host diagnostic constructors and reporter/tests to use typed codes.

## 検証

Add focused regression tests that reject string-typed SelfhostDiagnostic.code, require reporter rendering to call the code-name conversion, and run the self-host diagnostic/reporter/module loader doctests.

## 2026-04-29 解決メモ

`SelfhostDiagnosticCode` を `Loader` / `Lexer` / `Parser` / `Resolve` / `Cli` の階層 enum として導入し、`SelfhostDiagnostic.code` を `str` から typed code に変更した。stable string は `selfhost_diag_code_name` と各 category の `*_diag_code_name` だけで生成し、reporter は human / JSON の両方でこの変換を通す。

`selfhost_diag_error` / `selfhost_diag_warning` / `selfhost_diag_info` は `SelfhostDiagnosticCode` を受け取る API に変更した。parser、loader、import spec、module path resolver、module graph、CLI driver、file_io の診断生成箇所は typed code constructor へ移行し、自由文字列 code を渡す経路を削除した。lexer の独自 `LexErrorCode` は廃止し、共有 `SelfhostLexerDiagnosticCode` を直接使うようにした。

CLI usage diagnostic は compiler stage の parser / resolver / loader code と混ざらないよう `Cli` category に隔離した。core compiler の parser / loader / resolve diagnostic は Rust 側の redesign と同じく category enum から stable string へ変換する。

### 2026-04-29 検証

- `node nodesrc/test_selfhost_diag_code_enum.js`: passed
- `trunk build`: passed
- `node nodesrc/test_selfhost_cli_reporter_boundary.js`: passed
- `node nodesrc/test_selfhost_cli_driver_boundary.js`: passed
- `node nodesrc/test_selfhost_cli_file_io_boundary.js`: passed
- `node nodesrc/test_selfhost_lexer_rust_parity.js`: passed
- `node nodesrc/tests.js -i stdlib/neplg2/core/infra/diag.nepl --no-tree -o tmp/selfhost-diag-code-enum-rebased6-diag.json -j 1`: total=1 passed=1
- `node nodesrc/tests.js -i tests/stdlib/neplg2_diag_outcome.n.md --no-tree -o tmp/selfhost-diag-code-enum-rebased6-outcome.json -j 1`: total=3 passed=3
- `node nodesrc/tests.js -i tests/stdlib/selfhost_cli_reporter.n.md --no-tree -o tmp/selfhost-diag-code-enum-rebased6-reporter.json -j 1`: total=3 passed=3
- `node nodesrc/tests.js -i tests/stdlib/neplg2_import_spec.n.md --no-tree -o tmp/selfhost-diag-code-enum-rebased-import-spec.json -j 1`: total=3 passed=3
- `node nodesrc/tests.js -i tests/stdlib/neplg2_module_loader.n.md --no-tree -o tmp/selfhost-diag-code-enum-rebased-loader.json -j 1`: total=2 passed=2
- `node nodesrc/tests.js -i tests/stdlib/neplg2_module_graph.n.md --no-tree -o tmp/selfhost-diag-code-enum-rebased-graph.json -j 1`: total=3 passed=3
- `node nodesrc/tests.js -i tests/stdlib/neplg2_stdlib_map.n.md --no-tree -o tmp/selfhost-diag-code-enum-rebased-stdlib-map.json -j 1`: total=3 passed=3
- `node nodesrc/tests.js -i tests/stdlib/neplg2_lexer.n.md --no-tree -o tmp/selfhost-diag-code-enum-rebased-lexer.json -j 1`: total=13 passed=13
- `node nodesrc/tests.js -i tests/stdlib/selfhost_cli_driver.n.md --no-tree -o tmp/selfhost-diag-code-enum-rebased-cli-driver.json -j 1`: total=3 passed=3
- `node nodesrc/tests.js -i tests/stdlib/selfhost_cli_file_io.n.md --no-tree -o tmp/selfhost-diag-code-enum-rebased-file-io.json -j 1`: total=4 passed=4
- `node nodesrc/tests.js -i stdlib/neplg2 --no-tree -o tmp/selfhost-diag-code-enum-rebased6-neplg2.json -j 2`: total=39 passed=39
