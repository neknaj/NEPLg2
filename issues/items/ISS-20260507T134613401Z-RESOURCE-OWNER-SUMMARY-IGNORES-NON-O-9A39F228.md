---
id: ISS-20260507T134613401Z-RESOURCE-OWNER-SUMMARY-IGNORES-NON-O-9A39F228
title: "Resource owner summary ignores non-owning raw views consumed through helper parameters"
area: core
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-07
updated: 2026-05-07
target: "nepl-core/src/resource/owner_return_apply.rs, nepl-core/tests/resource_ir.rs, tests/stdlib/memory_safety.n.md"
---

# ISS-20260507T134613401Z-RESOURCE-OWNER-SUMMARY-IGNORES-NON-O-9A39F228: Resource owner summary ignores non-owning raw views consumed through helper parameters

## 概要

A non-owning raw address view from str_addr can be wrapped with mem_ptr_wrap, packed into RegionToken through region_new, and then passed to dealloc_region. The callee owner summary marks the RegionToken raw projection as consumed, but caller-side consume_call_argument_owner silently ignores actual non-owning views instead of reporting that no free obligation exists.

## 対象

- `nepl-core/src/resource/owner_return_apply.rs, nepl-core/tests/resource_ir.rs, tests/stdlib/memory_safety.n.md`

## 根拠

- `str_addr` 由来の raw `i32` は Resource IR lowering で `RawAddressViewKind::NonOwningProjection` として扱われる。
- `mem_ptr_wrap` と `region_new` は `RawAddressAlias` として raw view を伝播するため、caller 側の `token.ptr.raw` には non-owning view fact が残る。
- `dealloc_region` の function summary は `RegionToken` parameter の raw owner projection を consumed source として持つ。
- しかし `consume_call_argument_owner` は actual source が non-owning raw view の場合に診断を出さず、transferable owner がないため何もしなかった。

## 問題

A non-owning raw address view from str_addr can be wrapped with mem_ptr_wrap, packed into RegionToken through region_new, and then passed to dealloc_region. The callee owner summary marks the RegionToken raw projection as consumed, but caller-side consume_call_argument_owner silently ignores actual non-owning views instead of reporting that no free obligation exists.

## 影響

Safe code can forge a RegionToken-like wrapper around borrowed string storage and route it through a helper that consumes raw ownership, bypassing the MemPtr = non-owning pointer / OwnedRegion = free obligation owner split required by the static check plan.

## 修正方針

When applying consumed owner summary parameters, treat non-owning raw address views and owned-storage origins without transferable owners as resource.owner.no_free_obligation diagnostics instead of no-op. Keep unmanaged literal/fixed raw addresses without tracked provenance behavior unchanged.

## 検証

Add ResourceIR and .n.md compile_fail regressions for str_addr -> mem_ptr_wrap -> region_new -> dealloc_region. Run focused ResourceIR owner test, memory_safety doctests, cargo fmt/check, source policy, and issue validation.

## 関連

- 親 issue: [MemPtr and RegionToken lack compiler owned provenance model](./ISS-20260427T152958303Z-MEMPTR-AND-REGIONTOKEN-LACK-COMPILER-0BC8ECDF.md)
- 関連計画: [NEPLg2 静的検査の複雑化解消計画 Stage 4](../../doc/neplg2/static_check_complexity_reduction_plan.md#stage-4-resource-check-%E3%81%B8%E3%81%AE%E7%A7%BB%E8%A1%8C)

## 対応結果

`consume_call_argument_owner` を caller-side owner summary consumption の専用 module `owner_consumption.rs` へ分離し、callee summary が owner consumption を要求する actual projection について次を検査するようにした。

- actual projection が non-owning raw address view の場合は `OwnerState::NoFreeObligation` として `resource.owner.no_free_obligation` を出す。
- actual projection が owned storage origin を持つが transferable owner を持たない場合も `NoFreeObligation` として拒否する。
- provenance を持たない unmanaged fixed raw address は従来どおり owner summary consumption だけでは拒否しない。

これにより、`str_addr -> mem_ptr_wrap -> region_new -> dealloc_region` は safe source から borrowed string storage を `RegionToken` owner に偽装できず、`MemPtr = non-owning pointer` と `OwnedRegion/Storage = free obligation owner` の境界を Resource IR owner summary 適用時にも維持する。

検証:

- `cargo fmt -p nepl-core --check`
- `cargo check -p nepl-core`
- `cargo test -p nepl-core --test resource_ir resource_ir_owner_check_rejects_region_token_forged_from_str_addr_view -- --nocapture`
- `node nodesrc/run_source_policy_regressions.js --warn-only`
- `trunk build`
- `node nodesrc/tests.js -i tests/stdlib/memory_safety.n.md --no-tree -o tmp/agent1-memory-safety-region-forge.json -j 1 --dist web/dist`: 15 passed
- `node nodesrc/tests.js -i tests/compiler/move_effect.n.md --no-tree -o tmp/agent1-move-effect-region-forge.json -j 1 --dist web/dist`: 110 passed
