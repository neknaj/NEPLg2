---
id: ISS-20260518T052853094Z-USER-SOURCE-CAN-FORGE-RAW-MEMPTR-THR-109B2F18
title: "user source can forge raw MemPtr through core/mem/internal helpers"
area: core
status: fixed
resolved: true
priority: P0
type: bug
created: 2026-05-18
updated: 2026-05-18
target: "nepl-core/src/resource/model.rs, nepl-core/src/resource/effect_check.rs, nepl-core/src/resource/lower_raw_address.rs, nepl-core/src/source_capability/raw_builtin_evidence.rs, nepl-core/src/compiler.rs, tests/compiler/intrinsic.n.md, stdlib/core/mem/internal.nepl"
---

# ISS-20260518T052853094Z-USER-SOURCE-CAN-FORGE-RAW-MEMPTR-THR-109B2F18: user source can forge raw MemPtr through core/mem/internal helpers

## 概要

Ordinary user source can import core/mem/internal and call mem_ptr_wrap or mem_ptr_addr. Resource IR lowers these calls to RawAddressAlias or NonOwningProjection, but the effect boundary only reports raw memory operations and MemPtrOffset views, so raw pointer identity can be forged or observed outside a compiler-owned raw-memory boundary.

## 対象

- `nepl-core/src/resource/model.rs, nepl-core/src/resource/effect_check.rs, nepl-core/src/resource/lower_raw_address.rs, nepl-core/src/source_capability/raw_builtin_evidence.rs, nepl-core/src/compiler.rs, tests/compiler/intrinsic.n.md, stdlib/core/mem/internal.nepl`

## 根拠

- 未記入

## 問題

Ordinary user source can import core/mem/internal and call mem_ptr_wrap or mem_ptr_addr. Resource IR lowers these calls to RawAddressAlias or NonOwningProjection, but the effect boundary only reports raw memory operations and MemPtrOffset views, so raw pointer identity can be forged or observed outside a compiler-owned raw-memory boundary.

## 影響

The compiler claims raw memory authority is proven from source capability evidence, but raw address helper calls can bypass that proof. This weakens memory safety and lets user code manufacture MemPtr/RegionToken related raw identities that later checks may treat as legitimate.

## 修正方針

Encode internal raw address helper authority in Resource IR using enum variants for alias/view boundary kinds, report those diagnostics from the effect gate, and allow them only through exact source capability proof from compiler-owned stdlib source. Keep checked public projection helpers separate from internal raw helper primitives.

## 検証

Add compile_fail regression for user import of core/mem/internal mem_ptr_wrap/mem_ptr_addr and run focused Resource IR/effect gate tests plus source policy checks.

## 関連ドキュメント

- [NEPLg2 静的検査の複雑化解消計画](../../doc/neplg2/static_check_complexity_reduction_plan.md)

## 2026-05-18 Agent 1 解決内容

Resource IR の raw address 生成を `RawAddressAliasKind` と `RawAddressViewKind` で分離した。`mem_ptr_wrap` / `region_new` は `InternalHelper` の raw address alias、`mem_ptr_addr` / `region_token_raw_ref` / `str_addr` は `InternalHelper` の raw address view として記録し、checked public projection である `region_ptr` / `region_ptr_at` は `NonOwningProjection` のまま残す。

effect gate は `InternalHelper` alias/view だけを raw-memory boundary diagnostic にし、exact use-site source proof がある compiler-owned stdlib source でのみ抑止する。これにより ordinary source が `core/mem/internal` を import して raw pointer identity を作成・観測する経路を閉じる一方、checked `RegionToken -> MemPtr` projection は raw operation authority なしに継続できる。

Source capability proof は `RawAddressAliasBoundary` と `RawAddressViewBoundary` を別 fact として扱う。`raw_builtin_evidence.rs` を追加し、structural / view / alias / operation evidence の収集を一箇所に集約した。stdlib helper 名の allowlist ではなく、source 内の helper use-site と compiler-owned provenance から capability を証明する。

`tests/compiler/intrinsic.n.md` と `stdlib/core/mem/internal.nepl` には ordinary doctest source から `mem_ptr_wrap` / `mem_ptr_addr` を呼ぶ compile-fail regression を追加した。`core/mem/internal` の直接成功例は doctest harness では通常利用者 source として実行されるため、Stage 6 の境界設計と矛盾しない compile-fail 例へ更新した。

検証:

- `cargo fmt --all --check`
- `cargo check -p nepl-core --tests`
- `cargo test -p nepl-core raw_address_alias -- --nocapture`
- `cargo test -p nepl-core raw_memory_boundary_accepts_raw_address_alias_helper_evidence -- --nocapture`
- `cargo test -p nepl-core raw_memory_boundary_rejects_owner_constructor_helper_as_address_view_evidence -- --nocapture`
- `node nodesrc/test_static_check_boundary_responsibility.js`
- `trunk build`
- `node nodesrc/tests.js -i tests\compiler\intrinsic.n.md --no-tree -o tmp\agent1-internal-raw-helper-intrinsic-boundary.json -j 1 --dist web\dist --assert-io`: total=10, passed=10
- `node nodesrc/tests.js -i stdlib\core\mem\internal.nepl --no-tree -o tmp\agent1-core-mem-internal-boundary.json -j 1 --dist web\dist --assert-io`: total=4, passed=4

補足: focused verification 中に `stdlib/core/mem/pointer` と `compile_accepts_checked_mem_ptr_wrapper_from_region_provenance` が `resource.raw.memory_outside_boundary` で失敗することを確認した。この失敗は `origin/main` の `7541cb62` でも再現するため、本 issue の raw internal helper gate 変更による新規 regression ではない。checked `MemPtr` provenance regression として別 issue で追跡する。
