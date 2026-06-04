---
id: ISS-20260604T000000000Z-TEST-MODE-DIRECTIVE-FOR-DOCTEST-AND-CLI-TEST
title: "Add #test directive for doctest and nepl-cli test"
area: core
status: open
resolved: false
priority: P1
type: architecture
created: 2026-06-04
updated: 2026-06-04
target: "nepl-core/src/lexer.rs; nepl-core/src/parser.rs; nepl-core/src/target_gate.rs; nepl-core/src/target_precheck.rs; nepl-core/src/typecheck; nepl-cli/src/main.rs; nepl-web/src/lib.rs; nodesrc/run_test.js; nodesrc/tests.js; stdlib/neplg2/core/syntax; doc/neplg2/test_mode_directive_design.md"
---

# ISS-20260604T000000000Z-TEST-MODE-DIRECTIVE-FOR-DOCTEST-AND-CLI-TEST: Add #test directive for doctest and nepl-cli test

## 背景

NEPLg2 には Rust の `cfg(test)` に相当する test-only compile mode がない。

そのため、source file 内に doctest / `nepl-cli test` 専用 helper を置くと、通常 compile にも helper が混入する。逆に、通常 compile を汚さないために helper を外へ逃がすと、テスト対象の実装と近い場所で検証 code を保てない。

## 方針

`#test` を直後 1 statement に効く directive として追加する。

`test_mode=false` の compile では `#test` の直後 1 statement を無効化し、`test_mode=true` の compile では有効化する。

これは `profile=debug/release` と直交する compile axis である。

## 実装項目

- `TokenKind::DirTest` と `Directive::Test` を追加する。
- lexer は `#test` を `DirTest` として token 化する。
- parser は `DirTest` を `Directive::Test` として AST 化する。
- `target_gate` / `target_precheck` は `test_mode` を含めて active statement を判定する。
- typecheck、raw body precheck、LLVM codegen、source cache key、public surface hash、artifact / proof cache key を `test_mode` に対応させる。
- `nodesrc/tests.js` / `nodesrc/run_test.js` と `nepl-cli test` は test mode を true にする。
- selfhost lexer/parser の token/directive enum を同期する。

## 検証

- `cargo check -p nepl-core -p nepl-cli -p nepl-language`
- `cargo check` in `nepl-web/`
- `cargo test -p nepl-core --test functions test_directive`
- `node nodesrc/test_selfhost_lexer_raw_mode_directive_enum.js`
- `node nodesrc/issues.js check --dir issues`
- `git diff --check`
