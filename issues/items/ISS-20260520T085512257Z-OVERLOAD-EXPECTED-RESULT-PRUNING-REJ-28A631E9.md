---
id: ISS-20260520T085512257Z-OVERLOAD-EXPECTED-RESULT-PRUNING-REJ-28A631E9
title: "overload expected-result pruning rejects unresolved expectation variables"
area: core
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-20
updated: 2026-05-20
target: "nepl-core/src/typecheck/overload_selection.rs, nepl-core/src/typecheck/type_expectation.rs, nepl-core/tests/overload.rs"
---

# ISS-20260520T085512257Z-OVERLOAD-EXPECTED-RESULT-PRUNING-REJ-28A631E9: overload expected-result pruning rejects unresolved expectation variables

## 概要

Overload selection uses one-way type_pattern_matches(result, expected) for expected-result prefiltering. When the expected type is still an unresolved inference variable from an outer consumer, concrete overload results do not match that variable and all valid candidates can be rejected before unification.

## 対象

- `nepl-core/src/typecheck/overload_selection.rs, nepl-core/src/typecheck/type_expectation.rs, nepl-core/tests/overload.rs`

## 根拠

- `cargo test -p nepl-core --test overload more_specific_get_overload_beats_generic_catchall -- --nocapture` が remote main `71bf4810` で失敗した。
- verbose trace では `mul 2 3` のような overload に対して `expected_ret=var_...` が伝播し、`type_pattern_matches(result, expected)` の片方向判定で concrete `i32` result が unresolved expected var に一致しないため全候補が落ちていた。
- `field::get_ref` / `HashMap::new` / `HashMap::get` そのものの候補構築ではなく、outer consumer 由来の期待型を overload 事前フィルタに使う条件が強すぎることが原因だった。
- compile success 系の overload regression helper は warning も failure として扱っていたため、error 解消後も stdlib shadow warning だけで regression が失敗していた。

## 問題

Overload selection uses one-way type_pattern_matches(result, expected) for expected-result prefiltering. When the expected type is still an unresolved inference variable from an outer consumer, concrete overload results do not match that variable and all valid candidates can be rejected before unification.

## 影響

Valid annotated and grouped calls in HashMap/field/hash code fail with Type(OverloadNoMatch), masking later static checks and blocking overload regression coverage.

## 修正方針

Treat expected-result prefiltering as pattern overlap instead of one-way matching, so unresolved expectation variables keep candidates alive while concrete mismatches are still rejected. Add focused overload regression coverage.

## 検証

cargo test -p nepl-core --test overload -- --nocapture; cargo check -p nepl-core; node nodesrc/issues.js check

## 2026-05-20 Agent 1 修正

`overload_selection.rs` の expected-result prefilter を片方向 `type_pattern_matches(result, expected)` から、候補 result と expected target の pattern overlap 判定へ変更した。これにより、期待型が concrete な場合は従来通り明確な不一致を落とし、期待型が未解決の型変数なら候補を残して後続の `GenericCallConstraint` / unification に解かせる。

`overload.rs` には `if` branch の outer consumer が未解決 branch 型を要求している状態で `mul 2 3` を解ける regression を追加した。あわせて compile success helper は `Severity::Error` のみを failure とし、stdlib の warning が compile success regression を止めない形へ揃えた。

検証:

- `cargo test -p nepl-core --test overload -- --nocapture`: passed
- `cargo check -p nepl-core`: passed
- `node nodesrc/issues.js check`: passed
- `git diff --check`: passed
