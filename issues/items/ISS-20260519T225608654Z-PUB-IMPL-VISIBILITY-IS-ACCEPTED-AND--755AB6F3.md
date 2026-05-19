---
id: ISS-20260519T225608654Z-PUB-IMPL-VISIBILITY-IS-ACCEPTED-AND--755AB6F3
title: "pub impl visibility is accepted and discarded"
area: core
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-19
updated: 2026-05-20
target: "nepl-core/src/parser.rs; stdlib/neplg2/core/proof/solver.nepl; tests/compiler/impl_visibility.n.md; tests/stdlib/neplg2_checker_impl_visibility.n.md; tests/stdlib/neplg2_proof.n.md"
---

# ISS-20260519T225608654Z-PUB-IMPL-VISIBILITY-IS-ACCEPTED-AND--755AB6F3: pub impl visibility is accepted and discarded

## 概要

Rust parser accepts pub impl through the pub top-level branch, then parse_impl consumes visibility and discards it even though ImplDef has no visibility field. The self-host declaration header proof also treats public impl headers as valid.

## 対象

- `nepl-core/src/parser.rs; stdlib/neplg2/core/proof/solver.nepl; tests/compiler/impl_visibility.n.md; tests/stdlib/neplg2_checker_impl_visibility.n.md; tests/stdlib/neplg2_proof.n.md`

## 根拠

- `ImplDef` は `vis` field を持たないため、現行言語設計では `impl` declaration の visibility を AST / resolve / export model へ保持できない。
- それにもかかわらず Rust parser は `pub impl` を top-level `KwPub` branch から `parse_impl` へ渡し、`parse_impl` が `parse_visibility` の結果を `_vis` として捨てていた。
- self-host 側でも parser が `SelfhostModuleDeclarationVisibility::Public` を declaration header evidence に保持した後、proof solver が `Impl` + `Public` を拒否していなかった。
- Stage 6 の方針では、抽象化機能と静的検査の前提を文字列や黙殺にせず、enum/match による typed proof / diagnostic で扱う必要がある。

## 問題

Rust parser accepts pub impl through the pub top-level branch, then parse_impl consumes visibility and discards it even though ImplDef has no visibility field. The self-host declaration header proof also treats public impl headers as valid.

## 影響

Visibility becomes an implicit parser-side exception for trait/impl abstraction. Export and coherence checks cannot rely on typed AST facts, and source that appears public is compiled as private without a diagnostic.

## 修正方針

Reject public impl visibility at the parser/proof boundary. Keep impl visibility semantics explicit by requiring impl declarations to be private unless the language design later adds a typed visibility field to ImplDef and self-host declaration proof.

## 検証

Add compile_fail regression for pub impl with parser diagnostic and self-host checker regression that public impl declaration header evidence is refuted.

- `ParserDiagnosticCode::ImplVisibilityInvalid` / `parser.impl.visibility_invalid` を追加し、`pub impl` を generic token unexpected ではなく専用 enum variant で拒否するようにした。
- `parse_impl` から捨てていた `parse_visibility` を削除し、`ImplDef` に存在しない visibility を parser 内で受け取らない構造にした。
- `selfhost_proof_module_declaration_visibility_allowed` を追加し、`SelfhostModuleDeclarationVisibility::Public` の場合だけ declaration kind を網羅 match して `Impl` を拒否するようにした。
- `tests/compiler/impl_visibility.n.md` に `pub impl` の compile_fail regression を追加した。
- `tests/stdlib/neplg2_proof.n.md` に public impl header evidence が `ModuleDeclarationHeaderInvalid` になる proof regression を追加した。
- `tests/stdlib/neplg2_checker_impl_visibility.n.md` に self-host parser -> checker 経由で public impl header が拒否される regression を追加した。
- focused verification:
  - `cargo test -p nepl-core diagnostic_codes --lib -- --nocapture`: passed
  - `trunk build`: passed
  - `node nodesrc/test_selfhost_diag_code_enum.js`: passed
  - `node nodesrc/test_selfhost_proof_entry_contract.js`: passed
  - `node nodesrc/tests.js -i tests/compiler/impl_visibility.n.md --no-tree -o tmp/agent1-pub-impl-visibility.json -j 1 --dist web/dist --assert-io`: 1/1 passed
  - `node nodesrc/tests.js -i tests/stdlib/neplg2_proof.n.md --no-tree -o tmp/agent1-pub-impl-proof.json -j 1 --dist web/dist --assert-io`: 4/4 passed
  - `node nodesrc/tests.js -i tests/stdlib/neplg2_checker_impl_visibility.n.md --no-tree -o tmp/agent1-pub-impl-checker-impl-visibility.json -j 1 --dist web/dist --assert-io`: 1/1 passed
- 追加で、既存 `tests/stdlib/neplg2_checker.n.md::doctest#1` が default 60000ms compile timeout をわずかに超え、120000ms 設定では compile_ms 約 61208ms で通ることを確認した。この件は `ISS-20260519T232628976Z-SELF-HOST-CHECKER-DOCTEST-SUMMARY-CA-2E6B074B` として分離した。
