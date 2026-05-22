---
id: ISS-20260522T073914640Z-VEC-NON-COPY-LIFECYCLE-NEEDS-STORAGE-645ED85D
title: "Vec non-Copy lifecycle needs storage invariant separated from Copy raw-access proof"
area: stdlib
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-05-22
updated: 2026-05-22
target: "stdlib/alloc/collections/vec/invariant.nepl, nodesrc/test_stdlib_vec_no_unsafe_unwraps.js"
---

# ISS-20260522T073914640Z-VEC-NON-COPY-LIFECYCLE-NEEDS-STORAGE-645ED85D: Vec non-Copy lifecycle needs storage invariant separated from Copy raw-access proof

## 概要

VecCopyInvariant currently combines storage metadata/extent validation with Copy-only raw access proof. Non-Copy push/grow/drop lifecycle APIs need to validate len/initialized_len/cap/storage extent without requiring payload Copy.

## 対象

- `stdlib/alloc/collections/vec/invariant.nepl, nodesrc/test_stdlib_vec_no_unsafe_unwraps.js`

## 根拠

- 親 issue: [Non-Copy collection payload support needs compiler-issued owner and drop traversal](./ISS-20260520T152218366Z-NON-COPY-COLLECTION-PAYLOAD-SUPPORT--A6A88543.md)
- 関連 doc: [NEPLg2 静的検査の複雑化解消計画 Stage 6](../../doc/neplg2/static_check_complexity_reduction_plan.md#stage-6-resource-ir-と-stdlib-境界の完成)
- 開発方針: https://zenn.dev/bem130/articles/1b352797de94e7

## 問題

VecCopyInvariant currently combines storage metadata/extent validation with Copy-only raw access proof. Non-Copy push/grow/drop lifecycle APIs need to validate len/initialized_len/cap/storage extent without requiring payload Copy.

## 影響

Keeping the invariant Copy-bound forces future non-Copy Vec APIs either to keep an incorrect Copy dependency or to duplicate unchecked metadata validation at each lifecycle boundary.

## 修正方針

Introduce a generic VecStorageInvariant for payload-independent storage metadata/extent validation, then make VecCopyInvariant a Copy-bound wrapper over that proof. Keep raw payload access APIs Copy-only.

## 検証

Add source policy checks for the generic storage invariant and focused typecheck/Rust tests that Drop-bound Vec storage validation does not require Copy.

## 解決内容

2026-05-22 に Agent 1 が `VecCopyInvariant` から payload 非依存の storage proof を分離した。

- `VecStorageInvariantInvalid` / `VecStorageInvariant` を追加し、`len` / `initialized_len` / `cap` / `VecStorage` / backing extent の相関を `.T: Copy` なしで検査する typed proof にした。
- `vec_buffer_current_storage_invariant<T>` / `vec_current_storage_invariant<T>` を追加し、Drop payload の `Vec<T>` でも storage metadata / extent を raw payload access なしで観測できるようにした。
- `VecCopyInvariant` は削除せず、Copy-only raw access boundary として維持した。内部は `VecStorageInvariant` を `match` で受け、`vec_storage_invalid_to_copy_invalid` の exhaustive mapping を通して Copy raw-access proof へ変換する。
- `nodesrc/test_stdlib_vec_no_unsafe_unwraps.js` は storage proof と Copy raw-access proof の責務分離を source policy として固定し、Copy invariant に metadata/extent 検査が再複製されないことも監査する。
- `nepl-core/tests/resource_ir.rs` に、`Vec<DropPayload>` が `vec_current_storage_invariant` を使っても `vec_buffer_current_copy_invariant` へ流れない regression を追加した。

## 回帰テスト

- `node --check nodesrc/test_stdlib_vec_no_unsafe_unwraps.js`
- `node nodesrc/test_stdlib_vec_no_unsafe_unwraps.js`
- `node nodesrc/test_stdlib_collection_cleanup_contract.js`
- `cargo fmt --check`
- `cargo test -p nepl-core --test resource_ir resource_ir_initialized_check_vec_storage_invariant_accepts_drop_payload_without_copy -- --test-threads=1 --exact --nocapture`
