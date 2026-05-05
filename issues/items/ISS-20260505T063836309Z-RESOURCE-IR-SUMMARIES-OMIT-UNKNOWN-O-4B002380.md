---
id: ISS-20260505T063836309Z-RESOURCE-IR-SUMMARIES-OMIT-UNKNOWN-O-4B002380
title: "Resource IR summaries omit unknown-offset non-Copy raw load requirements"
area: core
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-05
updated: 2026-05-05
target: "nepl-core/src/resource/initialized_summary_destruction_address.rs,nepl-core/tests/resource_ir.rs"
---

# ISS-20260505T063836309Z-RESOURCE-IR-SUMMARIES-OMIT-UNKNOWN-O-4B002380: Resource IR summaries omit unknown-offset non-Copy raw load requirements

## 概要

parameter-derived unknown-offset raw address から non-Copy 値を load する helper の要件が function summary から落ち、caller が該当 cell の初期化を証明できない場合でも Resource IR check が通過する。

## 対象

- `nepl-core/src/resource/initialized_summary_destruction_address.rs,nepl-core/tests/resource_ir.rs`

## 根拠

- `cargo test -p nepl-core --test resource_ir resource_ir_cell_summary_rejects_unproven_unknown_offset_non_copy_raw_load -- --nocapture` を修正前に追加して実行すると、`main` 側の diagnostics が空になり、未証明の raw cell に対する helper load が拒否されていないことを確認した。
- `collect_param_moves_for_address` が `StorageOffset { bytes: None }` を含む address alias を summary 収集対象から除外していた。
- callee body は parameter-derived external raw storage root として許容されるため、caller-side precondition へ伝播しないと未初期化 / moved 済み non-Copy cell の raw load を見逃す。

## 問題

parameter-derived unknown-offset raw address から non-Copy 値を load する helper の要件が function summary から落ち、caller が該当 cell の初期化を証明できない場合でも Resource IR check が通過する。

## 影響

callee body は外部 raw storage root として許容される一方、caller 側に RawMemoryLoadCell precondition が伝播しないため、未初期化または moved 済み non-Copy cell の読み出しを見逃す。

## 修正方針

unknown offset を range cleanup 完了として扱わず、non-Copy raw load の caller-side availability precondition として summary へ保守的に伝播する。caller が initialized range を証明できない場合は RawMemoryLoadCell で拒否し、外部 untracked root は既存の外部 raw root 規則で許容する。

## 検証

manual ResourceModule の regression で unknown-offset helper load を main 側未証明 cell に適用した場合に CellUnavailable RawMemoryLoadCell が出ること、既存 exact raw-load summary tests と unknown-offset dealloc guard が通ることを確認する。

## 関連 issue

- `ISS-20260505T045316820Z-RESOURCE-IR-VEC-RANGE-CLEANUP-9A95DB6F`: dynamic offset range cleanup 証明の親 issue。本 issue は「unknown offset の 1-cell non-Copy raw load 要件を caller へ落とさない」false negative を先に塞ぐ。
- `ISS-20260505T061748334Z-RESOURCE-IR-SUMMARIES-OMIT-EXACT-NON-C8D2F16E`: exact offset の同種 summary 漏れ。

## 対応

- `collect_param_moves_for_address` から unknown offset alias の一律除外を削除した。
- unknown offset を range cleanup 完了としては扱わず、non-Copy raw load の caller-side `RawMemoryLoadCell` availability precondition としてだけ伝播する。
- callee 内で既に initialized と証明済みの cell は従来どおり summary 要件から除外し、external untracked root の許容は caller 側の既存規則に委ねる。
- manual `ResourceModule` regression を追加し、`p + ?` から non-Copy load する helper を未証明 local address に適用した場合に `main` 側で `CellUnavailable { operation: RawMemoryLoadCell, state: Uninit }` が出ることを固定した。

## 2026-05-05 検証結果

- `cargo test -p nepl-core --test resource_ir resource_ir_cell_summary -- --nocapture`: 4 passed
- `cargo test -p nepl-core --test resource_ir resource_ir_cell_check_allows_dealloc_after_non_copy_raw_load -- --nocapture`: passed
- `cargo test -p nepl-core --test resource_ir resource_ir_cell_check_reports_destructive_raw_storage_ops_over_live_cell -- --nocapture`: passed
- `cargo test -p nepl-core --test resource_ir resource_ir_cell_check_region_ptr_at_unknown_offset_rejects_dealloc_over_live_cell -- --nocapture`: 既存 source fixture が `Resolve::ShadowSameSignatureCallable` warning を hard failure として扱うため Resource IR 到達前に failed。この検証阻害は `ISS-20260505T064000205Z-RESOURCE-IR-SOURCE-FIXTURES-FAIL-ON--EDEA5603` として分離した。
