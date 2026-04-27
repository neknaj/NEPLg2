---
id: ISS-20260427T174539616Z-FUNCTION-PARAMETERS-WITH-DROP-ARE-NO-E6B629C5
title: "function parameters with Drop are not auto dropped at scope exit"
area: core
status: open
resolved: false
priority: P1
type: bug
created: 2026-04-28
updated: 2026-04-28
target: "nepl-core/src/passes/drop_insertion.rs, tests/compiler/drop.n.md"
---

# ISS-20260427T174539616Z-FUNCTION-PARAMETERS-WITH-DROP-ARE-NO-E6B629C5: function parameters with Drop are not auto dropped at scope exit

## 概要

Drop capability を持つ値が関数 parameter として渡された場合、parameter binding 自体が scope end の auto drop 対象に入らない。callback が payload を受け取るだけで本文内の local に移さない場合、destructor が走らず所有 resource が漏れる。

## 対象

- `nepl-core/src/passes/drop_insertion.rs, tests/compiler/drop.n.md`

## 根拠

- `stdlib/neplg2/core/infra/outcome.nepl` の payload cleanup callback 調査中、`fn drop_counter_discard <(DropCounter)*>()> (value): ()` では `DropCounter.drop` が実行されなかった。
- 同 callback 内で `let owned <DropCounter> value` と parameter を local へ移すと、scope end の auto drop により destructor が 1 回実行された。
- つまり現行 `drop_insertion` は local binding を drop 対象にする一方、function parameter binding を同じ resource として登録していない可能性が高い。

## 問題

Drop capability を持つ値が関数 parameter として渡された場合、parameter binding 自体が scope end の auto drop 対象に入らない。callback が payload を受け取るだけで本文内の local に移さない場合、destructor が走らず所有 resource が漏れる。

## 影響

cleanup callback や visitor API が by-value payload を受け取って破棄する設計で leak になる。SelfhostOutcome の payload cleanup callback でも、callback parameter を local に移さないと Drop が実行されないことを確認した。

## 修正方針

drop_insertion が top-level / nested function の parameter binding も local と同じ resource として登録し、move 済み parameter を除いて scope exit で drop elaboration する。

## 検証

Drop 実装が observable side effect を持つ型を parameter として受け取る関数を追加し、関数本文が parameter を使わなくても scope exit で destructor が 1 回だけ走る regression を追加する。
