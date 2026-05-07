# レビュー妥当性の再レビュー

## 目的

この文書は、今回の進捗確認及び総レビューの内容が、現行 source tree と issue registry に照らして妥当かを再確認した結果である。レビュー本文の作成時と同じく、この段階までは前回レビュー本文の結論を参照していない。

確認した観点:

- source の実装状態と review conclusion が一致しているか。
- open issue に既に記録済みの問題を重複 issue 化していないか。
- CI 状態の扱いが過大評価または過小評価になっていないか。
- selfhost readiness の判断が、静的検査とメモリ安全の必達方針に反していないか。
- review の結論が「表面修正」ではなく根本原因へ向いているか。

## 再確認した根拠

### Rust compiler static check

`nepl-core/src/compiler.rs` の pipeline では、`check_module_with_source_map` が codegen preparation 経路を通り、Resource IR static check と drop elaboration bridge gate を実行する構造になっている。`run_resource_static_check` は initialized move、borrow lifetime、effect boundary、owner obligation などの checker を呼んでいる。

このため、review の「Rust compiler の静的検査経路は改善している」という結論は妥当である。

ただし、これは「静的検査が完成した」という意味ではない。stdlib raw memory と Resource IR authority の接続は別の未完了課題として残っており、review ではその制限も明記している。

### Resource IR model

`nepl-core/src/resource/**` には `ResourceOp`, `EffectOp`, `CellState`, `OwnerState`, `BorrowState` などの enum model と typed ID が存在する。compiler 側診断も `ResourceDiagnosticCode` 系へ接続されている。

このため、review の「数値や文字列ではなく enum/typed state へ進んでいる」という評価は妥当である。

注意点として、Resource IR model があることと、stdlib の全 API が Resource IR に正しく閉じていることは別である。review はこの差を区別しているため、過大評価にはなっていない。

### Diagnostic ID

`DiagnosticCode` と領域別 enum、`ALL_DIAGNOSTIC_CODES` registry、`DiagnosticSpec`/`Diagnostic` の code field が存在する。文字列表現は internal authority ではなく表示や外部境界として扱える構造になっている。

このため、Rust 側 diagnostic ID の方向性を良い進捗とした判断は妥当である。

ただし selfhost 側 diagnostic ID の実装は未完了であり、open issue として残る。この点も review で P1 として扱っているため、未完了を隠していない。

### stdlib test と `.n.md`

`stdlib/std/test` には `AssertionStatus`, `AssertionKind`, `TestAssertion`, `TestReport`, `test_report_print_stdout`, `test_report_exit_code` がある。一方で `finish_checks` や `result_exit_code` のような旧互換経路も残っている。

このため、review の「structured assertion/report の方向は良いが、`.n.md` の stdout assertion report への統一は未完了」という判断は妥当である。

### stdlib memory boundary

`stdlib/core/mem.nepl` には `alloc_raw`, `dealloc_raw`, `realloc_raw` が残り、`MemPtr<T>` と `RegionToken<T>` がある一方で、compiler-owned provenance と drop obligation へ完全には接続されていない。collections には `free` があり、多くの collection が `Copy` 前提や drop traversal 未完成をコメントで明記している。

このため、review の「stdlib memory boundary が最大 P1 blocker」という判断は妥当である。これは仕様の弱点であり、helper を追加するだけの局所修正では解けない。

### selfhost typed model

`stdlib/neplg2/core/hir/hir.nepl` では `SelfhostHirExprPayload` と `SelfhostHirChildRange` が enum になっている。`stdlib/neplg2/core/resolve/name_resolver.nepl` では `SelfhostNameBinding.def_id` が `Option<SelfhostDefId>` である。`SelfhostBuiltinSignature` と `SelfhostTypeRecord` も variant payload へ寄っている。

このため、review の「直近 refactor で sentinel/shared payload debt は改善した」という結論は妥当である。

同時に、lexer raw mode は `i32` として残っている。review では selfhost が完全に方針達成済みとはせず、lexer raw/directive state の enum 化を未完了として扱っているため、判断は釣り合っている。

### CI 状態

GitHub Actions は review 時点で latest run が pending/in_progress であり、古い run は workflow concurrency により cancelled になっている。CI の最終成功を確認したとは書いていない。

このため、review の CI 記述は妥当である。今後 latest run の完了後に失敗が出た場合は、通常開発に戻った後の issue/修正対象として扱う。

## issue coverage

今回の review で確認した主要未完了点は、既存 open issue に対応している。

対応済み issue coverage:

- raw memory/effect/ownership bypass: `ISS-20260427T132406497Z-CORE-MEM-RAW-MEMORY-OPERATIONS-BYPAS-DC67DF04`
- `MemPtr`/`RegionToken` provenance: `ISS-20260427T152958303Z-MEMPTR-AND-REGIONTOKEN-LACK-COMPILER-0BC8ECDF`
- raw address escape: `ISS-20260427T152954558Z-CORE-MEM-EXPOSES-RAW-ADDRESS-ESCAPE--4185EA5D`
- dealloc/drop obligation: `ISS-20260427T164432612Z-CORE-MEM-DEALLOC-APIS-DO-NOT-ENCODE--204F1F47`
- stdlib raw-memory-backed API migration: `ISS-20260427T204839136Z-STDLIB-RAW-MEMORY-BACKED-APIS-REQUIR-E503CD84`
- collection free/drop traversal: `ISS-20260425T000000Z-RV-STDLIB-004-91534828`
- diagnostic alignment: `ISS-20260429T040748194Z-RUST-COMPILER-DIAGNOSTICS-ARE-NOT-AL-1617747D`
- `.n.md` stdout assertion report: `ISS-20260429T102425370Z-N-MD-TESTS-RELY-ON-RETURN-VALUES-INS-9B49EDAD`
- selfhost lexer enum/match gap: `ISS-20260507T151236784Z-SELFHOST-LEXER-RAW-MODES-AND-DIRECTI-B080723B`
- tutorials/examples CI gaps: `ISS-20260507T161156205Z-GETTING-STARTED-TUTORIAL-DOCTESTS-FA-A0324153`, `ISS-20260507T153812328Z-EXAMPLES-DOCTESTS-ARE-NOT-RUN-BY-CI-13ED1895`

新規 issue が必要な未記録問題は、この妥当性再レビューでは見つからなかった。既存 issue の scope が広すぎる場合は、実装着手時に child issue へ分割するのが適切である。

## review の制限

- 全ファイルの全行を逐語的に読む形式ではなく、領域別 review と source spot check を組み合わせた確認である。
- CI は `gh run list` の状態確認であり、latest run の最終結果が出る前の checkpoint が含まれる。
- 前回レビューとの差分は、今回レビューの独立性を守るため、この文書作成後に別途行う。
- local test は文書変更の pre-commit check 用途に限定し、review の test 結果判断は GitHub Actions を使う方針に従っている。

## 進捗状況

| 妥当性確認項目 | 状態 | 判断 |
| --- | --- | --- |
| source 根拠と findings の一致 | 確認済み | 主要結論は現行 source と一致 |
| open issue coverage | 確認済み | 新規 issue は不要。実装時に分割余地あり |
| CI 状態の扱い | 確認済み | latest pending/in_progress と明記しており過大評価なし |
| selfhost readiness | 確認済み | 限定開始可、本体は設計確定後という判断は妥当 |
| memory safety blocker | 確認済み | `core/mem` と collections を P1 とする判断は妥当 |
| previous review independence | 確認済み | review 完了前に前回レビュー本文は参照していない |

## 最終判断

今回の総レビューは、現行 source tree の実装状態と open issue に照らして妥当である。特に、Rust compiler の静的検査基盤を進捗として評価しつつ、stdlib memory boundary を最大 blocker として扱っている点は、型安全とメモリ安全を必達とする方針に合っている。

レビュー結果は通常開発へ戻るための判断材料として利用できる。ただし、前回レビューとの差分報告と、最新 GitHub Actions の完了結果確認は次のステップで行う。
