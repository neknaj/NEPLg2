---
id: ISS-20260515T003514038Z-VEC-SORT-MERGE-SOURCE-POLICY-STILL-E-BD811427
title: "Vec sort/merge source policy still expects raw merge helper re-exports"
area: test
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-05-15
updated: 2026-05-15
target: "nodesrc/test_stdlib_sort_merge_no_unsafe_unwraps.js, stdlib/alloc/collections/vec/sort/merge.nepl, stdlib/alloc/collections/vec/sort/merge/api.nepl, stdlib/alloc/collections/vec/sort/merge/buffer.nepl, stdlib/alloc/collections/vec/sort/merge/range.nepl"
---

# ISS-20260515T003514038Z-VEC-SORT-MERGE-SOURCE-POLICY-STILL-E-BD811427: Vec sort/merge source policy still expects raw merge helper re-exports

## 概要

nodesrc/test_stdlib_sort_merge_no_unsafe_unwraps.js still requires the sort/merge facade to re-export merge/buffer and merge/range, while Stage 6 now keeps raw scratch buffer and raw traversal helpers behind explicit merge submodule imports and exposes only merge/api from the facade.

## 対象

- `nodesrc/test_stdlib_sort_merge_no_unsafe_unwraps.js, stdlib/alloc/collections/vec/sort/merge.nepl, stdlib/alloc/collections/vec/sort/merge/api.nepl, stdlib/alloc/collections/vec/sort/merge/buffer.nepl, stdlib/alloc/collections/vec/sort/merge/range.nepl`

## 根拠

- `stdlib/alloc/collections/vec/sort/merge.nepl` は Stage 6 の public/raw facade split により `merge/api` だけを public re-export し、`merge/buffer` / `merge/range` は raw scratch buffer / raw traversal implementation module として明示 import 境界へ閉じている。
- `nodesrc/test_stdlib_vec_sort_module_split.js` はすでにこの設計を検査している。
- しかし `nodesrc/test_stdlib_sort_merge_no_unsafe_unwraps.js` は古い分割直後の期待を残し、`merge/buffer` / `merge/range` の public re-export を要求していた。

## 問題

nodesrc/test_stdlib_sort_merge_no_unsafe_unwraps.js still requires the sort/merge facade to re-export merge/buffer and merge/range, while Stage 6 now keeps raw scratch buffer and raw traversal helpers behind explicit merge submodule imports and exposes only merge/api from the facade.

## 影響

The source policy warning contradicts the raw/public facade split and could re-open raw MemPtr merge helpers through the ordinary sort/merge facade.

## 修正方針

Update the policy to require sort/merge.nepl to be implementation-free, re-export only merge/api, and assert merge/buffer and merge/range remain non-facade raw-boundary implementation modules with the expected Copy-only helper contracts.

## 検証

Run node nodesrc/test_stdlib_sort_merge_no_unsafe_unwraps.js, node nodesrc/run_source_policy_regressions.js --warn-only, node nodesrc/issues.js check --dir issues, and git diff --check.

## 関連ドキュメント

- [NEPLg2 静的検査の複雑化解消計画 Stage 6](../../doc/neplg2/static_check_complexity_reduction_plan.md#stage-6-stdlib--self-host-buffer-%E7%A7%BB%E8%A1%8C)

## 2026-05-15 Agent 1 解決

`nodesrc/test_stdlib_sort_merge_no_unsafe_unwraps.js` を現行 Stage 6 の raw/public split に合わせた。`sort/merge` facade は実装 body を持たず `merge/api` だけを再公開することを検査し、`merge/buffer` / `merge/range` が public facade から再公開されないことを固定した。

同時に、`merge/buffer` が `sort_buf_get` / `sort_buf_set` の Copy-only scratch buffer raw helper を所有すること、`merge/range` が `sort_merge_range_data<T: Ord&Copy>` を所有すること、`merge/api` が raw traversal を explicit import することを確認対象にした。これにより unsafe unwrap regression policy と facade boundary policy が矛盾しない。

## 2026-05-19 Agent 1 後続設計更新

`ISS-20260519T134548652Z-VEC-MERGE-SORT-RAW-HELPERS-ARE-DIREC-18BA8A0F` で、上記の「explicit raw-boundary implementation module として残す」設計も不十分と判明した。`merge/buffer` / `merge/range` は facade から再公開されなくても ordinary source が direct import できるため、unchecked `MemPtr` helper の public surface が残る。

後続修正では `merge/buffer.nepl` と `merge/range.nepl` を削除し、scratch buffer access と range traversal を `merge/api.nepl` の private helper に統合した。現在の source policy は「facade に再公開しない」だけでなく「direct-importable raw merge helper module を残さない」ことを検査する。

検証:

- `node nodesrc/test_stdlib_sort_merge_no_unsafe_unwraps.js`
- `node nodesrc/run_source_policy_regressions.js --warn-only`
  - sort/merge source policy warning は解消した。
  - 残警告は stdlib documentation contract と kpgraph source policy。documentation contract は既存 issue `ISS-20260514T154316014Z-STDLIB-DOCUMENTATION-CONTRACT-DECLAR-5F916C0F` の範囲であり、kpgraph policy は別 issue として扱う。
- `node nodesrc/issues.js check --dir issues`
- `git diff --check`
