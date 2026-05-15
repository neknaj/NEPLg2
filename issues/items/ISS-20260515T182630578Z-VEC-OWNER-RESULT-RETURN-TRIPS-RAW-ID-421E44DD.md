---
id: ISS-20260515T182630578Z-VEC-OWNER-RESULT-RETURN-TRIPS-RAW-ID-421E44DD
title: "Vec owner Result return trips raw identity escape in fs normalize range push"
area: core
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-15
updated: 2026-05-16
target: "nepl-core/src/resource/effect_identity.rs; nepl-core/src/resource/effect_return_escape.rs; nepl-core/tests/resource_ir.rs"
---

# ISS-20260515T182630578Z-VEC-OWNER-RESULT-RETURN-TRIPS-RAW-ID-421E44DD: Vec owner Result return trips raw identity escape in fs normalize range push

## 概要

std/fs/stat.nepl doctest stopped in fs_normalize_range_push with resource.raw.identity_escape because Result<Vec<i32>, i32> owner returns from Vec push were treated as raw internal allocation identity escapes.

## 対象

- `nepl-core/src/resource/effect_identity.rs`
- `nepl-core/src/resource/effect_return_escape.rs`
- `nepl-core/tests/resource_ir.rs`

## 根拠

- `node nodesrc/tests.js -i stdlib/std/fs/stat.nepl --no-tree -o tmp/agent1-fs-stat-region.json -j 1 --dist web/dist --assert-io` が `stdlib/std/fs/stat.nepl::doctest#1` の compile phase で失敗した。
- diagnostic は `error[resource.raw.identity_escape]: pure function 'fs_normalize_range_push__Vec_T_i32_i32_i32__Result_T_E_Vec_T_i32_i32__pure' returns raw address identity from internal Alloc` だった。
- `fs_normalize_range_push` は `Vec<i32>` owner を `v::push` で更新し、`Result<Vec<i32>, i32>` として返す safe typed owner boundary であり、`i32` raw address や `MemPtr<T>` raw pointer を public surface へ返しているわけではない。
- 調査の結果、`RawIdentityTable::groups_with_replaced_prefix` が descendant projection を写した時に aggregate root も同じ raw identity group へ追加していた。これにより `RegionToken.raw` や owner descendant の raw identity が `Vec` aggregate root へ粗く持ち上がり、`Result::Ok(Vec).field0` の `len: i32` まで内部 Alloc/Realloc raw identity と誤認された。
- `Result::Ok(Vec).field0` は public raw address ではなく Vec header の通常 scalar field である。descendant identity を aggregate root へ合成すると、raw pointer arithmetic に由来する identity と通常 scalar field の区別が失われる。

## 問題

std/fs/stat.nepl doctest stopped in fs_normalize_range_push with resource.raw.identity_escape because Result<Vec<i32>, i32> owner returns from Vec push were treated as raw internal allocation identity escapes.

## 影響

Safe filesystem path normalization and any pure API returning owner-protected Vec results can be rejected before the caller reaches the intended memory-safety checks. Weakening identity_escape globally would hide real MemPtr/i32 raw pointer leaks, so the compiler must distinguish Vec/Result owner carriers precisely.

## 修正方針

- `RawIdentityTable::groups_with_replaced_prefix` で descendant projection を target aggregate root へ粗く畳み込む処理を削除し、raw identity を projection 精度で移送する。
- Whole aggregate transfer では `replace_place_prefix` により各 descendant projection を対応する返却先 projection へ写す。aggregate root identity を新しく発明しないため、`Vec.len` のような public scalar field への false positive を作らない。
- return escape 判定は `str` / `RegionToken` の opaque owner と、最終 projection が owner carrier struct そのものを指す場合を区別して扱う。public `i32` / `MemPtr` leaf を返す経路は引き続き escape として拒否する。

## 検証

- `cargo test -p nepl-core --test resource_ir resource_ir_effect_check_accepts_vec_owner_result_return_identity -- --nocapture`: passing
- `cargo test -p nepl-core effect_return_escape -- --nocapture`: passing
- `cargo test -p nepl-core --test resource_ir resource_ir_effect_check_rejects_mem_ptr_return_identity_escape -- --nocapture`: passing
- `cargo test -p nepl-core --test resource_ir compile_accepts_checked_region_pointer_from_region_provenance -- --nocapture`: passing
- `trunk build`: passing
- `node nodesrc/tests.js -i stdlib/std/fs/stat.nepl --no-tree -o tmp/agent1-fs-stat-after-raw-identity-rerun.json -j 1 --dist web/dist --assert-io`: `resource.raw.identity_escape` は再発しない。次の blocker として `resource.owner.maybe_leak` が露出したため、[ISS-20260515T190417577Z-FS-NORMALIZE-OWNER-SUMMARIES-LEAK-VE-510B86A4](./ISS-20260515T190417577Z-FS-NORMALIZE-OWNER-SUMMARIES-LEAK-VE-510B86A4.md) に分離した。
