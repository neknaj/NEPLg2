---
id: ISS-20260507T153554496Z-SELFHOST-BUILTIN-SIGNATURES-USE-ERRO-AEFFF7D4
title: "Selfhost builtin signatures use Error placeholder argument slots"
area: selfhost
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-07
updated: 2026-05-08
target: "stdlib/neplg2/core/builtins/prelude.nepl, nodesrc/test_selfhost_builtin_signature_payload.js"
---

# ISS-20260507T153554496Z-SELFHOST-BUILTIN-SIGNATURES-USE-ERRO-AEFFF7D4: Selfhost builtin signatures use Error placeholder argument slots

## 概要

SelfhostBuiltinFunction stores arg0/arg1/arg2 fixed slots and fills unused slots with SelfhostTypeKind::Error while arg_count decides which slots are valid. That keeps invalid payload state in normal builtin records.

## 対象

- `stdlib/neplg2/core/builtins/prelude.nepl`
- `nodesrc/test_selfhost_builtin_signature_payload.js`

## 根拠

- `SelfhostBuiltinFunction` が存在しない引数 slot まで `SelfhostTypeKind` として保持し、未使用 slot に `SelfhostTypeKind::Error` を入れていた。
- 有効な payload は `arg_count` を別に読まないと判断できず、record 単体で invalid state を表現できる構造だった。
- builtin metadata は resolver/checker/codegen が共有する typed registry なので、signature と arity の不一致を構造上作れない model にする必要がある。

## 問題

SelfhostBuiltinFunction stores arg0/arg1/arg2 fixed slots and fills unused slots with SelfhostTypeKind::Error while arg_count decides which slots are valid. That keeps invalid payload state in normal builtin records.

## 影響

Checker and resolver code can accidentally read placeholder argument slots as real type metadata, and builtin arity changes are not forced through variant-specific match handling. This violates the enum-first typed model policy tracked by the selfhost typed IR parent issue.

## 修正方針

Replace fixed argument slots with a SelfhostBuiltinSignature enum whose variants carry only the arguments that exist for that arity, and make arg_count/arg_kind/result accessors match on the signature.

## 検証

Run source policy rejecting placeholder slots, focused selfhost prelude doctest, issues check, and source policy regressions.

## 対応結果

- `SelfhostBuiltinFunction` から `arg0` / `arg1` / `arg2` / `arg_count` / `result` の固定 slot を削除した。
- `SelfhostBuiltinSignature` を追加し、`Unary` / `Binary` / `Ternary` の payload struct が存在する引数と戻り値だけを保持する形にした。
- `alloc` / `dealloc` / `realloc` の registry は `selfhost_builtin_signature_unary` / `binary` / `ternary` を通して構築し、placeholder の `SelfhostTypeKind::Error` を使わない構造にした。
- `selfhost_builtin_function_arg_count` / `arg_kind` / `result_kind` は `builtin.signature` を直接 `match` する形にした。
- `nodesrc/test_selfhost_builtin_signature_payload.js` を追加し、fixed argument slots、numeric arity storage、`SelfhostTypeKind::Error` placeholder、signature accessor の `match` 不足を source policy で拒否する。

## 検証結果

- `node nodesrc/test_selfhost_builtin_signature_payload.js`: passed
- `node nodesrc/test_selfhost_model_no_numeric_kind_tags.js`: passed
- `node nodesrc/tests.js -i stdlib/neplg2/core/builtins/prelude.nepl --no-tree -o tmp/agent1-selfhost-builtin-signature-payload.json -j 1 --dist web/dist`: total=1, passed=1
