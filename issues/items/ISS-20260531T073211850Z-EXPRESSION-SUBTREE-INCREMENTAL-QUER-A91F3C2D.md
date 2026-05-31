---
id: ISS-20260531T073211850Z-EXPRESSION-SUBTREE-INCREMENTAL-QUER-A91F3C2D
title: "expression subtree incremental queries need 0.1s warm recompile path"
area: core
status: open
resolved: false
priority: P1
type: performance
created: 2026-05-31
updated: 2026-05-31
target: "nepl-core/src/compiler.rs; nepl-core/src/source_cache_key.rs; nepl-web/src/lib.rs"
---

# ISS-20260531T073211850Z-EXPRESSION-SUBTREE-INCREMENTAL-QUER-A91F3C2D: expression subtree incremental queries need 0.1s warm recompile path

## 概要

リテラル変更を、式木の leaf を別の式へ差し替える一般的な差分コンパイル操作として扱い、同一 warm `CompilerSession` 内で 0.1 秒以下にする。

## 対象

- `nepl-core/src/compiler.rs`
- `nepl-core/src/source_cache_key.rs`
- `nepl-web/src/lib.rs`

## 根拠

- Zenn 方針では、純粋性、依存関係の DAG 化、静的検査、cache により探索範囲と計算量を削減することが要求されている。
- 現在の compiled-output cache は comment-only edit では 10ms 未満へ入るが、実コードの式枝差し替えでは source key が変わり、Resource IR summary / final check / codegen が広く再実行される。
- リテラルは式なので、リテラル専用 cache ではなく typed AST / HIR の expression subtree query として設計する必要がある。

## 問題

現行の warm session cache は、完全同一 output や stdlib parse / Resource summary leaf replay には効くが、entry function 内の小さい式枝を差し替える操作を query 単位で invalidation できない。結果として、公開 surface が変わらない変更でも function body、Resource summary、final check、codegen の再実行範囲が大きく残る。

## 影響

Web playground や terminal worker で、1 文字の数値リテラル変更、small expression replacement、local pure helper call への差し替えが秒単位になり得る。NEPL は前置記法と静的検査により本来探索空間を小さくできるはずなので、このままでは性能方針と不整合になる。

## 修正方針

- `doc/neplg2/compiler_performance_cache_design.md` の「式枝差し替えの 0.1 秒 budget」を実装計画の正とする。
- source text 全体ではなく、function identity、stable lexical path id、subtree semantic hash、expected type boundary、local name scope、callable candidate set、effect expectation を持つ typed expression subtree query を設計する。
- public surface が不変なら dependency module の typecheck / Resource IR / codegen を invalidation しない。
- Resource IR 以降は function body hash と dependency closure hash で変更 function と dependent summary だけを再計算する。
- codegen fragment を function hash 単位に分け、unchanged fragment は signature table / link order だけ再接続する。

## 検証

- warm `CompilerSession` で RPN のリテラル変更、同じ expected type の式枝差し替え、local pure helper call への差し替えを測定し、0.1 秒以下を確認する。
- public signature edit、local binding shape edit、source capability policy edit、unresolved indirect call は fail-closed に広い invalidation へ戻ることを確認する。
- typed diagnostic enum と source span が現在 source map へ再投影できない場合に stale diagnostic を出さないことを確認する。
