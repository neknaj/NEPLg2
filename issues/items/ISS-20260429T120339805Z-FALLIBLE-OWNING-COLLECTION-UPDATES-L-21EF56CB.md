---
id: ISS-20260429T120339805Z-FALLIBLE-OWNING-COLLECTION-UPDATES-L-21EF56CB
title: "Fallible owning collection updates lose input owners on allocation failure"
area: stdlib
status: open
resolved: false
priority: P1
type: bug
created: 2026-04-29
updated: 2026-04-29
target: "stdlib/alloc/collections/list.nepl, stdlib/alloc/collections/hashmap.nepl"
---

# ISS-20260429T120339805Z-FALLIBLE-OWNING-COLLECTION-UPDATES-L-21EF56CB: Fallible owning collection updates lose input owners on allocation failure

## 概要

Resource IR owner checking now tracks raw owners stored inside collection backing memory. list cons/push style APIs and HashMap grow/new paths can consume an owning collection or backing entries, then return Err without returning or freeing all transferred owners on every failure path.

## 対象

- `stdlib/alloc/collections/list.nepl, stdlib/alloc/collections/hashmap.nepl`

## 根拠

- `cargo test -p nepl-core --test resource_ir resource_ir_owner_check_moves_stored_tail_owner_under_new_raw_node -- --nocapture` は pass。Resource IR は raw node field に格納した tail owner を `load_i32` で取り出し、tail と新 node の両方を free できる。
- `cargo test -p nepl-core --test neplg2 list_get_out_of_bounds_err -- --nocapture` は `cons__T_List_T_T__Result_T_E_List_T_T_Diag__imp_i32` の `Result::Ok(List).ptr` owner leak を報告する。allocation failure path が入力 tail owner を返すか解放する契約になっていない。
- `cargo test -p nepl-core --test neplg2 hashmap_custom_struct_key_roundtrips_value -- --nocapture` は `hashmap_new_with_capacity...` の header/entries owner と `hashmap_rehash_to...` の backing entries owner leak を報告する。HashMap grow/new path が途中で獲得または移動した backing owner を全 failure path で閉じていない。
- `cargo test -p nepl-core --test neplg2 llvm_hashmap_string_key_preserves_explicit_hasher_type_args -- --nocapture` も同じ HashMap backing owner contract に依存して失敗する。
- 関連計画: `doc/neplg2/static_check_complexity_reduction_plan.md` Stage 4 Resource check。

## 問題

Resource IR owner checking now tracks raw owners stored inside collection backing memory. list cons/push style APIs and HashMap grow/new paths can consume an owning collection or backing entries, then return Err without returning or freeing all transferred owners on every failure path.

## 影響

Strict memory safety gates cannot pass for List/HashMap integration tests without either weakening owner checking or fixing the stdlib API/implementation. The issue is a real ownership contract problem, not a Resource IR false positive.

## 修正方針

Redesign fallible owning collection APIs so failure paths either preserve and return the original owner, or fully release/roll back every owner consumed before returning Err. Add focused tests for allocation failure paths and successful free traversal.

## 検証

Run focused List/HashMap neplg2 tests and Resource IR owner regressions after the stdlib ownership contract is fixed.
