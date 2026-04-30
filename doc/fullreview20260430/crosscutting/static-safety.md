# 横断レビュー: 静的安全性

対象 commit: `f108cebd`

参照 Actions: `25157230630`

## 結論

静的検査の設計方針は、`DiagnosticCode` / Resource diagnostic / owner state / cell state を enum として持ち、`match` の網羅性検査を効かせる方向へ進んでいる。この方向は、型安全とメモリ安全を必達にするという開発方針に合っている。

ただし現時点の実装は、まだ final authority ではない。旧 `passes::move_check` と HIR 上の drop insertion が先に走り、その後に Resource IR gate を追加する二重構造である。これは移行中の防壁としては妥当だが、selfhost が採用すべき最終設計ではない。

今回の review では、静的検査を弱めるべき箇所は確認していない。Actions で `resource.owner.*` / `resource.cell.*` が失敗している箇所は、検査の誤検知として隠すのではなく、stdlib の所有権契約、Resource IR lowering、または function summary の不足として根本修正する必要がある。

## 進捗状況

| 領域 | 状況 | review |
|---|---|---|
| `nepl-core/src/diagnostic_codes.rs` | 実装済み寄り | 数値 ID ではなく階層 enum を中心にした設計へ移行済み。stable string は表示・JSON 境界に限定する方向。 |
| `nepl-core/src/resource` | 実装中 | Resource IR data model と owner/cell/borrow/raw diagnostic は進んだが、旧 checker との authority split が残る。 |
| `nepl-core/src/passes/move_check` | 移行対象 | 現行 Actions では防壁として有効だが、最終設計では Resource IR に統合して削除条件を固定すべき。 |
| `nepl-core/src/passes/drop_insertion.rs` | 移行対象 | HIR 上で drop を入れる現方式は、Resource IR が drop obligation の最終 authority になる設計と競合する。 |
| `nepl-core/src/typecheck` | 実装中 | match exhaustiveness、effect、typed diagnostic は進んだ。Resource IR に渡す型付き情報の完全性が今後の焦点。 |
| `stdlib/core/mem.nepl` | 過渡 | raw memory boundary は防壁が増えたが、`MemPtr` / owner token / initialized cell の分離が未完。 |
| `stdlib/neplg2/core/resource` | 未実装相当 | selfhost 側は Rust 旧 checker を移植せず、Resource IR authority を前提に設計すべき。 |

## Resource IR authority

Resource IR は、move / borrow / initialized cell / owner obligation / raw provenance / effect を同じ検査入力で扱うための中核である。`doc/neplg2/static_check_complexity_reduction_plan.md` と `doc/neplg2/static_check_soundness_review_20260430.md` の方向性は、この review でも妥当と判断する。

未完了点は次の通り。

- `passes::move_check::run` がまだ authoritative gate として残る。
- HIR drop insertion が Resource IR check より前に実行される。
- `UnsafeMemoryInPureFunction` が stdlib 移行のため shadow-only に残る。
- stdlib の `MemPtr` owner/view 混同を補うため、Resource IR 側に special-case alias summary が増えやすい。
- selfhost S3 以降にコピーできる最終形がまだ固定されていない。

したがって、今後の根本修正は「旧 checker に special-case を足す」ではなく、Resource IR を final authority にする方向で進めるべきである。

## enum / match / stable string

診断 code、resource state、token kind、AST kind、storage state は、raw number や raw string を主表現にしてはいけない。現在の Rust compiler diagnostic redesign は、`DiagnosticCode` と下位 enum を内部表現にし、`as_str()` を外部境界に限定する方針なので適切である。

selfhost 側も同じ制約を持つ。`SelfhostDiagnosticCode` を typed enum にした変更は良いが、parser で `TokenKind` を文字列化し、hash 値で分岐する実装が残っている。これは `ISS-20260430T141517141Z-SELF-HOST-PARSER-CLASSIFIES-TOKENKIN-645D236B` で追跡済みであり、`TokenKind` を直接 `match` する形へ直す必要がある。

review 上の判定:

- finite state は enum にする。
- finite branch は `match` にする。
- wildcard arm は、将来 variant の追加を握りつぶす場合は禁止する。
- stable string は CLI / web / JSON / doctest の表示・比較境界だけで使う。

## Actions evidence

対象 Actions run `25157230630` は failure である。`build`、`Source policy regressions`、`compile-test`、`llvm-test` は成功した一方で、`rust-test`、`stdlib-test`、`wasi-test`、`nmd-doctest`、`tutorials-test`、dual backend verification は失敗した。

Artifact 分類では、stdlib / nmd / wasi / dual backend の失敗に `resource.owner.maybe_leak`、`resource.owner.leak`、`resource.cell.possibly_moved`、`resource.cell.uninit` が多い。これは compiler が検出し始めた問題であり、検査を弱める理由にはならない。

特に重要な current failure:

- `sb_build_result`: owner may leak
- `stdio_write_fd_mem_result`: owner may leak
- `from_f64_result`: `resource.cell.possibly_moved`
- `fs_open_with_flags`: owner may leak
- selfhost module / parser / CLI doctest: timeout と owner failure

## 既存 issue との対応

| issue | review 判断 |
|---|---|
| `ISS-20260425T000000Z-RV-CORE-009-58589A3F` | Resource IR final authority の親 issue。引き続き P1。 |
| `ISS-20260427T132406497Z-CORE-MEM-RAW-MEMORY-OPERATIONS-BYPAS-DC67DF04` | raw memory effect / ownership boundary の根本 issue。 |
| `ISS-20260427T152958303Z-MEMPTR-AND-REGIONTOKEN-LACK-COMPILER-0BC8ECDF` | `MemPtr` / owner token / provenance 分離の中心 issue。 |
| `ISS-20260429T040748194Z-RUST-COMPILER-DIAGNOSTICS-ARE-NOT-AL-1617747D` | diagnostic enum / Resource IR mapping の親 issue。 |
| `ISS-20260430T135243330Z-RESOURCE-OWNER-VARIANT-PATH-BUILDER--87B356A8` | owner variant path builder の責務再集中。Resource IR の保守性リスク。 |
| `ISS-20260430T141517141Z-SELF-HOST-PARSER-CLASSIFIES-TOKENKIN-645D236B` | selfhost parser の enum/match 化 issue。 |

今回の横断 review では、上記で追跡できない新規の静的検査 issue は確認していない。

## selfhost への制約

selfhost の静的検査は、Rust 旧 checker の構造を模倣してはいけない。S3 以降は次を最低条件にする。

- typed AST / HIR から Resource IR を生成する。
- typecheck、effect check、Resource IR check の責務を分ける。
- diagnostic は `SelfhostDiagnosticCode` の下位 enum で分類する。
- move / borrow / drop / initialized cell を別々の ad-hoc pass に散らさない。
- raw memory helper を compiler core の public data structure に持ち込まない。

S1/S2 の lexer/parser/module/diagnostic は進めてよい。一方で、S3 typecheck、S4 Resource IR、S5 backend は Rust 側の final authority 化と stdlib memory model の進捗に同期して設計する必要がある。
