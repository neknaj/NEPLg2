---
id: ISS-20260514T153421179Z-SOURCETEXT-LINE-MAP-POLICY-STILL-EXP-597F282C
title: "SourceText line map policy still expects pre-VecPushError empty owner"
area: selfhost
status: fixed
resolved: true
priority: P1
type: test
created: 2026-05-14
updated: 2026-05-15
target: "stdlib/neplg2/core/infra/text.nepl, nodesrc/test_selfhost_source_text_no_recursive_line_map.js"
---

# ISS-20260514T153421179Z-SOURCETEXT-LINE-MAP-POLICY-STILL-EXP-597F282C: SourceText line map policy still expects pre-VecPushError empty owner

## 概要

The SourceText source policy still requires source_text_push_line_start to replace Vec::push failure with vec_empty, but Vec::push now returns VecPushError<T> carrying the original Vec owner. The focused SourceText doctest also lacks an explicit core/math import for eq, so it no longer executes under current doctest isolation.

## 対象

- `stdlib/neplg2/core/infra/text.nepl, nodesrc/test_selfhost_source_text_no_recursive_line_map.js`

## 根拠

- `nodesrc/run_source_policy_regressions.js --warn-only` が `nodesrc/test_selfhost_source_text_no_recursive_line_map.js` の失敗を報告した。
- 現行 `Vec::push` は `Result<Vec<T>, VecPushError<T>>` を返し、失敗時も入力 `Vec<T>` owner を `VecPushError.vec` に保持する設計へ変更済みである。
- `stdlib/neplg2/core/infra/text.nepl` の実装は `v::vec_push_error_vec<i32> e` を返して caller 側で `v::free<i32> out` するため、古い policy が要求する `v::vec_empty<i32>` は現在の owner-preserving API contract と衝突していた。
- focused doctest は `eq` を使うが `core/math` を import しておらず、`resolve.identifier.undefined` で止まっていた。

## 問題

The SourceText source policy still requires source_text_push_line_start to replace Vec::push failure with vec_empty, but Vec::push now returns VecPushError<T> carrying the original Vec owner. The focused SourceText doctest also lacks an explicit core/math import for eq, so it no longer executes under current doctest isolation.

## 影響

The policy now rejects the safer owner-preserving push error contract and source_text_nepl doctest coverage is broken. This can hide real selfhost line-map regressions behind stale source-policy expectations.

## 修正方針

Update the policy and comments to require the VecPushError owner to be returned and cleaned by the caller, and add the missing doctest import so SourceText coverage runs again.

## 検証

Run the SourceText source policy, focused SourceText doctest, issue check, and diff whitespace check.

## 2026-05-15 Agent 1 修正

`nodesrc/test_selfhost_source_text_no_recursive_line_map.js` を現行 `VecPushError<T>` contract に合わせ、`source_text_push_line_start` の Err branch が `v::vec_push_error_vec<i32> e` から入力 `Vec<i32>` owner を取り戻すことを要求するように更新した。あわせて Err branch で `v::vec_empty<i32>` を返す退行を明示的に拒否する検査を追加した。

`stdlib/neplg2/core/infra/text.nepl` 側は実装がすでに owner-preserving contract に沿っていたため、古い「失敗時は owner を持たない空 Vec」というコメントを修正した。focused doctest には `core/math` の明示 import を追加し、`eq` が doctest isolation でも解決されるようにした。

検証:

- `node nodesrc/test_selfhost_source_text_no_recursive_line_map.js`: passed
- `node nodesrc/tests.js -i stdlib/neplg2/core/infra/text.nepl --no-tree -o tmp/agent1-source-text-push-owner-after.json -j 1 --dist web/dist`: total=1, passed=1
- `node nodesrc/issues.js check --dir issues`: passed
- `git diff --check`: passed
