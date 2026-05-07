# プロジェクト進捗レビュー

確認対象 commit: `caca505d fix(selfhost): model lexer raw modes with enums`

## 確認した一次情報

- `plan.md`: NEPLg2 の核となる言語方針。前置記法、式指向、オフサイドルール、型注釈、型推論、`#import` / `#target` / `#indent` など。
- `note.n.md`: 2026-05-07 の Agent 1 / Agent 2 作業記録。ResourceIR coverage 修正、Vec / string / streamio / nm / stdio debug の module split、examples string import 修正、selfhost enum equality / builtin signature / type record payload / HIR range payload 修正など。
- `todo.md`: selfhost、NEPLg3、playground、tutorial、2026-04-25 review 由来の未着手作業。
- `issues/index.json`: issue 集計。現在 `total=608`, `open=14`, `resolved=594`。
- recent commits: ResourceIR 修正、stdlib split/refactor、selfhost enum/match 化が集中している。
- `doc/neplg2/self_host_plan.md` / `self_host_execution_plan.md`: selfhost の S0-S7 成功条件、branch/checkpoint/Issue 運用。

## 全体判定

NEPLg2 は「既存機能を増やす段階」から、「静的検査・memory model・stdlib の責務境界を selfhost 可能な形へ締め直す段階」に入っている。最近の main は ResourceIR の soundness regression、stdlib module split、selfhost model の enum/match 化が主であり、方向性は開発方針と一致している。

ただし、selfhost を全面的に進められる状態ではない。S1/S2 相当の lexer / parser / module loader は制限付きで進められるが、S3 以降の typecheck / ResourceIR / codegen は、MemPtr/RegionToken の owner model、stdlib raw-memory-backed API、diagnostic taxonomy、n.md test/assert 運用の残件に依存する。

## 進捗状況

| 領域 | 状態 | 判定 |
|---|---|---|
| Rust compiler `nepl-core` | ResourceIR 関連が大規模に分割され、owner/cell/borrow/raw coverage の regression が継続的に追加されている。`region_ptr` / `region_ptr_at` の non-owning provenance regression も固定された。public monomorphize API panic は `c58dd6e3` で Result 化され、parser/backend responsibility policy は `31291b37` で追加された。 | 中核改善中。policy は入ったため、次は実分割の継続確認。 |
| Rust CLI `nepl-cli` | CLI と backend runner は既存構造を維持。`--check` は `3742a1a7` で compile preparation を共有し、ResourceIR gate と drop insertion bridge まで通るよう修正された。 | 良い。artifact emission に入らず safety authority を共有する regression が追加済み。 |
| selfhost `stdlib/neplg2` | ディレクトリ骨格と S1/S2 周辺の module は存在する。`0fcc4839` で enum equality helper が direct match 化され、`0ac34132` で builtin signature が arity enum 化され、`4da7333` で type record payload が `Primitive` / `Function` に分離され、`6277239` で HIR range payload が `Empty` / `Range` に分離され、`b9e85f23` で mono instance absence が `Option<SelfhostMonoInstanceId>` 化され、`8ff05570` で HIR expr absence が `Option<SelfhostHirExprId>` 化され、`dc6b82bb` で resolver DefId absence が `Option<SelfhostDefId>` 化され、`c5f93163` で HIR expression payload が variant enum 化された。`caca505d` で lexer raw mode も `SelfhostLexerRawMode` enum へ移行済み。 | S1/S2 は進行可能。S3 以降は ResourceIR/stdlib owner model と HIR payload regression policy の制約を守る必要がある。 |
| NEPLg3 `stdlib/neplg3` | 仕様 doc と placeholder compiler tree がある。NEPLg2 selfhost とは別扱い。 | 今回は進捗確認対象。NEPLg2 selfhost の作業場所として使わない。 |
| stdlib core/alloc/std | string、Vec、streamio、nm、stdio debug などで facade 化と責務分割が進んだ。stdlib review で core/mem、collections、string、std I/O、nm/kp/TUIを整理した。open issue は stdlib 5 件。 | 方向は良いが `core/mem`、raw-memory-backed API、collection drop/free、巨大 file split が残る。 |
| tests / harness | `.n.md` doctest、source policy regression、Rust tests、playground editor tests が広い。n.md stdout/assert 運用は open issue。 | 検証資産は厚いが、test contract はまだ改善途中。 |
| examples | `examples/*.nepl` は CLI/WASI/stdio/nm/rpn 等の実行例として整っている。レビュー中に examples doctest が CI gate へ入っていない問題を issue 化した。 | examples は品質資産として有用だが、CI で runner 対象に含める必要がある。 |
| docs / tutorials | `doc/neplg2` に selfhost/static check/std safety/test plan が増えている。tutorial rewrite は進んだが、直前 Actions では `tutorials-test` failure が出ている。getting_started doctest failure は issue 化済み。 | latest completed run の log で再確認し、fixture drift と compiler/API regression を切り分ける。 |
| tools / editor | `nodesrc`、web/playground、LSP/editor、Zed extension を確認した。Zed target build artifact は issue 化済み。 | runner は強いが、生成物混入と test coverage gap は継続監視が必要。 |
| CI / Actions | workflow は build、compile-test、rust-test、WASI/nmd/tutorial/stdlib/LLVM/pages を持つ。最新 run `25508600937` は `c5f93163` 対象で in_progress。直前 `25507326678` は後続 push により cancelled だが、`build` / `compile-test` success、`tutorials-test` / `nm-compile` failure を確認。 | green 判定は未確定。completed latest run を `gh` で継続確認する。 |

## issue 状況

`issues/index.json` の現在値:

- total: 608
- open: 14
- resolved: 594

open issue の内訳:

| area | open | 主な内容 |
|---|---:|---|
| core | 3 | `core/mem` raw memory bypass、MemPtr/RegionToken owner provenance、ResourceIR/selfhost diagnostic alignment。 |
| stdlib | 5 | collection free/drop、safe mem API、dealloc obligation、raw-memory-backed API migration、巨大 stdlib file split。 |
| TEST | 2 | `.n.md` tests が return value に依存し stdout assertion report になっていない。VFS cross-file definition path tree tests が failing。 |
| tutorials | 1 | getting_started doctest が current main で failing。 |
| selfhost | 1 | selfhost compiler が部分実装に留まっている。lexer raw mode は enum 化済みのため回帰監視へ移った。 |
| examples | 1 | examples / doc examples の doctest が CI runner 対象に入っていない。 |
| tools | 1 | Zed extension の build artifacts が tracked file として混入している。 |

## selfhost 開始可否

現段階で開始してよい範囲:

- lexer / parser / token / span / source text の Rust parity fixture 作成。
- module graph / import spec / loader の Copy payload 中心の実装。
- diagnostic code enum と stable string 境界の設計追従。
- CLI driver / args / reporter の境界整理。
- builtin signature arity enum、type record variant payload、HIR range payload、mono/HIR expr id typed absence の維持、regression 追加。

まだ全面実装へ進めるべきでない範囲:

- typecheck / ResourceIR / borrow / drop の本実装。
- non-Copy AST/HIR/diagnostic payload を大量に保持する collection 設計。
- `MemPtr` / `RegionToken` を owner token として前提にする selfhost memory model。
- n.md test を return value contract のまま selfhost 共通テストへ固定すること。

## 次に確認する領域

1. quality / tools / NEPLg3 の個別レビュー内容を commit し、latest Actions failure を継続確認する。
2. `crosscutting/static-safety.md` で Rust ResourceIR、selfhost typed IR、stdlib memory model を一体で確認する。
3. `crosscutting/diagnostics-tests-docs.md` で diagnostic id、assert/stdout report、tutorial failure を整理する。
4. `crosscutting/stdlib-selfhost-readiness.md` で selfhost に必要な stdlib API と残 blocker を整理する。
