---
id: ISS-20260521T214612047Z-SOURCE-LEVEL-DROP-CALLS-DO-NOT-PRODU-730918AE
title: "Source-level Drop calls do not produce generic Resource drop proof"
area: core
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-05-21
updated: 2026-05-22
target: "nepl-core/src/typecheck/trait_call_apply.rs, nepl-core/src/resource/drop_call_identity.rs, nepl-core/src/resource/lower.rs, nepl-core/src/resource/coverage_hir*.rs, nepl-core/tests/resource_ir.rs, nepl-core/tests/drop.rs, nepl-core/tests/collection_slot_full_range.rs"
---

# ISS-20260521T214612047Z-SOURCE-LEVEL-DROP-CALLS-DO-NOT-PRODU-730918AE: Source-level Drop calls do not produce generic Resource drop proof

## 概要

HIR には `HirExprKind::Drop` があり、Resource lowering は `ResourceOp::Drop` を発行できる。しかし現状の parser / typecheck は source から `HirExprKind::Drop` を構築していない。

そのため user source や stdlib の generic cleanup code は、loaded-value drop proof を作るために assignment overwrite などの副作用に頼るしかない。これは `.T: Drop` collection traversal の汎用 proof boundary として不適切である。

## 対象

- `nepl-core/src/typecheck/**, nepl-core/src/resource/lower*.rs, nepl-core/src/passes/drop_insertion.rs, stdlib/alloc/collections/**`

## 根拠

- 親 issue: [Non-Copy collection payload support needs compiler-issued owner and drop traversal](./ISS-20260520T152218366Z-NON-COPY-COLLECTION-PAYLOAD-SUPPORT--A6A88543.md)
- 前提 issue: [Generic Drop-bound type variables are treated as StateOnly by Resource drop requirements](./ISS-20260521T214431160Z-GENERIC-DROP-BOUND-TYPE-VARIABLES-AR-F3533E34.md)
- 関連 doc: [NEPLg2 静的検査の複雑化解消計画](../../doc/neplg2/static_check_complexity_reduction_plan.md)
- 関連 doc: [stdlib collection/mem/string と静的検査の安全設計](../../doc/neplg2/stdlib_collection_mem_string_static_safety_design.md)
- 開発方針: https://zenn.dev/bem130/articles/1b352797de94e7

## 問題

`HirExprKind::Drop { name }` は Resource IR 上の `ResourceOp::Drop` に lower され、raw cell loaded-value drop proof と接続される。しかし source parser の `Symbol` には Drop が存在せず、`drop` は通常 identifier として処理される。

また Drop trait call は `HirExprKind::Call { callee: FuncRef::Trait, ... }` として残るため、現状では destructor call と Resource proof marker が分離されている。単純に `HirExprKind::Drop` へ置き換えると codegen で destructor 実行を落とす危険がある。

## 影響

Non-Copy Vec clear/free cannot be implemented as a clean source-level loop over .T: Drop without either relying on ad hoc mutation tricks or adding module-specific proof shortcuts. This blocks replacing Copy-only collection cleanup with compiler-proven generic drop traversal.

## 修正方針

destructor 実行を保持したまま `ResourceOp::Drop` evidence を発行する source-level drop proof path を設計する。

候補:

- verified Drop trait call を Resource lowering で精密に認識し、call 後に対応する `ResourceOp::Drop` proof を発行する。
- あるいは explicit drop syntax を追加し、typecheck 済み place semantics と destructor call generation を一体で扱う。

禁止事項:

- stdlib module / function allowlist で drop 済み扱いにしない。
- marker-only proof で loaded payload drop を省略しない。
- destructor 実行を落とす `HirExprKind::Drop` への単純置換はしない。

## 検証

- source-level explicit / generic drop proof が `collection_slot_drop_initialized` / `collection_slot_drop_traversal` の proof になる。
- raw load だけで drop しない場合は拒否される。
- double drop / drop 後 read / borrowed 中 drop は拒否される。
- destructor codegen が 1 回だけ実行され、Resource proof と runtime destructor が分離しない。

## 解決内容

2026-05-22 に Agent 1 が source-level `Drop::drop &place` を Resource IR の汎用 drop proof として扱う経路を実装した。

- trait method self type 推論を `&Self` などの構造から復元するようにし、`Drop::drop &loaded` が明示型引数なしで正しく `Self = LocalOwner` と解決されるようにした。
- `#capability drop` を持つ trait method と、その monomorphize 後の Drop impl function を `DropCallIdentityIndex` で一元的に識別するようにした。
- Resource lowering は destructor `Call` を保持したまま、その後に `ResourceOp::Drop` を追加する。これにより runtime destructor 実行と static loaded-value drop proof が分離しない。
- HIR/Resource coverage も同じ Drop call identity を参照するため、本番 pipeline の monomorphize 後でも coverage gate が正しく Drop proof を認識する。
- `collection_slot_full_range` の loop proof fixture を assignment overwrite trick から `Drop::drop &loaded` に置き換えた。

## 回帰テスト

- `cargo test -p nepl-core --test resource_ir resource_ir_collection_slot_source_drop_initialized_accepts_actual_loaded_value_drop -- --nocapture`
- `cargo test -p nepl-core --test resource_ir resource_ir_monomorphized_drop_trait_call_still_emits_drop_proof -- --nocapture`
- `cargo test -p nepl-core --test resource_ir resource_ir_collection_slot_source_drop_initialized_rejects_raw_load_without_drop -- --nocapture`
- `cargo test -p nepl-core --test collection_slot_full_range -- --nocapture`
- `cargo test -p nepl-core --test drop explicit_drop_trait_call_runs_once_and_suppresses_auto_drop -- --nocapture`
- `cargo check -p nepl-core`
- `cargo fmt --check -p nepl-core`
