---
id: ISS-20260513T215609976Z-SELF-HOST-LEXER-READS-VEC-RAW-DATA-F-8A56A6A1
title: "Self-host lexer reads Vec raw data field directly"
area: selfhost
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-13
updated: 2026-05-14
target: "stdlib/neplg2/core/syntax/lexer.nepl, stdlib/alloc/collections/vec/mutation/pop.nepl, nepl-core/src/resource/owner_check.rs, nepl-core/src/resource/owner_summary_variant_return.rs, nepl-core/src/resource/owner_variant_utils.rs, nepl-core/tests/resource_ir.rs, tests/stdlib/neplg2_lexer.n.md, nodesrc/test_selfhost_lexer_raw_mode_directive_enum.js"
---

# ISS-20260513T215609976Z-SELF-HOST-LEXER-READS-VEC-RAW-DATA-F-8A56A6A1: Self-host lexer reads Vec raw data field directly

## 概要

lex_stack_drop_top reconstructs the indent stack by reading Vec.data directly with field::get, bypassing the public Vec owner API and carrying raw storage-field discipline into self-host syntax code.

## 対象

- `stdlib/neplg2/core/syntax/lexer.nepl`
- `stdlib/alloc/collections/vec/mutation/pop.nepl`
- `nepl-core/src/resource/owner_check.rs`
- `nepl-core/src/resource/owner_summary_variant_return.rs`
- `nepl-core/src/resource/owner_variant_utils.rs`
- `nepl-core/tests/resource_ir.rs`
- `tests/stdlib/neplg2_lexer.n.md`
- `nodesrc/test_selfhost_lexer_raw_mode_directive_enum.js`

## 根拠

- `stdlib/neplg2/core/syntax/lexer.nepl` の `lex_stack_drop_top` は `field::get stack "data"` で `Vec<i32>` の raw `MemPtr<i32>` storage field を直接取り出していた。
- `Vec` の `data` field は Stage 6 の transitional owner model debt であり、self-host compiler code はこの layout に依存せず public `Vec` API を使う必要がある。
- 修正後の focused doctest で、Resource owner checker が non-Copy `ResourceOp::Read` を owner move として扱わず、`unwrap_ok push v0 7` の入力 `Vec` owner を call argument 側へ残す false leak が露出した。
- `push` の owner summary は同じ `Result::Ok` payload について fresh owner と parameter-derived owner の複数候補を同時に持つことがあり、適用時に fresh 側だけが実体化すると parameter 側の消費が失われていた。

## 問題

lex_stack_drop_top reconstructs the indent stack by reading Vec.data directly with field::get, bypassing the public Vec owner API and carrying raw storage-field discipline into self-host syntax code.

## 影響

Self-host compiler code can depend on Vec's transitional MemPtr storage layout instead of the compiler-proven public API boundary, weakening the Stage 6 plan to keep raw-memory-backed collection internals out of self-host implementation.

## 修正方針

Rewrite lex_stack_drop_top to consume the stack through a public Vec owner API, then fix the compiler-side owner transfer proof that the focused tests exposed. non-Copy `ResourceOp::Read` must move owner obligations like the initialized-cell checker already does, and ambiguous variant projection owner returns must be normalized so parameter consumption cannot be hidden by a fresh-owner alternative.

## 検証

Run the self-host lexer source policy, Vec source policy, focused Resource IR owner regressions, focused self-host lexer fixture, focused Vec doctests, and issue checks.

## 対応結果

- `Vec` module に、末尾値を使わず次の owner だけを返す `drop_last<T: Copy>` を追加した。
- `lex_stack_drop_top` を `drop_last<i32>` による public owner API 経由へ置き換えた。
- `nodesrc/test_selfhost_lexer_raw_mode_directive_enum.js` に、self-host lexer が `Vec.data` を直接読まないこと、`lex_stack_drop_top` が public `Vec` owner API を使うことを検査する regression を追加した。
- `ResourceOwnerCheckEngine` の `Read` 処理を initialized-cell checker と揃え、non-Copy source では owner state を output へ transfer して source raw view を clear するようにした。
- variant owner projection return summary は、同一 variant / suffix / ty に複数の owner 候補がある場合に `Maybe` へ正規化する。これにより `push` のように runtime path により fresh owner または parameter-derived owner を返す関数でも、parameter 消費を fresh return が隠さない。
- `unwrap_ok push<i32> v0 7` が入力 `Vec` owner を返却 payload へ正しく移送し、`free<i32> v1` で閉じられることを Rust regression で固定した。
- self-host lexer doctest のうち `Option.unwrap` を使う後半 fixture に `core/option` import が不足していたため、focused fixture が現在の明示 import 方針で通るように更新した。

## 検証結果

- `cargo test -p nepl-core --test resource_ir resource_ir_owner_check_unwrap_ok_push_transfers_vec_owner -- --nocapture`: passed
- `cargo test -p nepl-core --test resource_ir resource_ir_owner_check_forwards_nested_byte_builder_result_owner -- --nocapture`: passed
- `trunk build`: passed
- `node nodesrc/tests.js -i stdlib/alloc/collections/vec/mutation/push.nepl --no-tree -o tmp/agent1-vec-push-after-owner-summary-fix.json -j 2 --dist web/dist`: total=2, passed=2
- `node nodesrc/tests.js -i stdlib/alloc/collections/vec/mutation/pop.nepl --no-tree -o tmp/agent1-vec-pop-after-owner-summary-fix.json -j 2 --dist web/dist`: total=3, passed=3
- `node nodesrc/tests.js -i tests/stdlib/neplg2_lexer.n.md --no-tree -o tmp/agent1-selfhost-lexer-final.json -j 2 --dist web/dist`: total=13, passed=13
- `node nodesrc/test_selfhost_lexer_raw_mode_directive_enum.js`: passed
- `node nodesrc/test_stdlib_memptr_owner_field_policy.js`: passed

## 関連ドキュメント

- [NEPLg2 静的検査の複雑化解消計画](../../doc/neplg2/static_check_complexity_reduction_plan.md)
- [stdlib collection/mem/string と静的検査の安全設計](../../doc/neplg2/stdlib_collection_mem_string_static_safety_design.md)
