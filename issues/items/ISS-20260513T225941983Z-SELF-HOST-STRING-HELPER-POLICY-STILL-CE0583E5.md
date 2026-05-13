---
id: ISS-20260513T225941983Z-SELF-HOST-STRING-HELPER-POLICY-STILL-CE0583E5
title: "self-host string helper policy still requires lexer Vec.data raw read"
area: TEST
status: fixed
resolved: true
priority: P1
type: test
created: 2026-05-13
updated: 2026-05-13
target: "nodesrc/test_selfhost_string_helpers_boundary.js, stdlib/neplg2/core/syntax/lexer.nepl"
---

# ISS-20260513T225941983Z-SELF-HOST-STRING-HELPER-POLICY-STILL-CE0583E5: self-host string helper policy still requires lexer Vec.data raw read

## 概要

self-host string helper source policy still asserts that lex_stack_drop_top reads Vec.data and reconstructs Vec directly. That contradicts the Stage 6 policy that self-host code must use public Vec owner APIs and must not depend on transitional MemPtr storage layout.

## 対象

- `nodesrc/test_selfhost_string_helpers_boundary.js, stdlib/neplg2/core/syntax/lexer.nepl`

## 根拠

- `nodesrc/test_selfhost_string_helpers_boundary.js` が `lex_stack_drop_top` 内の `field::get stack "data"` と `Vec<i32> ... stack_data` 再構成を正しい形として要求していた。
- `nodesrc/test_selfhost_lexer_raw_mode_directive_enum.js` は同じ関数について direct `Vec.data` read を拒否し、`drop_last<i32>` 経由の public owner API を要求している。
- `stdlib/neplg2/core/syntax/lexer.nepl` の実装は既に `drop_last<i32> stack` へ移行済みであり、古い source policy だけが安全境界と矛盾していた。

## 問題

self-host string helper source policy still asserts that lex_stack_drop_top reads Vec.data and reconstructs Vec directly. That contradicts the Stage 6 policy that self-host code must use public Vec owner APIs and must not depend on transitional MemPtr storage layout.

## 影響

run_source_policy_regressions fails after the safer lexer implementation and can push future agents back toward raw Vec storage field access, weakening the memory-safety boundary.

## 修正方針

Update the policy to reject direct Vec.data field reads and require lex_stack_drop_top to delegate to drop_last<i32>. Keep the obsolete four-field constructor regression as a negative assertion.

## 検証

Run node nodesrc/test_selfhost_string_helpers_boundary.js, node nodesrc/test_selfhost_lexer_raw_mode_directive_enum.js, node nodesrc/run_source_policy_regressions.js --warn-only, node nodesrc/issues.js check --dir issues, and git diff --check.

## 解決内容

`test_selfhost_string_helpers_boundary.js` の `lex_stack_drop_top` 検査を、raw `Vec.data` field read / 手動 `Vec` 再構成の要求から、`drop_last<i32>` public owner API の要求へ更新した。あわせて direct `field::get/get stack "data"` を拒否する negative assertion を追加し、Stage 6 の self-host code が transitional `MemPtr` storage layout へ依存しないことを同じ policy でも監視する。
