# stdlib と selfhost readiness 横断レビュー

## レビュー範囲

確認対象:

- stdlib core: `stdlib/core/**`
- stdlib alloc: `stdlib/alloc/**`
- stdlib std: `stdlib/std/**`
- selfhost compiler: `stdlib/neplg2/**`
- test/source policy: `nodesrc/run_source_policy_regressions.js`
- issue registry

観点:

- selfhost を進めるうえで必要な stdlib 基盤が揃っているか。
- collection, string, mem が型安全、メモリ安全を壊さない API になっているか。
- selfhost 側が enum と `match` による静的検査を活用できる構造になっているか。

## stdlib の到達点

`stdlib/alloc/string.nepl` は facade 化され、UTF-8、storage、access、builder、search、slice、split、integer、float、concat、builder extension、find などに分割されている。過去の巨大単一ファイル状態からは改善しており、責務単位でレビューしやすくなっている。

`stdlib/alloc/collections/vec.nepl` も facade 化され、types、storage、access、raw、mutation、query、transform、sort へ分かれている。これは selfhost の parser/HIR builder で collection を使う前提として良い進捗である。

`stdlib/std/test` は structured assertion/report へ進んでいる。selfhost と Rust compiler の共通 `.n.md` 運用を設計するうえで、stdout report と exit code を分ける方向が見えている。

## stdlib の未完了リスク

stdlib の最大リスクは、`core/mem` と collection の破棄責務である。現状は raw allocator と typed wrapper が同じライブラリ内にあり、呼び出し側が安全性を守る前提の API が残っている。これは selfhost の土台として危険である。

理想は次の構造である。

1. safe API は `MemPtr<T>` や `RegionToken<T>` のような typed handle を返す。
2. handle には compiler-owned provenance と lifetime/region 情報が対応する。
3. 初期化済み cell と未初期化 cell の状態は Resource IR で追跡される。
4. collection は要素の drop obligation を持ち、free/dealloc 前に必ず消化する。
5. raw address は unsafe/raw 境界に隔離され、通常の stdlib 利用者に escape しない。

現状は 1 が部分的にあり、2 から 5 が未完了である。したがって `core/mem` の raw API を表面だけ隠す修正では足りない。Resource IR、effect system、stdlib API、collection の destructor semantics をまとめて設計し直す必要がある。

関連 open issue:

- `ISS-20260427T152958303Z-MEMPTR-AND-REGIONTOKEN-LACK-COMPILER-0BC8ECDF`
- `ISS-20260427T152954558Z-CORE-MEM-EXPOSES-RAW-ADDRESS-ESCAPE--4185EA5D`
- `ISS-20260427T164432612Z-CORE-MEM-DEALLOC-APIS-DO-NOT-ENCODE--204F1F47`
- `ISS-20260425T000000Z-RV-STDLIB-004-91534828`

## selfhost の到達点

selfhost compiler は、lexer/parser/AST/HIR/type/builtin/name resolver などの基礎が `stdlib/neplg2/core/**` に存在する。直近の refactor により、未割当 ID や shared payload を numeric sentinel で扱う箇所が改善され、`Option` と enum payload へ移行した。

特に `SelfhostHirExprPayload` の導入は重要である。式の共通 metadata と variant payload が分離され、`match` で variant ごとのデータを処理できる。これは今後 typecheck と Resource IR lowering を実装するときに、網羅性検査を効かせる前提になる。

`SelfhostBuiltinSignature` も arity 別の enum になっているため、builtin 呼び出し検査で magic number に依存しにくい。`SelfhostTypeRecord` の Primitive/Function 分離も、型表現を静的に扱う方向として妥当である。

## selfhost の未完了リスク

selfhost はまだ部分実装であり、静的検査の本体が不足している。lexer/parser や HIR の整備は進められるが、type safety と memory safety を満たす compiler としては、次が必要である。

- Rust compiler 側の診断 ID 再設計に対応した selfhost diagnostic enum registry。
- HIR から selfhost Resource IR への lowering。
- owner/cell/borrow/effect/drop obligation の checker。
- `.n.md` test を Rust/selfhost 共通に走らせる test harness。
- raw memory と collection drop semantics が Resource IR で検査できる stdlib API。

また lexer には raw mode と directive state を `i32` で扱う未完了点が残っている。これは selfhost の早い段階で修正できる領域であり、静的検査大規模修正と比較すると干渉は小さい。ただし修正は helper 追加だけでなく、directive/raw mode の enum 化と `match` 化まで行う必要がある。

## selfhost 実装開始可否

現時点で開始できる作業:

- syntax lexer/parser の enum/match 化。
- AST/HIR builder の型安全な ID と payload 整備。
- diagnostic enum registry の selfhost 実装。
- module graph、VFS、name resolution の deterministic fixture 整備。
- `.n.md` stdout assertion report に基づく selfhost 共通 test 設計。

現時点では慎重に扱うべき作業:

- selfhost の full type checker 実装。
- selfhost Resource IR の本格実装。
- collection/string/mem を前提にした allocator-heavy な selfhost runtime。
- codegen と runtime memory operation の最終仕様化。

理由は、stdlib memory boundary と Resource IR の接続がまだ P1 issue として残っているためである。ここを曖昧にしたまま selfhost を大きく進めると、後で設計破棄が必要になる可能性が高い。

## 進捗状況

| 領域 | 状態 | 判断 |
| --- | --- | --- |
| stdlib string | 改善済み、継続レビュー | facade 分割済み。raw memory 境界は横断課題 |
| stdlib Vec/collections | 改善済み、Drop 未完了 | module 分割済み。free/drop obligation が P1 |
| stdlib `core/mem` | 未完了 P1 | raw API と compiler-owned provenance の接続が必要 |
| stdlib test | 改善済み、移行中 | structured report はある。`.n.md` 運用統一が未完了 |
| selfhost lexer/parser | 実装中 | lexer raw/directive state は enum 化が必要 |
| selfhost HIR/type/builtin | 改善済み | `Option` と payload enum 化が進んだ |
| selfhost diagnostic | 未完了 | Rust 側新設計に合わせる必要がある |
| selfhost static check | 未着手に近い | Resource IR/checker 実装が必要 |
| selfhost 実装開始 | 限定的に可 | syntax/model/test infra は可。memory/type checker 本体は設計確定後 |

## 判断

stdlib と selfhost は前進しているが、selfhost に必要な土台としては `core/mem` と collection drop obligation がまだ弱い。selfhost の parser や diagnostic など、メモリ設計に干渉しにくい範囲は進めてよい。一方で、型検査とメモリ検査の本体は、Rust compiler 側 Resource IR と stdlib safe API の authority を揃えてから本格実装するべきである。

次の実装優先順位:

1. `core/mem` の safe/raw API 境界と Resource IR provenance を設計し直す。
2. collection drop obligation を API と static check に接続する。
3. selfhost lexer raw/directive state を enum と `match` に置き換える。
4. selfhost diagnostic enum registry を Rust 側設計に合わせて作る。
5. selfhost Resource IR lowering/checker の skeleton を、Rust 側 Resource IR と対応する形で設計する。
