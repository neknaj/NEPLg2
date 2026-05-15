---
id: ISS-20260515T115250172Z-RESOURCE-OWNER-SUMMARY-LOSES-NESTED--28CFC4D8
title: "Resource owner summary loses nested owner payload through result unwrapping helper"
area: core
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-15
updated: 2026-05-15
target: "nepl-core/src/resource/**, stdlib/alloc/collections/btreemap/**, stdlib/alloc/collections/btreeset/**"
---

# ISS-20260515T115250172Z-RESOURCE-OWNER-SUMMARY-LOSES-NESTED--28CFC4D8: Resource owner summary loses nested owner payload through result unwrapping helper

## 概要

A helper that matches Result<BTreeMap, BTreeMapInsertError> or Result<BTreeSet, BTreeSetInsertError> and returns the collection owner from either Ok or Err is rejected with resource.owner.leak/maybe_leak, even though the same match in the caller is accepted. This indicates ResourceIR owner summaries do not fully preserve nested owner projections through helper returns from owner-bearing enum payloads.

## 対象

- `nepl-core/src/resource/**, stdlib/alloc/collections/btreemap/**, stdlib/alloc/collections/btreeset/**`

## 根拠

- 2026-05-15 の `BTreeMapInsertError` / `BTreeSetInsertError` 導入中に、呼び出し側で直接 `match insert ...` して `Ok` / `Err.owner` のどちらかを返す最小ケースは ResourceIR を通過した。
- 同じ分岐を `fn must_map(Result<BTreeMap<i32,i32>, BTreeMapInsertError<i32,i32>>) -> BTreeMap<i32,i32>` / `fn must_set(...) -> BTreeSet<i32>` に閉じると、caller 側で `resource.owner.leak` または `resource.owner.maybe_leak` が出た。
- `VecPushError<T>` の単純 owner payload helper は通過するため、問題は owner-bearing Result 一般ではなく、`BTreeMapStorage -> Vec<Option<T>> -> RegionToken<T>` のような入れ子 owner projection を helper summary が保持しきれていない可能性が高い。

## 問題

A helper that matches Result<BTreeMap, BTreeMapInsertError> or Result<BTreeSet, BTreeSetInsertError> and returns the collection owner from either Ok or Err is rejected with resource.owner.leak/maybe_leak, even though the same match in the caller is accepted. This indicates ResourceIR owner summaries do not fully preserve nested owner projections through helper returns from owner-bearing enum payloads.

## 影響

Safe reusable helpers for owner-preserving fallible updates cannot be written reliably for nested owner aggregates. Users are forced to inline matches or use unwrap-style success helpers, which weakens maintainability and hides a ResourceIR composition gap.

## 修正方針

Extend Resource owner return/variant summaries so nested owner projections returned from matched enum payloads are propagated through helper function summaries, not only direct caller matches. Add regression tests using BTreeMapInsertError/BTreeSetInsertError-style nested owners.

## 解決

2026-05-15 に `PendingVariantOwnerEffects::materialize_return_owner_for_target` を追加し、callee の owner return summary が参照する引数または引数 projection が、未解決の `Result` payload owner return target である場合に、その pending variant owner を先に ResourceIR owner state へ materialize するようにした。

これにより、`insert` の戻り値を `must_map` / `must_set` のような helper へ渡したときも、`Result::Ok` payload と `Result::Err` 内の `owner` payload の双方から collection owner が caller summary へ正しく伝播する。materialize 済みの source owner に対応する pending return / consumption は削除し、同じ source owner を helper summary 適用後に再消費して leak / maybe_leak を出す stale pending entry を残さない。

## 検証

- `cargo test -p nepl-core --test resource_ir resource_ir_owner_check_transfers_nested_btree_insert_error_owner_through_helper -- --exact`
- `cargo test -p nepl-core --test resource_ir resource_ir_owner_check_moves_result_payload_field_owner_to_match_bind -- --exact`
- `cargo test -p nepl-core --test resource_ir resource_ir_owner_check_vec_push_error_owner_does_not_leak_through_result_err -- --exact`
- `cargo test -p nepl-core --test resource_ir resource_ir_owner_check_recursive_vec_result_err_does_not_keep_inactive_ok_owner -- --exact`
- `cargo test -p nepl-core --test resource_ir resource_ir_owner_check_preserves_branch_result_from_owner_returning_helper -- --exact`
- `cargo test -p nepl-core --test resource_ir resource_ir_owner_check_prefers_live_return_owner_over_moved_source_alias -- --exact`
- `cargo test -p nepl-core --test resource_ir resource_ir_owner_check_does_not_reconsume_unconditional_variant_argument -- --exact`
- `trunk build`
- `node nodesrc/tests.js -i stdlib/tests/btreemap.n.md -i stdlib/tests/btreeset.n.md --no-tree -o tmp/agent1-btree-helper-summary-doctests.json -j 1 --dist web/dist --assert-io`: total=10, passed=10
- `node nodesrc/tests.js -i tests/stdlib/btree_array_cost.n.md -i tests/stdlib/pipe_collections.n.md --no-tree -o tmp/agent1-btree-helper-summary-alias-doctests.json -j 1 --dist web/dist --assert-io`: total=14, passed=14
