---
id: ISS-20260524T020418962Z-RESOURCE-IR-NEEDS-TRANSFORM-RANGE-LI-77E29B37
title: "Resource IR needs transform range lifecycle certificate"
area: core
status: open
resolved: false
priority: P1
type: architecture
created: 2026-05-24
updated: 2026-05-24
target: "nepl-core/src/resource/**, stdlib/alloc/collections/vec/transform/**"
---

# ISS-20260524T020418962Z-RESOURCE-IR-NEEDS-TRANSFORM-RANGE-LI-77E29B37: Resource IR needs transform range lifecycle certificate

## 概要

Vec.filter<T: Drop> を stdlib 側だけで direct raw MoveOut / InitializeEmpty loop にすると、source slot と output slot の range lifecycle が Resource IR に証明されない。pop/push 再帰案は既存単一 slot proof を再利用できるが、Drop payload doctest が 240 秒でも compile timeout し、実用的な transform engine にならない。

## 対象

- `nepl-core/src/resource/**, stdlib/alloc/collections/vec/transform/**`

## 根拠

- `ISS-20260523T051658073Z-VEC-NON-COPY-TRANSFORMS-NEED-BORROWE-A2D4AFE1` の実装検討で、`filter<T: Drop>` を direct raw drain として試すと `collection_slot.unavailable` が発生した。
- 失敗内容は、source `MoveOut` が `Uninitialized` slot に対する操作として拒否され、戻り値 `Vec` の output slot も `BorrowRead` 時に `Uninitialized` と見なされるものだった。
- 既存の `Vec.pop` / `Vec.push` は単一 slot の `MoveOut` / `InitializeEmpty` proof を通せるが、`pop` / `push` 再帰で filter を組む案は focused doctest の Drop payload case が 240 秒 timeout になり、実用的な transform engine ではなかった。
- `collection_slot_drop_traversal` には full initialized range certificate がある一方、transform 用に「source range を move-out し、discard branch を actual drop し、選択 branch だけ output prefix を initialize する」証明はまだ存在しない。

## 問題

Vec.filter<T: Drop> を stdlib 側だけで direct raw MoveOut / InitializeEmpty loop にすると、source slot と output slot の range lifecycle が Resource IR に証明されない。pop/push 再帰案は既存単一 slot proof を再利用できるが、Drop payload doctest が 240 秒でも compile timeout し、実用的な transform engine にならない。

## 影響

Vec map/filter/prefix/partition の non-Copy payload 対応を安全に進められず、selfhost の AST/HIR/diagnostic owner payload collection transform が Copy-only 境界に残る。stdlib function allowlist や metadata-only initialized_len 更新で回避すると Stage 6 の generic Resource IR proof 方針を壊す。

## 修正方針

drop traversal range certificate の兄弟として transform range lifecycle certificate を設計する。source range の MoveOut coverage、discard branch の actual Drop proof、output prefix の InitializeEmpty coverage、partial output rollback cleanup を typed summary / replay / local check に載せる。stdlib 側は証明器が読める単純な loop shape に固定し、raw shallow copy や module allowlist は使わない。

## 進捗

2026-05-24 checkpoint:

- `ResourceOp::CollectionSlotTransformRange` と `collection_slot_transform_range<T>` primitive を追加し、typecheck / lowering / coverage / dump / effect / borrow / owner summary の match 接続を行った。
- `CollectionSlotLifecycleSummaryOp::TransformRange` と certificate model を追加し、summary replay で source range `MoveOut`、output prefix `InitializeEmpty`、discard actual drop proof を検査する scaffold を入れた。discard drop proof がない non-Copy transform replay は拒否する。
- `CollectionSlotStateTable` に initialized range state を追加し、output prefix が storage release をブロックし、drop traversal certificate 後に release できることを regression 化した。branch merge では共通 range と explicit slot override を range-aware に merge する。
- summary producer は source drain / output prefix loop の最小形を認識する。output count は 0 起点、loaded value store、output increment 1 回、discard branch actual drop を要求する。
- 未完了: return value 上の output initialized range propagation、partial output rollback cleanup の source-level fixture、stdlib `filter<T: Drop>` / prefix / map / partition への接続。local `CollectionSlotTransformRange` op は certificate なしに state を更新しない summary marker のままで、public transform overload はまだ開かない。

## 検証

Resource IR compile-pass/fail regressions for Vec<DropPayload>.filter, prefix, map, partition. Source policy must reject Copy-bound removal unless borrowed observation, MoveOut coverage, output InitializeEmpty coverage, discard Drop proof, and rollback cleanup are structurally present. Focused doctests must pass within normal timeout.
