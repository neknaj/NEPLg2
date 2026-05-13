---
id: ISS-20260513T101719832Z-DEALLOC-AND-REALLOC-SIZE-ARGUMENTS-N-D7EADBBD
title: "dealloc and realloc size arguments need owner extent proof"
area: core
status: fixed
resolved: true
priority: P0
type: architecture
created: 2026-05-13
updated: 2026-05-13
target: "nepl-core/src/resource/**, stdlib/core/mem/allocator.nepl, tests/compiler/*.n.md, tests/stdlib/memory_safety.n.md"
---

# ISS-20260513T101719832Z-DEALLOC-AND-REALLOC-SIZE-ARGUMENTS-N-D7EADBBD: dealloc and realloc size arguments need owner extent proof

## 概要

While hardening allocator payload overflow, adding runtime size upper-bound checks to dealloc/realloc made Resource IR report owner leaks because the current owner summary cannot express that a deallocation consumes storage only when its size argument is proven to match the allocated extent. The stdlib API also does not statically prove that dealloc/realloc sizes correspond to the allocation extent.

## 対象

- `nepl-core/src/resource/**, stdlib/core/mem/allocator.nepl, tests/compiler/*.n.md, tests/stdlib/memory_safety.n.md`

## 根拠

- `alloc_raw` / `alloc_region` の overflow hardening 中に、`dealloc` / `realloc` へ allocator payload 上限の runtime check を追加すると、既存の `tests/stdlib/memory_safety.n.md` が `resource.owner.leak` / `resource.owner.no_free_obligation` を出した。
- これは単なる stdlib 条件式の問題ではなく、Resource IR owner summary が「この size 引数なら free obligation を消費する」という allocation extent 証明を持っていないためである。
- 現状の API は `MemPtr` と size を別々に受け取るため、利用者が確保時と異なる size を渡しても型だけでは検査できない。

## 関連ドキュメント

- [NEPLg2 静的検査の複雑化解消計画](../../doc/neplg2/static_check_complexity_reduction_plan.md)
- 発見元: [ISS-20260513T101054155Z-CORE-MEM-ALLOCATION-BYTE-COUNTS-CAN--9B7BDEA4](./ISS-20260513T101054155Z-CORE-MEM-ALLOCATION-BYTE-COUNTS-CAN--9B7BDEA4.md)

## 問題

While hardening allocator payload overflow, adding runtime size upper-bound checks to dealloc/realloc made Resource IR report owner leaks because the current owner summary cannot express that a deallocation consumes storage only when its size argument is proven to match the allocated extent. The stdlib API also does not statically prove that dealloc/realloc sizes correspond to the allocation extent.

## 影響

If stdlib simply accepts arbitrary size arguments, an invalid size can corrupt allocator free-list metadata. If stdlib rejects sizes dynamically without compiler extent reasoning, the owner checker correctly sees paths where the free obligation is not consumed. This needs a compiler-level owner extent proof rather than ad hoc runtime checks.

## 修正方針

Extend Resource IR owner metadata to carry allocation extents or a checked dealloc-size proof, then update dealloc/realloc wrappers to reject mismatched or overflowing sizes without creating maybe-leak false positives. Keep allocator allocation overflow checks separate until this proof exists.

## 対応内容

- `OwnerState::Live` に allocation extent を持たせ、`alloc_raw` の size 引数を owner の payload byte extent として記録した。
- `dealloc_raw` / `realloc_raw` は、free obligation の owner extent と渡された size / old_size が一致することを Resource IR で証明してから owner を解放または再確保するようにした。
- 証明不能な関数境界では、owner summary に extent requirement を保存し、`dealloc_ptr` / `realloc_ptr` などの wrapper 越しでも呼び出し側の owner extent と照合するようにした。
- `realloc` 成功時は旧 owner の extent requirement と返却後 owner の new_size extent を分離し、parameter-return summary が返却後 extent を明示的に持つ設計へ変更した。
- variant path summary の再帰処理で `owner_extent_requirements` を破棄していた問題を修正し、`realloc_raw` の旧サイズ前提が nested `Result::Ok` 構築まで届くようにした。
- 既存の `realloc` 分岐 refinement 回帰テストが `lt` import 不足で停止していたため、Resource IR owner 回帰として実行できるように test source import を補った。

## 検証

- `cargo check -p nepl-core`
- `cargo test -p nepl-core --test resource_ir raw_dealloc_extent -- --nocapture`
- `cargo test -p nepl-core --test resource_ir raw_realloc_old_extent -- --nocapture`
- `cargo test -p nepl-core --test resource_ir dealloc_ptr_extent -- --nocapture`
- `cargo test -p nepl-core --test resource_ir realloc_ptr_old_extent -- --nocapture`
- `cargo test -p nepl-core --test resource_ir resource_ir_owner_check_accepts_deallocated_alloc -- --nocapture`
- `cargo test -p nepl-core --test resource_ir resource_ir_owner_check_reports_double_dealloc -- --nocapture`
- `cargo test -p nepl-core --test resource_ir resource_ir_owner_check_refines_realloc_result_branches -- --nocapture`
