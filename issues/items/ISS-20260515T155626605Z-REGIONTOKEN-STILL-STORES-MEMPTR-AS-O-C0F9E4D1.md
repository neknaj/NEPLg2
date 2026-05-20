---
id: ISS-20260515T155626605Z-REGIONTOKEN-STILL-STORES-MEMPTR-AS-O-C0F9E4D1
title: "RegionToken still stores MemPtr as owner-like field"
area: stdlib
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-05-15
updated: 2026-05-20
target: "stdlib/core/mem/types.nepl; stdlib/core/mem/internal.nepl; stdlib/core/mem/pointer/region.nepl; nepl-core/src/resource/lower_raw_address*.rs; nodesrc/test_stdlib_memptr_owner_field_policy.js"
---

# ISS-20260515T155626605Z-REGIONTOKEN-STILL-STORES-MEMPTR-AS-O-C0F9E4D1: RegionToken still stores MemPtr as owner-like field

## 概要

RegionToken remains the only direct MemPtr struct field, so the free-obligation owner is still represented by the same non-owning pointer wrapper used for borrowed views.

## 対象

- `stdlib/core/mem/types.nepl; stdlib/core/mem/internal.nepl; stdlib/core/mem/pointer/region.nepl; nepl-core/src/resource/lower_raw_address*.rs; nodesrc/test_stdlib_memptr_owner_field_policy.js`

## 根拠

- `doc/neplg2/static_check_complexity_reduction_plan.md` Stage 6 は `MemPtr = non-owning pointer`、`OwnedRegion/Storage = free obligation owner`、`InitializedCell/Resource IR = initialized/moved/drop state` へ責務を分離する方針である。
- `RegionToken<T>` が `ptr: MemPtr<T>` を持つ限り、source policy は `MemPtr` owner-like field の最後の transitional exception を残す必要があり、`MemPtr` の非所有 pointer 化を型構造から固定できない。
- `dealloc_region<T>` は `RegionToken<T>` の owner を消費する API なので、`RegionToken.raw` へ直接移した後も Resource IR owner summary が `dealloc_ptr<T>` summary 経由の raw owner consumption を拾う必要がある。

## 問題

RegionToken remains the only direct MemPtr struct field, so the free-obligation owner is still represented by the same non-owning pointer wrapper used for borrowed views.

## 影響

Stage 6 cannot make MemPtr a strictly non-owning pointer while RegionToken.ptr is the remaining owner-like MemPtr field; the source policy must keep a transitional exception.

## 修正方針

Store the owner token base address as a compiler-owned raw address field, expose only checked MemPtr projections, update ResourceIR raw-address lowering to use the raw field directly, and remove the MemPtr field-policy exception.

## 対応内容

- `RegionToken<T>` の layout を `raw: i32, size: i32` に変更し、`ptr: MemPtr<T>` owner-like field を削除した。
- `region_new<T>` は `MemPtr<T>` から raw owner identity を取り出して `RegionToken<T>` を構築する内部 helper とし、public projection は `region_ptr<T>(&RegionToken<T>) -> MemPtr<T>` / `region_ptr_at<T,U>` の checked non-owning view に限定した。
- `string_region_len_ptr` / `string_region_data_ptr` / `string_finish` は `RegionToken.raw` を直接 owner identity として扱う形へ更新し、`MemPtr` field への依存を削除した。
- Resource IR lowering / initialized summary / raw-address return summary / raw memory source evidence は `region_token_raw_ref` と direct `RegionToken.raw` field を authority として扱うように更新した。
- owner checker は `MemPtr.raw -> RegionToken.raw` のような wrapper raw field 間の owner move を transfer として扱う。
- owner summary seed は、直接 raw memory op だけでなく callee owner summary が消費する raw owner alias も見るようにし、`dealloc_region -> dealloc_ptr` のような checked helper 経由の free obligation consumption を関数境界に反映する。
- callee summary 経由の raw owner consumption 判定は `owner_summary_raw_use_call.rs` に分離し、Resource checker の責務分割 policy で固定した。
- `nodesrc/test_stdlib_memptr_owner_field_policy.js` の transitional allowlist を 0 件にし、今後 stdlib struct field に `MemPtr` / `Option<MemPtr>` owner-like field が再導入された場合に失敗するようにした。
- `stdlib/core/mem/pointer/region.nepl` の `region_ptr` doctest を追加し、`RegionToken.raw` からの checked non-owning projection が stdout report 付きで監視されるようにした。

## 検証

Focused core/mem doctests, tests/stdlib/memory_safety.n.md, ResourceIR raw-address tests, source policies, and MemPtr owner-field policy must pass with zero transitional MemPtr fields.

実施済み:

- `cargo fmt -p nepl-core`
- `cargo fmt -p nepl-core --check`
- `cargo check -p nepl-core`
- `cargo test -p nepl-core --test resource_ir checked_region -- --nocapture`
- `cargo test -p nepl-core --test resource_ir resource_ir_owner_check_accepts_region_ptr_through_known_identity_callback -- --nocapture`
- `cargo test -p nepl-core --test resource_ir resource_ir_owner_check_rejects_region_token_forged -- --nocapture`
- `cargo test -p nepl-core loader::tests::owner_aggregate_boundary_accepts_intrinsic_field_evidence -- --exact`
- `cargo test -p nepl-core loader::tests::compiler_memory_type_definitions_use_source_shape_not_raw_boundary_evidence -- --exact`
- `node nodesrc/test_stdlib_memptr_owner_field_policy.js`
- `node nodesrc/test_stdlib_core_mem_boundary.js`
- `node nodesrc/test_stdlib_string_no_unsafe_unwraps.js`
- `node nodesrc/test_stdlib_string_storage_boundary.js`
- `node nodesrc/test_resource_checker_responsibility.js`
- `node nodesrc/test_stdlib_documentation_contract.js`: `declarationNoDoctest=1032`
- `node nodesrc/run_source_policy_regressions.js --warn-only`: source-policy warning なし
- `trunk build`
- `node nodesrc/tests.js -i tests/stdlib/memory_safety.n.md --no-tree -o tmp/agent1-regiontoken-raw-memory-safety.json -j 1 --dist web/dist --assert-io`: 38 passed
- `node nodesrc/tests.js -i stdlib/core/mem/pointer/region.nepl --no-tree -o tmp/agent1-regiontoken-region-doctests.json -j 1 --dist web/dist --assert-io`: 11 passed
- `node nodesrc/run_doctest.js -i stdlib/core/mem/types.nepl --assert-io --dist web/dist`
- `node nodesrc/run_doctest.js -i stdlib/core/mem/internal.nepl --assert-io --dist web/dist`
- `node nodesrc/run_doctest.js -i stdlib/core/mem/pointer/region.nepl --assert-io --dist web/dist`

補足: `stdlib/alloc/string/storage.nepl` は該当ファイル内に doctest がないため、source policy 回帰で確認した。

## 2026-05-20 Agent 1 追記

この issue の修正時点では `RegionToken<T>` の field から `MemPtr<T>` を外した一方で、`region_new<T>` の入力はまだ `MemPtr<T>` だった。後続の `ISS-20260520T074855359Z-REGION-NEW-ACCEPTS-NON-OWNING-MEMPTR-10E3BBC9` でその残りも解消し、`region_new<T>` は allocator / realloc 由来の raw owner identity と size だけを受け取る internal boundary になった。

現在の責務分割は次の通りである。

- `RegionToken<T>.raw`: free obligation owner identity。
- `RegionToken<T>.size`: owner extent metadata。raw owner seed にはしない。
- `MemPtr<T>`: `region_ptr` / `region_ptr_at` が返す non-owning projection view。
- Resource IR: `RegionToken.raw` の owner transfer / extent proof / non-owning view rejection を検査する。

この追記により、`RegionToken` から `MemPtr` field を削除しただけでなく、`MemPtr` を owner token construction input として扱う過渡設計も閉じた。
