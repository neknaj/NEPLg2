# プロジェクト進捗レビュー

確認対象 commit: `545d2ab0 fix(resource): align region_ptr reference coverage`

## 確認した一次情報

- `plan.md`: NEPLg2 の核となる言語方針。前置記法、式指向、オフサイドルール、型注釈、型推論、`#import` / `#target` / `#indent` など。
- `note.n.md`: 2026-05-07 の Agent 1 / Agent 2 作業記録。ResourceIR coverage 修正、Vec / string / streamio / nm / stdio debug の module split、examples string import 修正など。
- `todo.md`: selfhost、NEPLg3、playground、tutorial、2026-04-25 review 由来の未着手作業。
- `issues/index.json`: issue 集計。現在 `total=590`, `open=10`, `resolved=580`。
- recent commits: ResourceIR 修正と stdlib split/refactor が集中している。
- `doc/neplg2/self_host_plan.md` / `self_host_execution_plan.md`: selfhost の S0-S7 成功条件、branch/checkpoint/Issue 運用。

## 全体判定

NEPLg2 は「既存機能を増やす段階」から、「静的検査・memory model・stdlib の責務境界を selfhost 可能な形へ締め直す段階」に入っている。最近の main は ResourceIR の soundness regression と stdlib module split が主であり、方向性は開発方針と一致している。

ただし、selfhost を全面的に進められる状態ではない。S1/S2 相当の lexer / parser / module loader は制限付きで進められるが、S3 以降の typecheck / ResourceIR / codegen は、MemPtr/RegionToken の owner model、stdlib raw-memory-backed API、diagnostic taxonomy、n.md test/assert 運用の残件に依存する。

## 進捗状況

| 領域 | 状態 | 判定 |
|---|---|---|
| Rust compiler `nepl-core` | ResourceIR 関連が大規模に分割され、owner/cell/borrow/raw coverage の regression が継続的に追加されている。最新 commit は `region_ptr` helper 経由の reference coverage を修正した。 | 中核改善中。静的検査 authority の最終形はまだ個別レビューで確認が必要。 |
| Rust CLI `nepl-cli` | CLI と backend runner は既存構造を維持。recent open issue は CLI にはない。 | 主要 blocker は現時点では core 側。 |
| selfhost `stdlib/neplg2` | ディレクトリ骨格と S1/S2 周辺の module は存在する。issue 上は `NEPLg2 self-host compiler が部分実装に留まっている` が open。 | S1/S2 は進行可能。S3 以降は静的検査と stdlib owner model の制約を守る必要がある。 |
| NEPLg3 `stdlib/neplg3` | 仕様 doc と placeholder compiler tree がある。NEPLg2 selfhost とは別扱い。 | 今回は進捗確認対象。NEPLg2 selfhost の作業場所として使わない。 |
| stdlib core/alloc/std | string、Vec、streamio、nm、stdio debug などで facade 化と責務分割が進んだ。open issue は stdlib 5 件。 | 方向は良いが `core/mem`、raw-memory-backed API、collection drop/free、巨大 file split が残る。 |
| tests / harness | `.n.md` doctest、source policy regression、Rust tests、playground editor tests が広い。n.md stdout/assert 運用は open issue。 | 検証資産は厚いが、test contract はまだ改善途中。 |
| docs / tutorials | `doc/neplg2` に selfhost/static check/std safety/test plan が増えている。tutorial rewrite は計画済み。 | tutorial は古さが残るため個別レビューで確認する。 |
| CI / Actions | workflow は build、compile-test、rust-test、WASI/nmd/tutorial/stdlib/LLVM/pages を持つ。最新 run は queued。 | 成功/失敗は `gh` で継続確認が必要。 |

## issue 状況

`issues/index.json` の現在値:

- total: 590
- open: 10
- resolved: 580

open issue の内訳:

| area | open | 主な内容 |
|---|---:|---|
| core | 3 | `core/mem` raw memory bypass、MemPtr/RegionToken owner provenance、ResourceIR/selfhost diagnostic alignment。 |
| stdlib | 5 | collection free/drop、safe mem API、dealloc obligation、raw-memory-backed API migration、巨大 stdlib file split。 |
| TEST | 1 | `.n.md` tests が return value に依存し stdout assertion report になっていない。 |
| selfhost | 1 | selfhost compiler が部分実装に留まっている。 |

## selfhost 開始可否

現段階で開始してよい範囲:

- lexer / parser / token / span / source text の Rust parity fixture 作成。
- module graph / import spec / loader の Copy payload 中心の実装。
- diagnostic code enum と stable string 境界の設計追従。
- CLI driver / args / reporter の境界整理。

まだ全面実装へ進めるべきでない範囲:

- typecheck / ResourceIR / borrow / drop の本実装。
- non-Copy AST/HIR/diagnostic payload を大量に保持する collection 設計。
- `MemPtr` / `RegionToken` を owner token として前提にする selfhost memory model。
- n.md test を return value contract のまま selfhost 共通テストへ固定すること。

## 次に確認する領域

1. `nepl-core/src/resource/**` を中心に Rust 静的検査の authority と残リスクを確認する。
2. `stdlib/core/mem.nepl`、`stdlib/alloc/string/**`、`stdlib/alloc/collections/**` の owner model と source policy を確認する。
3. `stdlib/neplg2/**` の selfhost 実装段階と Rust parity の不足を確認する。
4. `tests/**` と `nodesrc/**` の test contract、特に `.n.md` / assert / stdout report を確認する。
