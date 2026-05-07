# 前回レビューとの差分

## 比較基準

前回レビュー:

- review commit: `c6db4213 docs(review): add final full review summary`
- 前回レビュー対象 commit: `f108cebd`
- 前回参照 Actions: `25157230630`

今回レビュー:

- review commit: `9655c078 docs(review): validate full review conclusions`
- 追加同期 commit: `caca505d fix(selfhost): model lexer raw modes with enums`
- 今回レビュー対象 commit: `caca505d`
- 今回参照 Actions: `25509681725` 以降の main push run

注意:

- 今回の独立レビューと妥当性再レビューが完了するまで、前回レビュー本文は参照していない。
- この文書だけは、レビュー完了後の差分調査として前回レビュー本文と git 差分を参照している。
- 現在の Actions は連続 push により pending/in_progress/cancelled が混在しているため、current main を green として扱わない。

## 全体差分

`c6db4213..9655c078` の主な変更範囲:

- 対象範囲の差分: 727 files changed, 80675 insertions, 44491 deletions
- Rust compiler: Resource IR、drop elaboration、initialized/owner/borrow/effect、diagnostic、loader、compiler pipeline が大きく更新
- stdlib: string、Vec、hashmap/hashset、json、sha256、io、math、fs、stdio、streamio、std/test、tui などで大規模分割
- selfhost: HIR/type/builtin/name resolver/lexer/parser/module graph 周辺が更新
- nodesrc: source policy regression が大幅追加
- tests: stdlib/resource/selfhost/tutorial/examples 周辺の fixture が更新
- issues: total 461 から 608 へ増加

## issue 差分

| 指標 | 前回 | 今回 | 差分 |
| --- | ---: | ---: | ---: |
| total | 461 | 608 | +147 |
| open | 16 | 14 | -2 |
| resolved | 445 | 594 | +149 |
| core open | 6 | 3 | -3 |
| stdlib open | 7 | 5 | -2 |
| selfhost open | 2 | 1 | -1 |
| TEST open | 1 | 2 | +1 |
| tutorials open | 0 | 1 | +1 |
| examples open | 0 | 1 | +1 |
| tools open | 0 | 1 | +1 |

前回 open から fixed になった issue:

- `ISS-20260425T000000Z-RV-CORE-009-58589A3F`: move/borrow/drop が Resource IR なしで後付け実装されている
- `ISS-20260430T135243330Z-RESOURCE-OWNER-VARIANT-PATH-BUILDER--87B356A8`: Resource owner variant path builder responsibility split
- `ISS-20260430T141517141Z-SELF-HOST-PARSER-CLASSIFIES-TOKENKIN-645D236B`: selfhost parser TokenKind string/hash dispatch
- `ISS-20260429T155343006Z-COLLECTION-STORAGE-STATES-USE-NUMERI-E4B3A749`: collection storage state numeric/null sentinel
- `ISS-20260430T140641137Z-FROM-F64-RESULT-SCRATCH-BUFFER-REINT-1D9324F1`: `from_f64_result` scratch buffer moved-cell failure
- `ISS-20260430T012045983Z-RESOURCE-IR-CANNOT-SUMMARIZE-RETURNE-2FDA4B38`: returned raw headers with dynamic ranges

前回から継続 open の issue:

- `ISS-20260427T132406497Z-CORE-MEM-RAW-MEMORY-OPERATIONS-BYPAS-DC67DF04`
- `ISS-20260427T152958303Z-MEMPTR-AND-REGIONTOKEN-LACK-COMPILER-0BC8ECDF`
- `ISS-20260429T040748194Z-RUST-COMPILER-DIAGNOSTICS-ARE-NOT-AL-1617747D`
- `ISS-20260425T000000Z-RV-STDLIB-004-91534828`
- `ISS-20260427T152954558Z-CORE-MEM-EXPOSES-RAW-ADDRESS-ESCAPE--4185EA5D`
- `ISS-20260427T164432612Z-CORE-MEM-DEALLOC-APIS-DO-NOT-ENCODE--204F1F47`
- `ISS-20260427T204839136Z-STDLIB-RAW-MEMORY-BACKED-APIS-REQUIR-E503CD84`
- `ISS-20260429T102425370Z-N-MD-TESTS-RELY-ON-RETURN-VALUES-INS-9B49EDAD`
- `ISS-20260425T000000Z-RV-STDLIB-008-F4BCB5DD`
- `ISS-20260425T000000Z-RV-STDLIB-009-01749CCF`

今回新たに open として残った issue:

- `ISS-20260507T161416607Z-VFS-CROSS-FILE-DEFINITION-PATH-TREE--CCFBA9F9`
- `ISS-20260507T161156205Z-GETTING-STARTED-TUTORIAL-DOCTESTS-FA-A0324153`
- `ISS-20260507T153812328Z-EXAMPLES-DOCTESTS-ARE-NOT-RUN-BY-CI-13ED1895`
- `ISS-20260507T153515441Z-ZED-EXTENSION-BUILD-ARTIFACTS-ARE-TR-B7D814F1`

今回新たに見つかり、レビュー中の同期で fixed になった代表 issue:

- `ISS-20260507T151236784Z-SELFHOST-LEXER-RAW-MODES-AND-DIRECTI-B080723B`

## 作業範囲ごとの進捗

### Rust compiler static check

前回は、Resource IR は正しい方向だが final authority ではなく、旧 `passes::move_check` と HIR drop insertion が残ると評価していた。

今回までに、Resource IR gate は `--check` 経路へ入り、drop elaboration plan、drop bridge validation、owner/cell/borrow/effect の Resource IR 側 checker が進んだ。前回 open だった「Resource IR なしで後付け実装されている」issue は fixed になっている。

ただし、`core/mem` raw API と Resource IR authority の接続はまだ open であり、メモリ安全の最終 blocker は残る。

### Resource IR

前回以降、initialized summary、raw range、owner summary、drop plan、drop point path、borrow usage、effect counts などが大きく分割・増強された。特に returned raw header、dynamic range、drop elaboration plan まわりの issue が fixed になった点は大きい。

一方で、raw address escape、raw memory effects、`MemPtr`/`RegionToken` provenance は継続 open である。Resource IR の内部表現は進んだが、stdlib public API と完全に接続される段階ではない。

### stdlib

前回は string/collections/std/fs/stdio などが過渡期で、memory model が大きな未完了として扱われていた。

今回までに、string は多数の submodule に分割され、Vec も storage/access/raw/mutation/query/transform/sort へ分割された。hashmap/hashset、json、sha256、alloc io、math、fs、stdio、streamio、std/test、wasix tui も facade と submodule へ整理されている。collection storage state numeric sentinel issue は fixed になった。

まだ残る根本課題は、collection element Drop、`core/mem` raw escape、dealloc/drop obligation、raw-memory-backed API migration である。つまり分割と source policy は大きく進んだが、memory safety の最終設計は未完了である。

### selfhost compiler

前回は S1/S2 まで限定して進める段階で、TokenKind direct match、Rust parity、module graph、diagnostic などが課題だった。

今回までに、selfhost parser の TokenKind string/hash dispatch issue は fixed になり、HIR/type/builtin/name resolver の sentinel/shared payload debt も大きく改善した。`SelfhostHirExprPayload`、`SelfhostHirChildRange`、`SelfhostTypeRecord`、`SelfhostBuiltinSignature`、`Option<SelfhostDefId>` など、enum/Option を使う方向へ進んでいる。

一方で、selfhost compiler が部分実装であること自体は継続 open である。新たに見つかった lexer raw mode/directive state の `i32` sentinel 問題は、remote main の `caca505d` で `SelfhostLexerRawMode` enum と `match` による実装へ修正され、issue は fixed になった。今後は回帰監視対象として扱う。

### diagnostics と `.n.md`

前回は diagnostic / `.n.md` / assert が移行中で、stdout assertion report と exit code separation が必要と判断していた。

今回までに Rust 側の diagnostic enum registry は維持され、stdlib `std/test` は structured assertion/report へ進んだ。ただし `.n.md` の return-value 依存 issue は継続 open である。selfhost diagnostic ID も Rust 側設計に追従する必要があり、P1 として残る。

### tests, tutorials, examples, tools

前回は Actions が green ではなく、stdlib/WASI/`.n.md`/tutorial/dual backend の failure が残ると評価していた。

今回 review 時点でも latest Actions は pending/in_progress であり、green と判断できない。代わりに、CI coverage gap はより具体的に issue 化された。getting_started tutorial doctest failure、VFS cross-file tree test failure、examples doctest が CI gate になっていない問題、Zed extension build artifact tracking が新規 open として整理された。

## 進捗状況

| 領域 | 前回 | 今回 | 進捗 |
| --- | --- | --- | --- |
| Rust Resource IR authority | 方向性は良いが旧 checker 残存 | `--check` gate と drop bridge が進行 | 大きく前進 |
| Resource dynamic range/drop plan | returned raw header などが open | 関連 issue 複数 fixed | 大きく前進 |
| stdlib string/Vec 分割 | 過渡期 | 多数 submodule 化、source policy 追加 | 大きく前進 |
| stdlib memory safety | P1 blocker | `core/mem`/Drop/dealloc が継続 open | 未解決 |
| selfhost TokenKind dispatch | open | fixed | 解決 |
| selfhost typed model | 部分的 | HIR/type/builtin/resolve が enum/Option 化 | 前進 |
| selfhost lexer raw mode | 未検出 | issue 化後に `caca505d` で fixed | 解決 |
| selfhost full implementation | 部分実装 | 部分実装 open 継続 | 未解決 |
| `.n.md` stdout report | 必要 | open 継続 | 未解決 |
| tutorials/examples CI | 追従必要 | 新規 issue で具体化 | 課題が明確化 |
| latest Actions | failure | pending/in_progress | green 判定不可 |

## 総合評価

前回からの最大の進捗は、Rust compiler の Resource IR authority と stdlib/selfhost の enum/Option 化が進み、前回 open の一部が fixed になった点である。特に Resource IR の drop/initialized/raw range 周辺と、stdlib の大規模分割、selfhost HIR/type/builtin/resolve の sentinel 排除は有意な前進である。

一方で、前回から変わらず最重要 blocker は memory safety である。`core/mem` raw API、`MemPtr`/`RegionToken` provenance、collection Drop/dealloc obligation、raw-memory-backed API migration は継続 open であり、selfhost の typecheck/resource/codegen を本格化する前に解く必要がある。

したがって、通常開発に戻る際の優先順位は次の通りである。

1. `core/mem` safe/raw boundary と compiler-owned provenance を設計・実装する。
2. collections の element Drop と free/dealloc obligation を Resource IR と接続する。
3. `.n.md` stdout assertion report と exit code separation を完了する。
4. selfhost lexer raw/directive state の enum/match 化が戻らないよう source policy を維持する。
5. tutorials/examples/tools の CI gap を潰し、review で見えた drift を通常開発で回収する。
