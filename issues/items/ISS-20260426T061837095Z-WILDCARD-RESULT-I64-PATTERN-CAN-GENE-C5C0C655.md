---
id: ISS-20260426T061837095Z-WILDCARD-RESULT-I64-PATTERN-CAN-GENE-C5C0C655
title: "wildcard Result i64 pattern can generate invalid wasm"
area: compiler
status: open
resolved: false
priority: P2
type: bug
created: 2026-04-26
updated: 2026-04-26
target: nepl-core/src
---

# ISS-20260426T061837095Z-WILDCARD-RESULT-I64-PATTERN-CAN-GENE-C5C0C655: wildcard Result i64 pattern can generate invalid wasm

## 概要

`Result<i64, E>` を `Result::Ok _` で match し、payload を使わず bool / i32 を返すだけの arm を書くと、i64 payload が stack に残ったままになり invalid wasm が生成される場合がある。stdlib numeric overflow の回帰テスト追加中に、`Result::Ok _` arm が単に `false` を返すだけで再現した。

## 対象

- `nepl-core/src`

## 根拠

- `tests/stdlib/string_numeric_overflow.n.md` の作成中、`to_i64 "9223372036854775808"` を `match` し、`Result::Ok _:` で `false` を返す arm を置くと `invalid wasm generated: type mismatch: expected i64, found i32` で compile phase が失敗した。
- 同じ判定を `core/result.is_err<i64,i32>` に置き換えると compile/run は通るため、numeric parser の意味論ではなく wildcard pattern lowering / drop generation 側の問題と見ている。

## 問題

`Result<i64, E>` を `Result::Ok _` で match し、payload を使わず bool / i32 を返すだけの arm を書くと、i64 payload が stack に残ったままになり invalid wasm が生成される場合がある。stdlib numeric overflow の回帰テスト追加中に、`Result::Ok _` arm が単に `false` を返すだけで再現した。

## 影響

テストや利用者コードが、未使用の i64 payload を人工的に評価する workaround を必要とし得る。これは compiler の stack / drop bug を隠し、stdlib の例にも不自然な未使用値処理を増やす原因になる。

## 修正方針

pattern lowering / drop generation を修正し、非 i32 payload の wildcard pattern が payload を正しく消費し、match の各 arm が期待される stack type だけを残すようにする。

## 検証

`Result<i64,i32>` を `Result::Ok _` で match し、payload を使わず i32 / bool を返す compiler regression test を追加する。
