# Resolve / typecheck review

対象 commit: `f108cebd`

## 概要

typecheck は `typecheck/*` へ分割されているが、`prefix_check.rs` と `driver.rs` はまだ大きい。match exhaustiveness、effect check、trait capability、overload、function value / indirect call は実装済みで、selfhost S3 の主要参考になる。

## match / enum / scalar

`typecheck/match_check.rs` は、scrutinee type を次に分けている。

- enum
- `bool`
- `i32`
- `u8`
- `char`

enum match は variant coverage を検査する。bool match は true/false または wildcard を要求する。i32/u8/char は wildcard がない限り非網羅とする。wildcard arm は最後だけ許可される。

これは「match による網羅性検査を効かせる」方針に合っている。stdlib の有限分岐も、この基盤を使って `if` nest ではなく `match` に寄せるべきである。

## effect / indirect call

`typecheck/indirect_apply.rs` は function value の effect を HIR に保持し、pure context から impure function value を呼ぶ経路を検査する。直近 main では Resource IR lowering 側にも `EffectOp::IndirectCall { effect }` が入っており、以前の `Unknown` 問題は前進している。

## 良い点

- diagnostic は `type_error(...)` 経由で `TypeDiagnosticCode` を持つ。
- match exhaustiveness と wildcard-last が明示的に検査される。
- trait capability と Copy/Drop などの stdlib trait contract が typecheck に接続されている。
- overload selection は過去の full clone 問題から改善されている。

## 残る問題

- `prefix_check.rs` は約 1900 行で、prefix reduction、stack invariant、literal / field / borrow / call application が集中している。
- `stack.last().unwrap()` のような invariant 依存が多く、panic しない理由が型ではなく control flow に埋もれやすい。
- `driver.rs` も module-level hoist / directive / type environment assembly が大きい。

## selfhost への示唆

selfhost S3 では、`check/expr_reduce`, `check/overload`, `check/effect_check`, `check/pattern_check`, `ty/unify`, `resolve/scope` を分ける。特に prefix reduction の stack invariant は、空 stack を型で表せない場合でも `Outcome` / diagnostic path として扱う。

diagnostic code、type kind、match pattern kind は enum を主表現にし、raw string / numeric sentinel を主判定にしない。
