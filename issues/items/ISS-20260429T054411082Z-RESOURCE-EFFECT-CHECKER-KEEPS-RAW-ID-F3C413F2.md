---
id: ISS-20260429T054411082Z-RESOURCE-EFFECT-CHECKER-KEEPS-RAW-ID-F3C413F2
title: "Resource effect checker keeps raw identity payload after destructive overwrite"
area: core
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-04-29
updated: 2026-04-29
target: "nepl-core/src/resource/effect_check.rs, nepl-core/tests/resource_ir.rs"
---

# ISS-20260429T054411082Z-RESOURCE-EFFECT-CHECKER-KEEPS-RAW-ID-F3C413F2: Resource effect checker keeps raw identity payload after destructive overwrite

## 概要

Resource IR effect boundary tracking clears raw identity payload for Store with a non-identity value, but BulkCopy/BulkMove from a non-identity source and Fill leave the destination slot marked as carrying an internal allocation identity. The stale payload can make later Load/Return report RawAddressEscapeFromInternalAlloc even though the raw bytes were overwritten.

## 対象

- `nepl-core/src/resource/effect_check.rs, nepl-core/tests/resource_ir.rs`

## 根拠

- `nepl-core/src/resource/effect_check.rs` の `apply_raw_memory_identity_effect` は `RawMemoryOp::Store` で非 identity value を書いた場合に `RawMemoryIdentityTable` を clear していた。
- 一方で `RawMemoryOp::BulkCopy` / `BulkMove` は source が identity payload を持つ場合だけ destination を mark し、source が非 identity の場合に destination の既存 identity payload を消していなかった。
- `RawMemoryOp::Fill` も destination bytes を破壊的に上書きする operation だが、identity payload state を更新していなかった。

## 問題

Resource IR effect boundary tracking clears raw identity payload for Store with a non-identity value, but BulkCopy/BulkMove from a non-identity source and Fill leave the destination slot marked as carrying an internal allocation identity. The stale payload can make later Load/Return report RawAddressEscapeFromInternalAlloc even though the raw bytes were overwritten.

## 影響

Stage 5 raw identity escape diagnostics cannot become authoritative if destructive raw storage operations keep stale identity state. This causes false positives and makes the Resource IR model disagree with the actual raw memory transition.

## 修正方針

Make every destructive overwrite update RawMemoryIdentityTable with explicit write semantics: BulkCopy/BulkMove propagate source identity when present and clear destination when absent; Fill always clears destination identity. Add focused Resource IR regressions.

## 検証

cargo test -p nepl-core --test resource_ir resource_ir_effect_check_clears_raw_identity_payload -- --nocapture

## 2026-04-29 対応結果

`RawMemoryIdentityTable` の更新規則を destructive overwrite semantics に揃えた。

- `BulkCopy` / `BulkMove` は source slot が internal allocation identity payload を持つ場合は destination へ伝播し、持たない場合は destination の stale identity payload を clear する。
- `Fill` は destination bytes を値で上書きするため、destination の identity payload を常に clear する。
- `Store` の既存挙動は維持し、raw identity value を store した場合だけ payload mark、通常値の store では clear する。

この修正は `MemPtr` を owner として拡張するものではなく、Stage 5 の raw identity escape 判定で「raw memory slot の中身」と「pointer value 自体」を分離して扱うための Resource IR state 修正である。

回帰テストとして、identity payload を一度 raw slot に store した後、`Fill`、identity を含まない source からの `BulkCopy`、identity を含まない source からの `BulkMove` で上書きし、その後の `Load` / `Return` が `RawAddressEscapeFromInternalAlloc` にならないことを `nepl-core/tests/resource_ir.rs` に追加した。

検証:

- `cargo test -p nepl-core --test resource_ir resource_ir_effect_check_clears_raw_identity_payload -- --nocapture`: pass
- `cargo test -p nepl-core --test resource_ir -- --nocapture`: pass
- `cargo fmt --check -p nepl-core`: pass
- `cargo check -p nepl-core --tests`: pass
- `node nodesrc/issues.js check`: pass
