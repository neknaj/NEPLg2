# selfhost typecheck and resource review

確認対象 commit: `31291b37 fix(core): add parser backend responsibility policy`

## 確認対象

- `stdlib/neplg2/core/ty/ty.nepl`
- `stdlib/neplg2/core/check/checker.nepl`
- `stdlib/neplg2/core/resource/move_state.nepl`
- `stdlib/neplg2/core/builtins/prelude.nepl`
- `doc/neplg2/static_check_complexity_reduction_plan.md`
- `doc/neplg2/self_host_plan.md`

## 良い点

`ty.nepl` は `SelfhostTypeKind`、`SelfhostTypeId`、`SelfhostTypeArena`、function argument table、structural type equality を持つ。type stage の entry model は存在し、Rust 側の type arena に対応する設計へ進める余地がある。

`builtins/prelude.nepl` は builtin function を typed registry として持ち、`SelfhostBuiltinKind` と `SelfhostTypeKind` を使って signature を表している。文字列名だけで builtin を扱わない点は良い。

`check/checker.nepl` と `resource/move_state.nepl` は placeholder に近い。これは実装不足だが、Rust 側 ResourceIR 大規模修正が進行中の段階で selfhost が独自の簡易検査を作ってしまうより安全である。

## 問題とリスク

`ty.nepl` は `SelfhostTypeId(-1)`、primitive record の `first_arg = -1`、invalid result type で欠損状態を表す。これは型安全の基盤に invalid state を通常値として混ぜる設計であり、S3 で拡張すると後から直しにくい。

`builtins/prelude.nepl` は固定 arg slot `arg0` / `arg1` / `arg2` と `arg_count` を持ち、unused slot を `SelfhostTypeKind::Error` にしている。これは builtin signature の arity と payload が型で対応しない。arity 追加時にも match coverage が効きにくい。

`check/checker.nepl` は現状 smoke API であり、Rust typecheck の match exhaustiveness、trait/generic/overload/effect typing、diagnostic id にはまだ追従していない。

`resource/move_state.nepl` は Rust ResourceIR の owner/cell/borrow/drop/effect authority をまだ持たない。selfhost 側で簡易 move checker を積むと、Rust 側で避けようとしている二重 authority と同じ問題を再発させる。

## 追加 issue

- `ISS-20260507T150754473Z-SELFHOST-TYPE-HIR-AND-BUILTIN-MODELS-8EBC822D`
  - `ty` / `builtins` を含む typed IR model の sentinel を typed absence へ移す。

## 進捗状況

| 領域 | 状態 | 判定 |
|---|---|---|
| `ty/ty.nepl` | arena、TypeId、function type equality。 | modelはあるが invalid sentinelを直す。 |
| `builtins/prelude.nepl` | builtin kind/signature registry。 | string-onlyではないが Error placeholderを直す。 |
| `check/checker.nepl` | placeholder。 | S3未着手。 |
| `resource/move_state.nepl` | placeholder。 | S4未着手。Rust ResourceIR設計追従が必要。 |

## 推奨対応

- TypeId は invalid constructor を消し、`Option<SelfhostTypeId>` または `SelfhostTypeRef` enum で未解決を表す。
- Function type の args/result は `Primitive` と `Function` の variant-specific record に分けるか、typed `SelfhostTypeSignature` を導入する。
- Builtin signature は fixed arg slot ではなく、typed arity enum または arg range table を使う。
- selfhost typecheck は Rust 側 diagnostic code taxonomy と match exhaustiveness policy を先に移植してから実装する。
- Resource stage は Rust ResourceIR の model を参考に、owner/cell/borrow/drop/effect を同じ typed IR authority に統合する。簡易 checker を暫定設計として固定しない。
