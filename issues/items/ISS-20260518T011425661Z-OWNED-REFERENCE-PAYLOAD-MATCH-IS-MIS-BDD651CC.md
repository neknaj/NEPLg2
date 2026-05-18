---
id: ISS-20260518T011425661Z-OWNED-REFERENCE-PAYLOAD-MATCH-IS-MIS-BDD651CC
title: "Owned reference payload match is misclassified as borrowed payload binding"
area: core
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-18
updated: 2026-05-18
target: "nepl-core/src/hir.rs, nepl-core/src/typecheck/match_check.rs, nepl-core/src/resource/lower.rs, nepl-core/src/codegen_wasm.rs, nepl-core/src/codegen_llvm.rs"
---

# ISS-20260518T011425661Z-OWNED-REFERENCE-PAYLOAD-MATCH-IS-MIS-BDD651CC: Owned reference payload match is misclassified as borrowed payload binding

## 概要

HIR and Resource IR only inferred match payload binding behavior from the binding type. An owned enum payload whose payload type is itself a reference was therefore lowered like a borrowed enum match payload, causing Resource IR to borrow from a moved payload and causing wasm/LLVM codegen to bind the payload slot address instead of the stored reference value.

## 対象

- `nepl-core/src/hir.rs, nepl-core/src/typecheck/match_check.rs, nepl-core/src/resource/lower.rs, nepl-core/src/codegen_wasm.rs, nepl-core/src/codegen_llvm.rs`

## 根拠

- CI artifact `tmp/gh-run-26004187689/wasi-tests/tests-current.json` で `tests/compiler/move_check.n.md::doctest#38/#39` が失敗し、owned enum `RefOpt::Some &x` の payload bind が `resource.cell.moved` に到達していた。
- `cargo test -p nepl-core --test move_check move_match_reference_payload -- --nocapture` でも `move_match_reference_payload_last_use_releases_owner` が `Borrow on ... EnumPayload { variant: "Some" } found Moved` で失敗した。
- `nepl-core/src/resource/lower.rs` は `bind_ty` が reference であることを根拠に match arm 冒頭へ `ResourceOp::Borrow` を挿入していたため、owned match の reference payload と borrowed enum match の payload reference を区別できなかった。
- wasm/LLVM backend も同じく `bind_ty` が reference であることを根拠に payload slot address を束縛しており、owned reference payload では stored reference value ではなく slot address を読む危険があった。

## 問題

HIR and Resource IR only inferred match payload binding behavior from the binding type. An owned enum payload whose payload type is itself a reference was therefore lowered like a borrowed enum match payload, causing Resource IR to borrow from a moved payload and causing wasm/LLVM codegen to bind the payload slot address instead of the stored reference value.

## 影響

Static checking reports resource.cell.moved before the intended resource.borrow.move_from_shared diagnostic, and valid owned enum reference payload matches can fail or dereference the wrong address. This weakens the Resource IR proof boundary by conflating owner transfer and borrowed observation.

## 修正方針

Represent match payload binding mode explicitly in HIR and Resource IR as an enum. Use Owned mode to transfer/copy payload borrow tokens from the matched value, and Borrowed mode only for matches over &Enum that must seed a borrow of the payload place. Make wasm/LLVM codegen branch on that mode rather than on whether the payload binding type is a reference.

## 検証

Focused Rust move_check tests must pass for both live-reference rejection and last-use release. The affected move_check doctests #38/#39 and reference_codegen doctests must pass after trunk build.

## 解決

- `HirMatchBindMode` と `ResourceMatchBindMode` を追加し、match payload binding が `Owned` なのか `Borrowed { is_mut }` なのかを型付き enum として HIR から Resource IR へ渡すようにした。
- Resource lowering は `Borrowed` のときだけ payload place から synthetic `ResourceOp::Borrow` を生成し、`Owned` の reference payload では既存の payload token transfer 経路を使うようにした。
- Resource borrow checker の match payload token propagation は `Owned` binding では reference payload の borrow token を arm binding へ移し、`Borrowed` binding では synthetic borrow に任せる形に分離した。
- wasm/LLVM backend は `bind_ty` が reference かどうかではなく binding mode に基づいて payload address binding と payload value load を選ぶようにした。
- `tests/compiler/reference_codegen.n.md` に owned enum reference payload を deref して値を読む doctest を追加した。

## 修正後検証

- `cargo fmt --all --check`: pass
- `cargo check -p nepl-core`: pass
- `cargo test -p nepl-core --test move_check move_match_reference_payload -- --nocapture`: 2/2 passed
- `cargo test -p nepl-core resource_ir_compiler_rejects_match_payload_borrow_move -- --nocapture`: pass
- `trunk build`: pass
- `node nodesrc/run_doctest.js -i tests/compiler/move_check.n.md -n 38 --dist web/dist`: pass
- `node nodesrc/run_doctest.js -i tests/compiler/move_check.n.md -n 39 --dist web/dist`: pass
- `node nodesrc/tests.js -i tests/compiler/reference_codegen.n.md --no-tree -o tmp/agent1-match-payload-bind-mode-reference-codegen.json -j 1 --dist web/dist --assert-io`: 6/6 passed
- `node nodesrc/tests.js -i tests/compiler/move_check.n.md --no-tree -o tmp/agent1-match-payload-bind-mode-move-check.json -j 1 --dist web/dist --assert-io`: timeout after 184s; focused doctest #38/#39 で代替確認済み。
