---
id: ISS-20260521T232236770Z-DROP-TRAVERSAL-RANGE-WITNESS-MISSES--0623A850
title: "Drop traversal range witness misses source-level Drop call proof"
area: core
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-21
updated: 2026-05-22
target: "nepl-core/src/resource/collection_slot_summary_build_range_witness*.rs, nepl-core/tests/collection_slot_full_range.rs"
---

# ISS-20260521T232236770Z-DROP-TRAVERSAL-RANGE-WITNESS-MISSES--0623A850: Drop traversal range witness misses source-level Drop call proof

## 概要

After witness load/drop indexing was made explicit, source-level Drop::drop lowering still emits ResourceOp::Drop proof but the range witness detector only accepts a direct Drop of the raw-load output, so source-level full-range cleanup no longer produces ForallInitializedRange summary.

## 対象

- `nepl-core/src/resource/collection_slot_summary_build_range_witness*.rs, nepl-core/tests/collection_slot_full_range.rs`

## 根拠

- Parent issue: [ISS-20260520T152218366Z-NON-COPY-COLLECTION-PAYLOAD-SUPPORT--A6A88543](./ISS-20260520T152218366Z-NON-COPY-COLLECTION-PAYLOAD-SUPPORT--A6A88543.md)
- 設計段階: [NEPLg2 静的検査の複雑化解消計画 Stage 6](../../doc/neplg2/static_check_complexity_reduction_plan.md#stage-6-resource-ir-と-stdlib-境界の完成)
- 直前の関連修正: [ISS-20260521T225837153Z-DROP-TRAVERSAL-RANGE-CERTIFICATE-SUR-83ACCED9](./ISS-20260521T225837153Z-DROP-TRAVERSAL-RANGE-CERTIFICATE-SUR-83ACCED9.md)
- source-level regression: `nepl-core/tests/collection_slot_full_range.rs` の `source_loop_drop_traversal_summary_cleans_caller_initialized_range` は、`Drop::drop &loaded` が runtime call と `ResourceOp::Drop` proof の両方へ lower される実経路を通す。

## 問題

After witness load/drop indexing was made explicit, source-level Drop::drop lowering still emits ResourceOp::Drop proof but the range witness detector only accepts a direct Drop of the raw-load output, so source-level full-range cleanup no longer produces ForallInitializedRange summary.

## 影響

Source-level compiler-owned collection cleanup cannot prove initialized range disposal, leaving caller storage release with live non-Copy slots and blocking non-Copy collection/self-host progress.

## 修正方針

Recognize source-derived Drop call proof as the witness flow without weakening the certificate: derive witness end from Resource IR state transition/drop proof rather than exact direct Drop place equality, while still rejecting extra protected loads.

## 対応

- `LoopBodyCandidateSlot` から raw-load output と direct `Drop` equality への依存を削除し、candidate load 以降の prefix を Resource IR checker に通して、直前 prefix では未証明かつ当該 prefix で `collection_slot_drop_traversal_result` が成功する位置を witness drop index として採用するようにした。
- source lowering が `Call { UnsafeMemory::Load }` と `RawMemory { Load }` を同じ output / args で連続発行する形を、generic な paired witness load として扱うようにした。関数名や stdlib module 名の allowlist は追加していない。
- witness 構築区間の preservation を通常 tail preservation から分離し、純粋 call と raw address view / alias 構築は Resource IR effect と state proof に基づいて通す一方、選択 witness 以外の protected storage load、protected storage への直接書き込み、nested control / lifecycle / relocate / traversal は拒否する。
- witness 後の `EndScope` は raw pointer / slot alias の local scope を閉じるだけなら certificate を失効させず、result が protected anchor を外へ運ぶ場合は従来通り拒否するようにした。
- op 単位の witness-window 判定を `collection_slot_summary_build_range_preserve_witness_op.rs` へ分離し、`collection_slot_summary_build_range_preserve_witness.rs` は body 走査に集中させた。
- source-level negative regression として、`Drop::drop &loaded` 後かつ `set i add i 1` 前に同じ protected slot を raw load するケースが `ForallInitializedRange` summary を生成しないことを固定した。

## 検証

Run collection_slot_full_range source-level regression plus focused range certificate tests, cargo check, responsibility monitor, and issue checks.

- `cargo test -p nepl-core --test collection_slot_full_range source_loop_drop_traversal_summary_cleans_caller_initialized_range -- --nocapture`
- `cargo test -p nepl-core --test collection_slot_full_range -- --nocapture`
- `cargo test -p nepl-core --test resource_ir resource_ir_collection_slot_source_drop_initialized_accepts_actual_loaded_value_drop -- --nocapture`
- `cargo test -p nepl-core --test resource_ir resource_ir_collection_slot_source_drop_initialized_rejects_raw_load_without_drop -- --nocapture`
- `cargo test -p nepl-core --lib collection_slot_summary_loop_induction -- --nocapture`
- `cargo test -p nepl-core --lib collection_slot_summary_loop_certificate -- --nocapture`
- `cargo test -p nepl-core --lib body_preserve -- --nocapture`
- `cargo check -p nepl-core`
- `cargo fmt --check -p nepl-core`
- `node nodesrc/test_resource_checker_responsibility.js`
- `node nodesrc/issues.js check --dir issues`
- `git diff --check`
