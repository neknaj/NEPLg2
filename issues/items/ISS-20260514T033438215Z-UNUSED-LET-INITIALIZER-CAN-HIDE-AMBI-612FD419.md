---
id: ISS-20260514T033438215Z-UNUSED-LET-INITIALIZER-CAN-HIDE-AMBI-612FD419
title: "unused let initializer can hide ambiguous overload diagnostics"
area: CORE
status: fixed
resolved: true
priority: P0
type: bug
created: 2026-05-14
updated: 2026-05-14
target: "nepl-core/src/typecheck/**, tests/compiler/overload.n.md, tests/stdlib/math.n.md"
---

# ISS-20260514T033438215Z-UNUSED-LET-INITIALIZER-CAN-HIDE-AMBI-612FD419: unused let initializer can hide ambiguous overload diagnostics

## 概要

A let binding such as let v cast 10 has no expected result type for the overloaded cast call, but compilation succeeds when the binding is unused. This lets an unresolved or ambiguous initializer escape type checking.

## 対象

- `nepl-core/src/typecheck/**, tests/compiler/overload.n.md, tests/stdlib/math.n.md`

## 根拠

- `tests/compiler/overload.n.md` の `overload_cast_mixed_requires_ascription` は `let v cast 10` に戻り値型の文脈が無いにもかかわらず成功 fixture になっていた。
- `tests/stdlib/math.n.md` の `cast_ambiguous_without_expected_type` は同じ問題を認識しつつ `type.overload.ambiguous` が出ないため skip されていた。
- 根本原因は `function_user_param_specificity` が名前に反して戻り値型の `type_shape_specificity` も加算し、期待戻り値型が無い overload 呼び出しでも戻り値型の形で候補を一意化していたこと。`cast (i32)->i128` のような構造的に大きい戻り値が、利用文脈の証明なしに選ばれ得た。

## 問題

A let binding such as let v cast 10 has no expected result type for the overloaded cast call, but compilation succeeds when the binding is unused. This lets an unresolved or ambiguous initializer escape type checking.

## 影響

Type safety and overload checking become use-dependent: using the binding can surface ambiguity, while leaving it unused silently accepts an underconstrained expression. Static verification must reject the initializer at the declaration site.

## 修正方針

戻り値型の文脈が無い overload 呼び出しでは、候補の順位付けに戻り値型の形を使わない。引数型だけで一意化できない候補は `type.overload.ambiguous` として拒否する。戻り値型は、型注釈・関数戻り値・外側引数位置などから期待型がある場合の候補フィルタでのみ使う。

## 対応結果

- `nepl-core/src/typecheck/binding_rules.rs` の `function_user_param_specificity` から戻り値型の具体度加算を削除した。
- 期待戻り値型が無い `castlike (i32)->i32` / `castlike (i32)->Wide` の overload が、戻り値の形だけでは選ばれない Rust 回帰テストを追加した。
- `tests/compiler/overload.n.md` と `tests/stdlib/math.n.md` の `cast` 曖昧性 fixture を `compile_fail` にし、`diag_code: type.overload.ambiguous` を確認する形に更新した。

## 検証

- `cargo test -p nepl-core --test neplg2 overloads_without_expected_return_do_not_use_return_shape_specificity -- --nocapture`: pass
- `cargo test -p nepl-core --test neplg2 overloads_ambiguous_return_type_is_error -- --nocapture`: pass
- `cargo fmt --package nepl-core --check`: pass
- `cargo run -q -p nepl-cli -- --stdlib-root stdlib --check -i <temp> --target core`: `let v cast 1` が `type.overload.ambiguous` で失敗することを確認
- `trunk build`: pass
- `node nodesrc/run_doctest.js -i tests/compiler/overload.n.md -n 35 --assert-io --dist web/dist`: pass
- `node nodesrc/run_doctest.js -i tests/stdlib/math.n.md -n 6 --assert-io --dist web/dist`: pass
- `node nodesrc/tests.js -i tests/compiler/overload.n.md -o .tmp-overload-tests.json --dist web/dist --assert-io --no-tree`: total=45, passed=45
