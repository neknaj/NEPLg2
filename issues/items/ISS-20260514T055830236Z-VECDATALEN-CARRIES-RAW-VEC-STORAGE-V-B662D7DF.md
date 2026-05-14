---
id: ISS-20260514T055830236Z-VECDATALEN-CARRIES-RAW-VEC-STORAGE-V-B662D7DF
title: "VecDataLen carries raw Vec storage view in a struct field"
area: stdlib
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-05-14
updated: 2026-05-14
target: "stdlib/alloc/collections/vec/types.nepl, stdlib/alloc/collections/vec/access/data.nepl, stdlib/neplg2/**, tests/stdlib/*.n.md, nodesrc/test_stdlib_memptr_owner_field_policy.js"
---

# ISS-20260514T055830236Z-VECDATALEN-CARRIES-RAW-VEC-STORAGE-V-B662D7DF: VecDataLen carries raw Vec storage view in a struct field

## 概要

VecDataLen<T> packages Vec.data MemPtr<T> and len into a public struct. Even though data_len is Copy-only, the struct keeps a raw storage view as a field and remains one of the MemPtr owner-field migration exceptions.

## 対象

- `stdlib/alloc/collections/vec/types.nepl, stdlib/alloc/collections/vec/access/data.nepl, stdlib/neplg2/**, tests/stdlib/*.n.md, nodesrc/test_stdlib_memptr_owner_field_policy.js`

## 根拠

- [NEPLg2 静的検査の複雑化解消計画](../../doc/neplg2/static_check_complexity_reduction_plan.md) Stage 6 は、`MemPtr` を non-owning pointer に限定し、storage owner / initialized cell / drop obligation を別表現に分ける方針を定めている。
- [stdlib collection/mem/string と静的検査の安全設計](../../doc/neplg2/stdlib_collection_mem_string_static_safety_design.md) の MemPtr owner-field policy は、`MemPtr` field を安全証明として認めず、移行対象として固定する。

## 問題

VecDataLen<T> packages Vec.data MemPtr<T> and len into a public struct. Even though data_len is Copy-only, the struct keeps a raw storage view as a field and remains one of the MemPtr owner-field migration exceptions.

## 影響

Stage 6 still exposes Vec raw storage identity through a reusable aggregate instead of keeping raw views as short-lived internal observations. Self-host and diagnostics code can depend on field projection of raw storage views.

## 修正方針

Remove VecDataLen and data_len, update callers to request len and data_mem_ptr separately, and lower the MemPtr owner-field migration baseline without adding compatibility aliases.

## 対応

- `VecDataLen<T>` struct と `data_len<T>` observer を削除した。互換 alias は残していない。
- self-host CLI args parser、diag renderer、diag error helper、KP prefix helper、sort/trait fixtures は `len<T>(&Vec<T>)` と `data_mem_ptr<T>(&Vec<T>)` の明示的な別観測へ移した。
- `nodesrc/test_stdlib_memptr_owner_field_policy.js` の transitional baseline から `VecDataLen.data` を削除し、残件を 5 field に下げた。
- 旧 `data_len` を前提にしていた doctest は、safe `Vec` observer と所有権解放を明示する現在の書き方へ更新した。

## 検証

Run the MemPtr owner-field policy, Vec source policy, focused doctests using the former data_len API, and issue checks.

- `node nodesrc/test_stdlib_memptr_owner_field_policy.js`: pass
- `node nodesrc/test_stdlib_vec_no_unsafe_unwraps.js`: pass
- `node nodesrc/test_stdlib_vec_borrowed_observers.js`: pass
- `node nodesrc/test_selfhost_cli_args_no_owner_field_reads.js`: pass
- `node nodesrc/tests.js -i stdlib/alloc/collections/vec/access/data.nepl -i stdlib/neplg2/cli/args/parse.nepl -i stdlib/alloc/diag/diag.nepl -i stdlib/alloc/diag/error/diags.nepl -i stdlib/kp/kpprefix.nepl -i tests/stdlib/traits_order.n.md -i tests/stdlib/sort.n.md --no-tree -o tmp/agent1-vecdatalen-focused.json -j 1 --dist web/dist`: 33 passed
- `node nodesrc/issues.js check --dir issues`: pass
- `git diff --check`: pass
