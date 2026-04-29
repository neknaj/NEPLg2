---
id: ISS-20260429T170036695Z-RESOURCE-OWNER-SUMMARY-KEEPS-OWNED-E-BFDE4F98
title: "Resource owner summary keeps owned enum payloads from non-returning match arms"
area: core
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-04-29
updated: 2026-04-30
target: "nepl-core/src/resource/owner_check.rs, nepl-core/src/resource/owner_summary.rs, nepl-core/tests/resource_ir.rs"
---

# ISS-20260429T170036695Z-RESOURCE-OWNER-SUMMARY-KEEPS-OWNED-E-BFDE4F98: Resource owner summary keeps owned enum payloads from non-returning match arms

## 概要

Resource owner return summaries skip owner leaves that become NoFreeObligation on all returning paths. Functions such as unwrap_ok consume Result<T,E> by value and make the Err arm unreachable, but caller-side summary application transfers the Ok payload while leaving an owned Err payload from the argument live. This appears as resource.raw.ownership_violation on Result::Err Diag payloads in collection integration tests.

## 対象

- `nepl-core/src/resource/owner_check.rs`
- `nepl-core/src/resource/owner_summary.rs`
- `nepl-core/tests/resource_ir.rs`

## 根拠

- `cargo test -p nepl-core --test neplg2 -- --nocapture` の `list_get_out_of_bounds_err` は、`unwrap_ok` / `uwok` 経由で `Result::Err Diag` payload の `Diag.message` owner が `main` に残ると診断していた。
- `unwrap_ok` は `Result<T,E>` を by-value で受け、`Err e` arm は `#intrinsic "unreachable"` で戻らない。戻る path は `Ok v` だけなので、caller 側では `Err` payload owner も call 引数として消費済みにする必要がある。
- Resource IR lowering では match scrutinee が parameter read alias になるため、inactive enum payload を `NoFreeObligation` にする処理が alias 元の parameter place へ届いていなかった。
- さらに owner return summary は `NoFreeObligation` になった parameter owner leaf を consumed source として記録していなかった。

## 問題

Resource owner return summaries skip owner leaves that become NoFreeObligation on all returning paths. Functions such as unwrap_ok consume Result<T,E> by value and make the Err arm unreachable, but caller-side summary application transfers the Ok payload while leaving an owned Err payload from the argument live. This appears as resource.raw.ownership_violation on Result::Err Diag payloads in collection integration tests.

## 影響

Valid code that unwraps fallible constructors can be rejected, and developers may incorrectly weaken owner checking or patch stdlib call sites instead of fixing the function-boundary ownership summary.

## 修正方針

Record NoFreeObligation parameter owner leaves as consumed by by-value calls when they are not returned, then apply that consumed projection summary at call sites. Add a Resource IR regression with an owned Err payload and an unwrap-style non-returning arm.

## 修正結果

- `ResourceOp::Match` の inactive enum payload 処理で、scrutinee 自身だけでなく owner alias 解決後の scrutinee place に対しても sibling payload を `NoFreeObligation` にするようにした。
- owner return summary 生成時に、returned source ではない parameter owner leaf が `NoFreeObligation` になっている場合も consumed parameter source として記録するようにした。
- これにより `unwrap_ok` 形式の helper は、返した `Ok` payload だけを caller output へ移し、戻らない `Err` arm の owned payload は caller 側で消費済みとして処理できる。
- `Result<Boxed, OwnedErr>` の `Err` payload に raw owner を持たせた Resource IR regression を追加し、`Err => unreachable` の call boundary で owner が残らないことを固定した。

## 検証

- `cargo test -p nepl-core --test resource_ir resource_ir_owner_summary_consumes_owned_err_payload_from_unreachable_arm -- --nocapture`: passed
- `cargo test -p nepl-core --test resource_ir resource_ir_owner_summary -- --nocapture`: 2 passed
- `cargo test -p nepl-core --test resource_ir resource_ir_owner_check_moves_result_payload_field_owner_to_match_bind -- --nocapture`: passed
- `cargo test -p nepl-core --test neplg2 list_get_out_of_bounds_err -- --nocapture`: passed
- `trunk build`: passed
- `node nodesrc/tests.js -i tests/compiler/move_effect.n.md --no-tree -o tmp/resource-owner-noreturn-move-effect.json -j 1 --dist web/dist`: total=110, passed=110
- `cargo test -p nepl-core --test neplg2 hashmap_custom_struct_key_roundtrips_value -- --nocapture`: still fails with HashMap header/entries owner leaks, tracked separately.
- `cargo test -p nepl-core --test neplg2 llvm_hashmap_string_key_preserves_explicit_hasher_type_args -- --nocapture`: still fails with the same HashMap owner contract problem, tracked separately.
