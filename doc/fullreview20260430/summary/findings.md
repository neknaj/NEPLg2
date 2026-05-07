# 総括 findings

## レビュー前提

この総括は、今回の進捗確認及び総レビューで確認した現行 source tree、issue registry、GitHub Actions 状態に基づく。前回レビュー本文の結論は、この時点では参照していない。前回レビューとの差分調査は、レビュー妥当性の再レビューが完了した後に別作業として行う。

基準 commit:

- `15e51fe0 docs(review): add crosscutting safety review`

直近の refactor と review checkpoint:

- `c64396d6 docs(review): add stdlib review`
- `b350213c docs(review): add selfhost compiler review`
- `46aa9bf7 docs(review): add quality tools review`
- `15e51fe0 docs(review): add crosscutting safety review`

issue registry:

- total: 608
- open: 15
- resolved: 593

## 最重要 findings

### 1. Rust compiler の静的検査経路は改善している

Rust compiler の `--check` 経路は、codegen 準備経路と同じ型検査、Resource IR monomorphize、Resource static check、drop elaboration bridge gate を通る構造になっている。これは「検査が行われるような実装にする」という方針に沿っている。

Resource IR は typed ID と enum state を中心にした model であり、owner/cell/borrow/effect/drop の責務を数値や文字列ではなく型で扱う方向に進んでいる。この方向は維持するべきである。

残る懸念は、Resource IR 自体の正しさよりも、stdlib raw memory API と Resource IR authority の接続である。ここが未完了だと、Resource IR が正しくても stdlib 経由で安全性が逃げる。

### 2. stdlib memory boundary が最大の P1 リスクである

`stdlib/core/mem.nepl` には raw allocator と typed wrapper が併存しており、現状では compiler-owned provenance、初期化状態、drop obligation を完全に表現できていない。raw API を使う caller に安全性を委ねる構造は、NEPLg2 の方針に対して弱い。

collections も、要素の `Drop` を伴う free/dealloc obligation が未完了である。selfhost が collections を広く使う前に、collection ownership と destructor semantics を Resource IR と接続する必要がある。

この問題は局所修正では解決しない。`core/mem` safe/raw boundary、`MemPtr<T>`/`RegionToken<T>`、collection free、drop insertion、Resource IR owner obligation を一つの設計として揃える必要がある。

### 3. selfhost は構文・モデル整備は進められるが、静的検査本体は設計確定が必要である

selfhost compiler は AST/HIR/type/builtin/name resolver の基礎が存在し、直近 refactor で sentinel や shared payload の問題がかなり改善した。`Option` と enum payload の利用は、方針に沿う良い進捗である。

一方、selfhost の full type checker、Resource IR、borrow/effect/drop checker はまだ未完成である。Rust compiler 側の Resource IR と diagnostic ID 設計を selfhost 側に移植する前提を固めずに大きく進めると、後で設計破棄が必要になる。

現時点では、lexer/parser/module graph/diagnostic registry/test harness など、静的検査本体に干渉しにくい範囲を優先して進めるのが妥当である。

### 4. 診断 ID と test report は正しい方向だが、CI gate 化が残る

Rust compiler の diagnostic ID は enum registry 化されており、文字列 ID を内部 authority にしない方向へ進んでいる。selfhost diagnostic ID はこの設計に揃える必要がある。

`stdlib/std/test` は structured assertion/report へ進んでいる。`.n.md` を Rust/selfhost 共通 test として運用するには、stdout に assertion report を出し、exit code は可否だけを表す運用へ統一する必要がある。

source policy regression は CI に入っているが warn-only である。静的検査方針を必達にするには、policy を fail gate にできる状態まで整える必要がある。

### 5. docs/tutorial/examples は仕様 drift の検出対象として扱う必要がある

tutorial と examples は単なる補助文書ではなく、現行仕様が利用者にどう見えるかを検査する regression である。getting_started tutorial doctest failure、examples doctest が CI gate になっていない問題は、仕様 drift を隠す。

特に後方互換不要の方針では、古い tutorial を残す意味は薄い。古い書き方は削除し、現行 compiler と stdlib の正しい使い方に合わせて継続的に検査する必要がある。

## open issue の優先度整理

P1 として先に解くべき issue:

- `ISS-20260427T132406497Z-CORE-MEM-RAW-MEMORY-OPERATIONS-BYPAS-DC67DF04`
- `ISS-20260427T152958303Z-MEMPTR-AND-REGIONTOKEN-LACK-COMPILER-0BC8ECDF`
- `ISS-20260429T040748194Z-RUST-COMPILER-DIAGNOSTICS-ARE-NOT-AL-1617747D`
- `ISS-20260425T000000Z-RV-STDLIB-004-91534828`
- `ISS-20260427T152954558Z-CORE-MEM-EXPOSES-RAW-ADDRESS-ESCAPE--4185EA5D`
- `ISS-20260427T164432612Z-CORE-MEM-DEALLOC-APIS-DO-NOT-ENCODE--204F1F47`
- `ISS-20260427T204839136Z-STDLIB-RAW-MEMORY-BACKED-APIS-REQUIR-E503CD84`
- `ISS-20260429T102425370Z-N-MD-TESTS-RELY-ON-RETURN-VALUES-INS-9B49EDAD`
- `ISS-20260507T161416607Z-VFS-CROSS-FILE-DEFINITION-PATH-TREE--CCFBA9F9`
- `ISS-20260507T161156205Z-GETTING-STARTED-TUTORIAL-DOCTESTS-FA-A0324153`

P2 として進めるべき issue:

- `ISS-20260507T153812328Z-EXAMPLES-DOCTESTS-ARE-NOT-RUN-BY-CI-13ED1895`
- `ISS-20260425T000000Z-RV-STDLIB-008-F4BCB5DD`
- `ISS-20260507T151236784Z-SELFHOST-LEXER-RAW-MODES-AND-DIRECTI-B080723B`
- `ISS-20260425T000000Z-RV-STDLIB-009-01749CCF`
- `ISS-20260507T153515441Z-ZED-EXTENSION-BUILD-ARTIFACTS-ARE-TR-B7D814F1`

## 進捗状況

| 作業範囲 | 状態 | 次の判断 |
| --- | --- | --- |
| Rust compiler parser/AST/HIR/codegen | 実装済み、継続レビュー | Resource IR gate との整合を維持する |
| Rust compiler static check | 実装中、重要改善あり | stdlib memory boundary との接続を優先 |
| Rust compiler diagnostics | 実装中、enum registry あり | selfhost へ同設計を展開する |
| stdlib string | 分割改善済み | raw memory 依存と API 境界を継続確認 |
| stdlib collections | 分割改善済み、Drop 未完了 | free/drop obligation を P1 で解く |
| stdlib `core/mem` | 未完了 P1 | safe/raw boundary と provenance を再設計 |
| stdlib `std/test` | 実装中、方向は良い | `.n.md` stdout report 運用へ統一 |
| selfhost syntax/model | 実装中、改善あり | lexer raw/directive state を enum 化 |
| selfhost typecheck/resource | 未完成 | Rust 側設計確定後に本格化 |
| tutorials/examples | 一部未追従 | CI gate と doctest を整備 |
| tools/editor/web | 実用段階、未整理あり | build artifact と diagnostic code 同期を整理 |

## 総合判断

NEPLg2 は、Rust compiler の静的検査基盤と stdlib/selfhost の型表現の改善が進んでいる。方向性は開発方針に合っている。ただし、メモリ安全の必達という観点では、stdlib `core/mem` と collections の raw/dealloc/drop 境界がまだ最重要 blocker である。

次に行うべき開発は、個別の表面修正ではなく、Resource IR authority と stdlib safe API を接続する設計の実装である。そのうえで selfhost の Resource IR/static check を Rust 側と揃えて構築するべきである。
