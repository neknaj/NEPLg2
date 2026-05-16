---
id: ISS-20260516T234310403Z-SOURCE-CAPABILITY-PROOF-MISSES-RAW-C-489C7267
title: "Source capability proof misses raw calls in constructor payloads"
area: core
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-16
updated: 2026-05-17
target: nepl-core/src/source_capability/walk.rs
---

# ISS-20260516T234310403Z-SOURCE-CAPABILITY-PROOF-MISSES-RAW-C-489C7267: Source capability proof misses raw calls in constructor payloads

## 概要

Raw memory source capability proof only observes top-level prefix call-head positions. In compiler-owned helpers like core/mem/pointer/scalar.nepl, Option::Some load_u8 raw wraps the raw primitive call as a constructor payload, so the raw operation evidence is missed and the helper is rejected by effect/resource raw-boundary checks.

## 対象

- `nepl-core/src/source_capability/walk.rs`

## 根拠

- `node nodesrc/tests.js -i stdlib/alloc/collections/adjacency_matrix/api/create.nepl --no-tree -o tmp/agent1-adjacency-create-doc-after-trunk.json -j 1 --dist web/dist --assert-io` が `/stdlib/core/mem/pointer/scalar.nepl:87:31` の `load_u8 raw` で `effect.pure.calls_impure` / `resource.raw.memory_outside_boundary` を出した。
- 問題の source は `Option<i32>::Some load_u8 raw` であり、raw primitive call が constructor payload の先頭にある。既存 walker は expression 先頭など `PrefixCallHead` が認める位置だけを `observe_call_head_symbol` へ渡すため、この nested call を evidence として見ていなかった。

## 問題

Raw memory source capability proof only observes top-level prefix call-head positions. In compiler-owned helpers like core/mem/pointer/scalar.nepl, Option::Some load_u8 raw wraps the raw primitive call as a constructor payload, so the raw operation evidence is missed and the helper is rejected by effect/resource raw-boundary checks.

## 影響

Compiler-owned raw helpers that return Option/Result can be rejected even when their source contains the exact raw primitive operation. This blocks stdlib doctests and exposes that SourceCapabilities are not proving nested prefix calls generically.

## 修正方針

Teach the shared source capability walker to observe payload-leading nested calls when an identifier has a following payload, while preserving shadow checks and the local/value non-call regression.

## 検証

Add loader regressions for raw primitive calls inside constructor payloads, run raw_memory_boundary tests, static-check source policy, trunk build, and the adjacency_matrix create doctest.

## 対応

- shared source capability walker で、従来の prefix call-head 位置に加え、後続 payload を持つ identifier も payload-leading nested call evidence として `observe_call_head_symbol` に渡すようにした。
- shadow / current function / raw helper registry による raw evidence gate は既存の `proof.rs` / `raw_evidence_gate.rs` 側で維持し、`consume load_i32` のような payload を持たない値参照 regression は引き続き拒否する。
- `Option::Some load_u8 raw` 形の loader regression を追加し、constructor payload 内の raw primitive call を source proof として固定した。

## 検証結果

- `cargo fmt -p nepl-core --check`
- `node nodesrc/test_static_check_boundary_responsibility.js`
- `cargo test -p nepl-core raw_memory_boundary_accepts_raw_helper_call_in_constructor_payload -- --nocapture`
- `cargo test -p nepl-core raw_memory_boundary_requires_raw_operation_call_head -- --nocapture`
- `cargo check -p nepl-core`
- `trunk build`
- `node nodesrc/tests.js -i stdlib/alloc/collections/adjacency_matrix/api/create.nepl --no-tree -o tmp/agent1-adjacency-create-doc-after-payload-proof.json -j 1 --dist web/dist --assert-io`
- `node nodesrc/run_doctest.js -i stdlib/alloc/collections/adjacency_matrix/api/create.nepl -n 1 --dist web/dist`
