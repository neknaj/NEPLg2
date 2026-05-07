# selfhost typecheck and resource review

確認対象 commit: `c5f93163 fix(selfhost): split hir expr payloads`

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

remote main の `0fcc4839 fix(selfhost): compare model enums without numeric tags` により、`SelfhostTypeKind` と `SelfhostBuiltinKind` の equality helper は enum を i32 tag に落とさず、直接 `match` する形へ改善された。これはユーザー提示の enum/match 静的検査方針に沿う前進である。

remote main の `0ac34132 fix(selfhost): model builtin signatures by arity` により、builtin signature は `SelfhostBuiltinSignature::Unary` / `Binary` / `Ternary` の arity enum へ移行した。unused arg slot を `SelfhostTypeKind::Error` で埋める設計ではなくなり、arity と payload の対応を `match` で検査できる方向へ改善された。

remote main の `4da7333 fix(selfhost): split type record payloads` により、type record は `SelfhostTypeRecord::Primitive` / `Function` の variant payload へ分離された。primitive record に `first_arg = -1` と invalid result TypeId を入れる設計ではなくなり、function-only field は `SelfhostFunctionTypeRecord` に閉じた。

remote main の `6277239 fix(selfhost): split hir range payloads` により、HIR child/param range は `Empty` / `Range` enum へ分離された。`SelfhostHirChildRange(-1, 0)` / `SelfhostHirParamRange(-1, 0)` の empty sentinel は通常値ではなくなり、range payload の有無を `match` で確定できる。

remote main の `b9e85f23 fix(selfhost): model mono instance absence with option` により、monomorphize instance の未割当は `Option<SelfhostMonoInstanceId>` で表されるようになった。`SelfhostMonoInstanceId` は stable table index の payload に限定され、`-1` invalid constructor と validity helper は削除された。

remote main の `8ff05570 fix(selfhost): model hir expr id absence with option` により、HIR expression ID の未割当は `Option<SelfhostHirExprId>` で表されるようになった。`SelfhostHirExprId` は stable table index の payload に限定され、`-1` invalid constructor と validity helper は削除された。

remote main の `dc6b82bb fix(selfhost): model def id absence with option` により、resolver DefId の未割当は `Option<SelfhostDefId>` で表されるようになった。`SelfhostNameBinding` の `def_id` が Option payload になり、binding 追加前の pending state と追加後の assigned state を型で区別できる。

remote main の `c5f93163 fix(selfhost): split hir expr payloads` により、HIR expression record は共通 `ty` / `span` と `SelfhostHirExprPayload` enum へ分離された。literal/name/child payload は variant を match した場合だけ読めるようになり、flat field と kind-independent placeholder payload の問題は解消済みである。

`check/checker.nepl` と `resource/move_state.nepl` は placeholder に近い。これは実装不足だが、Rust 側 ResourceIR 大規模修正が進行中の段階で selfhost が独自の簡易検査を作ってしまうより安全である。

## 問題とリスク

`ty.nepl` の primitive/function flat record 問題は `4da7333` で解消済みである。今後のリスクは、type model 拡張時に `SelfhostTypeRecord` の variant payload を崩して flat field / invalid sentinel を再導入しないことに移った。

`hir/hir.nepl` の child/param empty range sentinel は `6277239` で解消済みである。expression flat payload は `c5f93163` で解消済みである。残る HIR リスクは、今後 expression kind を増やすときに payload enum と accessor の match coverage を同時に増やせるか、また non-Copy payload owner model を stdlib memory model と整合させられるかである。

`builtins/prelude.nepl` の固定 arg slot / `Error` placeholder 問題は `0ac34132` で解消済みである。今後のリスクは、arity enum を増やすときに fallback 分岐や numeric tag 比較へ戻さず、constructor ごとの payload と `match` coverage を維持できるかに移った。

`check/checker.nepl` は現状 smoke API であり、Rust typecheck の match exhaustiveness、trait/generic/overload/effect typing、diagnostic id にはまだ追従していない。

`resource/move_state.nepl` は Rust ResourceIR の owner/cell/borrow/drop/effect authority をまだ持たない。selfhost 側で簡易 move checker を積むと、Rust 側で避けようとしている二重 authority と同じ問題を再発させる。

## 追加 issue

- `ISS-20260507T150754473Z-SELFHOST-TYPE-HIR-AND-BUILTIN-MODELS-8EBC822D`
  - selfhost typed IR model debt は fixed。typed absence / variant payload の親 issue として扱う。
- `ISS-20260507T152220930Z-SELFHOST-ENUM-EQUALITY-HELPERS-LOWER-4E1FAA87`
  - enum equality helper の numeric tag 化は fixed。親 issue の一部解決として扱う。
- `ISS-20260507T153554496Z-SELFHOST-BUILTIN-SIGNATURES-USE-ERRO-AEFFF7D4`
  - builtin signature の fixed slot / `Error` placeholder は fixed。親 issue の一部解決として扱う。
- `ISS-20260507T154503761Z-SELFHOST-TYPE-RECORDS-USE-INVALID-TY-E984125D`
  - primitive/function flat type record と invalid TypeId / `first_arg = -1` は fixed。親 issue の一部解決として扱う。
- `ISS-20260507T155231568Z-SELFHOST-HIR-RANGES-ENCODE-EMPTY-STA-8B562D49`
  - HIR child/param empty range の `(-1, 0)` sentinel は fixed。親 issue の一部解決として扱う。
- `ISS-20260507T155948337Z-SELFHOST-MONO-INSTANCE-IDS-USE-1-INV-434774DA`
  - mono instance ID の `-1` invalid sentinel は fixed。親 issue の一部解決として扱う。
- `ISS-20260507T160530818Z-SELFHOST-HIR-EXPRESSION-IDS-USE-1-IN-7A6D6ABC`
  - HIR expression ID の `-1` invalid sentinel は fixed。親 issue の一部解決として扱う。
- `ISS-20260507T161157719Z-SELFHOST-DEFINITION-IDS-USE-1-INVALI-E74DCE86`
  - resolver DefId の `-1` invalid sentinel は fixed。親 issue の一部解決として扱う。
- `ISS-20260507T161930297Z-SELFHOST-HIR-EXPRESSIONS-STORE-KIND--54E75EE3`
  - HIR expression flat payload は fixed。親 issue の一部解決として扱う。

## 進捗状況

| 領域 | 状態 | 判定 |
|---|---|---|
| `ty/ty.nepl` | arena、TypeId、function type equality。enum equality は direct match 化済み。type record は `Primitive` / `Function` payload 分離済み。 | primitive/function flat field sentinel は解消。今後は拡張時の variant payload 維持を監視する。 |
| `hir/hir.nepl` range | child/param range は `Empty` / `Range` payload 分離済み。 | empty range sentinel は解消。 |
| `hir/hir.nepl` expr id / payload | expr absence は `Option<SelfhostHirExprId>` 化済み。expression payload は `SelfhostHirExprPayload` enum 化済み。 | invalid expr ID sentinel と expression shared record は解消。expression 追加時の match coverage 退行を監視する。 |
| `builtins/prelude.nepl` | builtin kind/signature registry。enum equality は direct match 化済み。signature は arity enum 化済み。 | fixed slot / `Error` placeholder は解消。arity 追加時の coverage 退行を監視する。 |
| `mono/mono.nepl` | instance ID は stable table index。未割当は `Option<SelfhostMonoInstanceId>`。 | invalid instance ID sentinel は解消。cache 実装時の typed absence 維持を監視する。 |
| `resolve/name_resolver.nepl` | binding の DefId は `Option<SelfhostDefId>`。 | invalid DefId sentinel は解消。parent/import/hoist 拡張時の typed absence 維持を監視する。 |
| `check/checker.nepl` | placeholder。 | S3未着手。 |
| `resource/move_state.nepl` | placeholder。 | S4未着手。Rust ResourceIR設計追従が必要。 |

## 推奨対応

- TypeId は invalid constructor を public helper として戻さず、未解決や欠損は `Option<SelfhostTypeId>` または専用 enum で表す。
- Function type の args/result は導入済みの `SelfhostTypeRecord::Function` payload に閉じ込め、primitive variant から function-only field を読ませない。
- Builtin signature は導入済みの typed arity enum を維持し、arity 追加時は `match` arm と regression を同時に追加する。
- selfhost typecheck は Rust 側 diagnostic code taxonomy と match exhaustiveness policy を先に移植してから実装する。enum equality の numeric tag 退行は source policy で監視する。
- Resource stage は Rust ResourceIR の model を参考に、owner/cell/borrow/drop/effect を同じ typed IR authority に統合する。簡易 checker を暫定設計として固定しない。
