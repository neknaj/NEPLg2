---
id: ISS-20260507T144627703Z-RUST-PARSER-AND-BACKEND-CODEGEN-LACK-11798587
title: "Rust parser and backend codegen lack responsibility split policy"
area: core
status: fixed
resolved: true
priority: P2
type: architecture
created: 2026-05-07
updated: 2026-05-08
target: "nepl-core/src/parser.rs, nepl-core/src/codegen_wasm.rs, nepl-core/src/codegen_llvm.rs, nepl-core/src/monomorphize.rs, nodesrc/test_parser_backend_responsibility_policy.js, doc/neplg2/parser_backend_responsibility_split_plan.md"
---

# ISS-20260507T144627703Z-RUST-PARSER-AND-BACKEND-CODEGEN-LACK-11798587: Rust parser and backend codegen lack responsibility split policy

## 概要

Current Rust compiler review shows parser.rs has 4234 lines, codegen_llvm.rs has 4188 lines, codegen_wasm.rs has 2573 lines, and monomorphize.rs has 1391 lines. Typecheck and ResourceIR now have module split/source-policy guards, but parser/backend/mono do not have equivalent responsibility boundaries. Parser.rs also repeats the module-level doc line, which is a small sign that the file is no longer being curated at the same granularity as the safety-critical modules.

## 対象

- `nepl-core/src/parser.rs, nepl-core/src/codegen_wasm.rs, nepl-core/src/codegen_llvm.rs, nepl-core/src/monomorphize.rs`

## 根拠

- `Get-ChildItem nepl-core/src -File` による現行行数確認で `parser.rs` は 4234 lines、`codegen_llvm.rs` は 4188 lines、`codegen_wasm.rs` は 2573 lines、`monomorphize.rs` は 1391 lines。
- `nodesrc/test_static_check_boundary_responsibility.js` は typecheck file の分割と line limit を監視している。
- `nodesrc/test_resource_checker_responsibility.js` は ResourceIR file の module ownership と line limit を監視している。
- 同等の parser/backend/monomorphize responsibility source policy は現時点で確認できなかった。
- `nepl-core/src/parser.rs` の冒頭には同じ module doc line が重複しており、巨大 file の保守粒度が荒くなっている兆候がある。

## 問題

Current Rust compiler review shows parser.rs has 4234 lines, codegen_llvm.rs has 4188 lines, codegen_wasm.rs has 2573 lines, and monomorphize.rs has 1391 lines. Typecheck and ResourceIR now have module split/source-policy guards, but parser/backend/mono do not have equivalent responsibility boundaries. Parser.rs also repeats the module-level doc line, which is a small sign that the file is no longer being curated at the same granularity as the safety-critical modules.

## 影響

Large unguarded compiler modules make diagnostics, target gates, layout lowering, match/codegen parity, and future selfhost parity harder to audit. Under the project policy, static safety must stay reviewable; unchecked growth in parser/backend code can hide string/number sentinel logic, wildcard branches, backend-only layout drift, or panic-prone paths.

## 修正方針

Design a responsibility split before further large parser/backend changes. Parser should be split by token stream/navigation, declarations, block/expression parsing, match/literal/type-expression parsing, and recovery. WASM/LLVM codegen should split shared layout/runtime helper lowering from backend-specific instruction emission, call lowering, aggregate lowering, match lowering, and raw body handling. Add source-policy regressions with line limits and required module ownership, similar to typecheck/resource policy.

## 検証

Add nodesrc source-policy tests for parser/codegen/monomorphize responsibility boundaries, then run the Rust compiler parser/codegen regression suites in GitHub Actions. Local pre-commit checks should include git diff --check and focused source-policy validation when implementation begins.

## 対応結果

2026-05-08 に [NEPLg2 parser / backend responsibility split plan](../../doc/neplg2/parser_backend_responsibility_split_plan.md) を追加し、parser / WASM backend / LLVM backend / monomorphize の責務境界と段階的な分割 stage を明文化した。

`nodesrc/test_parser_backend_responsibility_policy.js` を追加し、次を source policy として固定した。

- split plan が存在し、parser / backend / monomorphize の stage とこの issue への link を含むこと。
- `parser.rs` の重複 module doc line が戻らないこと。
- `parser.rs` / `codegen_wasm.rs` / `codegen_llvm.rs` / `monomorphize.rs` が現行 baseline 以上に増えないこと。
- `nodesrc/run_source_policy_regressions.js` からこの policy が実行されること。

巨大 file の物理分割はこの commit で雑に始めず、まず凍結線を置いた。今後 parser/backend/monomorphize に大きな変更を入れる場合は、計画文書の stage に沿って module を切り出し、root file の line limit を段階的に下げる。limit を上げることで責務集中を隠すことは禁止する。

検証:

- `node nodesrc/test_parser_backend_responsibility_policy.js`
- `node nodesrc/issues.js check`
- `git diff --check`
- `node nodesrc/run_source_policy_regressions.js --warn-only`
