---
id: ISS-20260512T085522724Z-RESOURCE-OWNER-STORAGE-ORIGIN-CROSSE-180392ED
title: "Resource owner storage origin crosses raw deref into Copy cell payloads"
area: core/resource
status: verified
resolved: true
priority: P1
type: bug
created: 2026-05-12
updated: 2026-05-12
target: "nepl-core/src/resource/storage_origin.rs, nepl-core/tests/resource_ir.rs"
---

# ISS-20260512T085522724Z-RESOURCE-OWNER-STORAGE-ORIGIN-CROSSE-180392ED: Resource owner storage origin crosses raw deref into Copy cell payloads

## 概要

`StorageOriginTable` が `PlaceProjection::Deref` をまたいで `Owned` storage origin を継承していたため、owned raw address の指す cell payload まで free obligation owner として扱われていた。raw storage から読み出した Copy `i32` を callback / predicate に渡す経路で、payload value が storage owner であるかのように `resource.owner.no_free_obligation` が発生する。

## 対象

- `nepl-core/src/resource/storage_origin.rs, nepl-core/tests/resource_ir.rs`

## 根拠

- 静的検査大規模修正計画 Stage 4 の owner token / free obligation 分離に関わる問題。
- `MemPtr` / raw address は storage owner と initialized cell payload を分離して扱うべきであり、`Deref` は storage identity の投影ではなく cell payload への境界である。
- remote main の Vec stdlib 側では query / transform helper が `get<T: Copy>` 経由へ整理されたが、core Resource IR 側の owner identity 伝播規則も直接固定する必要がある。

## 問題

storage origin の prefix / suffix 判定が `Deref` を通常の aggregate field と同じように扱っていた。これにより、`p` が owned storage origin を持つとき、`p.*` から読み出した Copy payload や、その payload をさらに move/copy した値まで owner obligation を期待される。

## 影響

unknown callback や higher-order helper は保守的に引数 owner を消費するため、Copy payload が storage owner と誤認されると false positive が発生する。逆にこの境界が曖昧なままだと、storage owner identity と initialized cell payload state の責務分離が崩れ、Stage 4/6 の `MemPtr = non-owning pointer` 方針を壊す。

## 修正方針

storage origin の継承、`entries_under`、`origin_source`、origin move/copy の prefix 変換は、suffix に `PlaceProjection::Deref` を含む場合に owner identity を伝播しない。field / tuple / enum payload のような aggregate wrapper 内の owner identity は維持し、raw cell payload 境界だけを明示的に切る。

## 検証

- `cargo test -p nepl-core --test resource_ir resource_ir_owner_check_does_not_treat_raw_cell_payload_as_storage_owner -- --nocapture`
- `cargo check -p nepl-core`
- `cargo fmt --check -p nepl-core`
- `node nodesrc/issues.js check --dir issues`

## 解決

- `StorageOriginTable` の prefix / suffix origin lookup に、`Deref` を含む suffix を owner identity preserving と見なさない判定を追加した。
- `move_origin` / `copy_origin` が aggregate wrapper の owner origin は移せる一方、raw cell payload へは storage origin を移さないようにした。
- 回帰テストは stdlib Vec の現在の実装に依存せず、Resource IR の raw allocation -> raw load -> unknown indirect callback の最小経路で `CallArgument` false positive が出ないことを確認する。
