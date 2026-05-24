---
id: ISS-20260524T123402690Z-GENERIC-CALLS-WITH-UNCONSTRAINED-TYP-DD4E3093
title: "Generic calls with unconstrained type args reach codegen"
area: compiler
status: verified
resolved: true
priority: P0
type: bug
created: 2026-05-24
updated: 2026-05-24
target: "nepl-core/src/typecheck/**; nepl-core/src/monomorphize.rs; stdlib/tests/option.n.md; stdlib/tests/result.n.md"
---

# ISS-20260524T123402690Z-GENERIC-CALLS-WITH-UNCONSTRAINED-TYP-DD4E3093: Generic calls with unconstrained type args reach codegen

## 概要

NEPLg2.1 postfix removal exposes generic calls whose type parameters remain unconstrained after typecheck, then reach wasm codegen as unknown specialized functions.

## 対象

- `nepl-core/src/typecheck/**; nepl-core/src/monomorphize.rs; stdlib/tests/option.n.md; stdlib/tests/result.n.md`

## 根拠

- `is_none none` は `--check` では通るが、wasm codegen で `unknown function 'none__unit__Option_T_T__pure_var_...'` になる。
- `is_err ok 5` は `ok` の error type が未確定のまま `ok__T__Result_T_E_T_E__pure_i32_var_...` として codegen に到達する。
- `is_ok err 7` は `err` の ok type が未確定のまま `err__E__Result_T_E_T_E__pure_var_..._i32` として codegen に到達する。
- 外側に `is_none<i32>` / `is_err<i32,i32>` / `is_ok<i32,i32>` がある場合は、同じ source shape が codegen まで通る。

## 問題

Expressions such as `is_none none`, `is_err ok 5`, and `is_ok err 7` can typecheck while leaving constructor or observer type parameters unresolved. The unresolved type variables then appear in specialized function names such as `none__unit__Option_T_T__pure_var_...` during wasm codegen.

## 影響

NEPLg2.1 generic postfix removal cannot safely remove consumer annotations in these sites. Worse, an underconstrained generic call is reported as a backend unknown-function error instead of a frontend static diagnostic or a resolved instantiation.

## 修正方針

Add a typecheck/monomorphization boundary check that rejects user-visible generic calls with unresolved type arguments before codegen, and improve consumer-driven expected-type propagation where an outer generic observer or explicit annotation can legitimately constrain an inner constructor. Keep truly ambiguous cases diagnostic rather than defaulting silently.

## 解決

- `nepl-core/src/typecheck/hir_finalize.rs` に generic user call の `type_args` 検査を追加し、関数自身の type parameter 以外の未束縛型変数が残る場合は `type.generic_call.unresolved_type_args` で停止するようにした。
- `FuncRef::User` の未解決型引数を typed HIR 確定後、Resource IR / codegen より前で拒否するため、`none__unit__Option_T_T__pure_var_...` のような codegen unknown function には到達しない。
- `Option .T` のように generic function 本体で正当に関数 type parameter を含む型適用は許可し、`choose opt` などの既存 NEPLg2.1 generic body は通す。

## 検証

Add focused tests for `is_none none`, `is_err ok 5`, and `is_ok err 7`: either they must resolve when an explicit consumer annotation supplies the missing type, or they must fail with a type diagnostic before wasm codegen. Also verify existing postfix-free `and_then` and branch-return constructor cases still pass.

- `cargo fmt --check`: passed.
- `cargo check -p nepl-core`: passed.
- `cargo test -p nepl-core --test functions neplg21 -- --nocapture`: passed.
- `cargo test -p nepl-core --test functions function_first_class -- --nocapture`: passed.
- `cargo test -p nepl-core --test functions function_neplg21_generic_body_type_params_remain_allowed -- --nocapture`: passed.
- Direct `nepl-cli.exe --target core --emit wasm --run` smoke confirms `is_none none`, `is_err ok 5`, and `is_ok err 7` now stop with `type.generic_call.unresolved_type_args` before wasm codegen.
- Direct `nepl-cli.exe --target core --emit wasm --run` smoke confirms explicit consumer forms `is_none<i32> none`, `is_err<i32,i32> ok 5`, and `is_ok<i32,i32> err 7` still compile/run.
