# 総レビュー findings

対象 commit: `f108cebd`

参照 Actions: `25157230630`

## 結論

NEPLg2 は、Rust compiler の Resource IR / typed diagnostic / stdlib safety migration が大きく進んでいる。一方で、main は GitHub Actions green ではなく、stdlib、WASI、`.n.md`、tutorial、dual backend に広域 failure が残る。現段階を「selfhost compiler 全体を実装開始できる状態」と扱うのは早い。

進めてよいのは、selfhost S1/S2 の source map、lexer、parser subset、module loader、typed diagnostic、CLI shell、stdlib helper の整備である。S3 以降の typecheck / Resource IR / codegen は、Rust 側の Resource IR final authority と stdlib memory model の完了条件に同期して設計する必要がある。

## レビュー commit

| review 単位 | commit | 内容 |
|---|---|---|
| outline | `f185c880` | `README.md` / `index.md` / review method の追加 |
| project progress | `aba537df` | project progress、Actions status、risk map |
| issue tracking | `d6af3c03` | owner variant path builder issue 追加 |
| Rust compiler | `bc9951af` | Rust compiler pipeline / parser / typecheck / Resource IR / backend review |
| stdlib | `b67db2ac` | stdlib core / string / collections / std / nm review |
| issue tracking | `f4d14740` | selfhost parser TokenKind hash dispatch issue 追加 |
| selfhost compiler | `a7911df9` | selfhost S0-S7 review |
| tools / quality | `478c298e` | CLI / nodesrc / language / web / tests / tutorials / examples review |
| crosscutting | `c04dd880` | static safety、stdlib readiness、diagnostics/tests/docs review |

## 重要 findings

### 1. Actions は green ではない

対象 run `25157230630` は failure である。`build`、`Source policy regressions`、`compile-test`、`llvm-test` は成功しているが、`rust-test`、`stdlib-test`、`wasi-test`、`nmd-doctest`、`tutorials-test`、`nm-compile`、dual backend verification が失敗している。

このため、compile gate が通ることと、stdlib / runtime / doctest / backend parity が正しいことは分けて扱う必要がある。

### 2. Resource IR は方向性は正しいが final authority ではない

Resource IR data model、typed diagnostic、owner/cell/borrow/raw 分類は正しい方向である。ただし旧 `passes::move_check` と HIR drop insertion が残るため、最終設計としては未完である。

旧 checker に special-case を足して維持するのではなく、move / borrow / initialized cell / owner obligation / effect / drop authority を Resource IR に統合する方針を維持する。

### 3. stdlib memory model は過渡期である

`StringBuilder` / `ByteBuf` / `ByteBuilder` の `Option<MemPtr<u8>>` 化、HashMap / HashSet の enum state 化、derived collection の raw header 廃止は良い進捗である。

しかし `core/mem` と `Vec<T>` はまだ根本 issue を残す。`MemPtr<T>` が owner field と non-owning view を兼ねる構造は、selfhost の長期基盤にしてはいけない。`OwnedBuffer`、owner token、initialized prefix、enum storage state への移行が必要である。

### 4. selfhost は S1/S2 に限定して進める段階

`stdlib/neplg2/` は placeholder から前進している。source text、span、diagnostic、lexer、module graph、CLI 境界は作業可能である。

一方で、S3 typecheck、S4 HIR/resource/mono、S5 backend は受け皿と marker API が中心で、Rust compiler と同等の安全性判定を持っていない。旧 HIR checker や raw memory helper を selfhost に移植すると、後で設計を破棄することになる。

### 5. diagnostic / `.n.md` / assert は移行中

Rust 側の `DiagnosticCode` と selfhost 側の `SelfhostDiagnosticCode` は正しい方向である。stable string は CLI / web / JSON / doctest 境界に限定すべきである。

`.n.md` test は Rust と selfhost で共通運用できるが、main が `i32` を返すだけの形式では失敗内容を追えない。stdout assertion report と exit code separation を stdlib assert と runner contract で固定する必要がある。

### 6. tutorial / README / examples は追従が必要

tutorial は getting_started の序盤から Actions failure を拾っている。README には NEPLg3 と NEPLg2 selfhost の説明が混在して見える箇所がある。examples は obsolete API 修正が進んだが、web sync と current stdlib API の継続確認が必要である。

## 追加 issue

今回の総レビュー中に追加した issue は次の 2 件である。

| issue | 理由 |
|---|---|
| `ISS-20260430T140641137Z-FROM-F64-RESULT-SCRATCH-BUFFER-REINT-1D9324F1` | Actions artifact で `from_f64_result` の `resource.cell.possibly_moved` が current main に残っているため。 |
| `ISS-20260430T141517141Z-SELF-HOST-PARSER-CLASSIFIES-TOKENKIN-645D236B` | selfhost parser が `TokenKind` を文字列/hash/numeric arm に変換し、enum/match の網羅性検査を効かせていないため。 |

横断 review では、既存 open issue で追跡できない新規 issue は確認していない。

## 修正優先順位

1. Resource IR final authority の完了条件を固定し、旧 move_check / HIR drop insertion の削除条件を明文化する。
2. `MemPtr` / `RegionToken` / owner token / initialized cell を stdlib と compiler で分離する。
3. `Vec<T>` と collection Drop contract を `OwnedBuffer` / initialized prefix / enum state に再設計する。
4. stdio/fs/string numeric formatter の current Actions failure を owner-preserving Result contract で直す。
5. `.n.md` stdout assertion report と exit code separation を実装し、Rust/selfhost 共通 fixture 運用へ移す。
6. selfhost parser の TokenKind direct match 化、Rust parity fixture、module/CLI timeout 分類を進める。
7. tutorial / README / examples を current NEPLg2 / stdlib API / selfhost plan に合わせる。
