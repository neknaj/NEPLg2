# 横断レビュー: diagnostics / tests / docs

対象 commit: `f108cebd`

参照 Actions: `25157230630`

## 結論

diagnostic は Rust compiler と selfhost の両方で、内部 enum と外部 stable string の境界を分ける方向が正しい。`.n.md` test は Rust と selfhost で共通運用できる可能性が高いが、現行の return value 中心の assertion では失敗内容を追えない。stdout report と exit code を分ける test contract を stdlib assert と runner の両方で固定する必要がある。

docs は active な設計文書が増えており、Resource IR、diagnostic redesign、stdlib memory model、selfhost plan の方向は概ね揃っている。一方で README / tutorial / examples は、Actions failure と current stdlib API への追従がまだ完了していない。

## 進捗状況

| 領域 | 状況 | review |
|---|---|---|
| Rust diagnostics | 実装中 | `DiagnosticCode` 階層 enum は導入済み。Resource IR mapping の typed 化が継続課題。 |
| selfhost diagnostics | 実装中 | `SelfhostDiagnosticCode` は導入済み。parser/checker parity と variant 追加が今後必要。 |
| `.n.md` runner | 移行中 | Rust/selfhost 共通運用の計画はあるが、return value 依存の assertion が残る。 |
| stdlib assert | 再設計対象 | stdout report / exit code separation の contract が必要。 |
| tutorials | 更新対象 | Actions で `44 total / 21 passed / 23 failed`。現行 stdlib API と owner contract に未追従。 |
| examples | 部分追従 | obsolete API 修正は進んだが、web examples sync と source policy 継続監視が必要。 |
| docs | 実装中 | 設計 doc は増えた。レビュー対象 commit と Actions 根拠を明記する運用が必要。 |

## diagnostic code 方針

`doc/neplg2/compiler_diagnostics_redesign_plan.md` の方針は妥当である。内部表現は `DiagnosticCode` と下位 enum にし、文字列は `as_str()` による表示・JSON・doctest 比較だけに限定する。数値 ID や自由文字列を内部で持ち回る設計は、Resource IR の owner / cell / borrow / raw provenance の意味を失わせる。

Rust 側では D0/D1 が大きく進み、active call site から旧 `diag_id` や code-less diagnostic を減らしている。残る焦点は、Resource IR diagnostic mapping の粒度と、note/help/related label への意味情報分離である。

selfhost 側では `SelfhostDiagnosticCode` の導入が正しい。今後は Rust 側の diagnostic code taxonomy と乖離しないよう、lexer / parser / resolver / checker / resource / backend で下位 enum を増やす。selfhost の diag ID を別体系の raw string として設計してはいけない。

## `.n.md` 共通 test

`.n.md` は Rust compiler と selfhost compiler の共通 fixture になり得る。現状の課題は、main が exit code 相当の `i32` を返すだけの形式では、assertion failure の詳細を runner が十分に確認できないことである。

必要な contract:

- assertion は stdout に deterministic report を出す。
- exit code は test case 全体の成功 / 失敗だけを表す。
- failure report は expected / actual / location / assertion kind を含む。
- `should_panic` / `compile_fail` / `run` の結果分類を runner が同じ形式で扱う。
- Rust runner と selfhost runner は同じ `.n.md` block metadata を読む。
- diagnostic code は stable string で比較するが、compiler 内部では enum を保持する。

この方針は `ISS-20260429T102425370Z-N-MD-TESTS-RELY-ON-RETURN-VALUES-INS-9B49EDAD` と `doc/neplg2/nmd_assert_output_plan.md` で追跡済みである。

## Actions evidence

対象 Actions run `25157230630` の test 状況は次の通り。

| job / artifact | 結果 |
|---|---|
| `build` | success |
| `compile-test` | success |
| `llvm-test` | success |
| `Source policy regressions` | success |
| `stdlib-test` | failure: `415 total / 232 passed / 173 failed / 10 errored` |
| `nmd-doctest` | failure: `1034 total / 812 passed / 185 failed / 37 errored` |
| `tutorials-test` | failure: `44 total / 21 passed / 23 failed` |
| `wasi-test` | failure: nmd と同傾向 |
| dual backend tests | failure: runtime / Resource IR / stdlib failure が混在 |

review では local test を main の test 状況の根拠にしていない。local 実行は、local code change の commit 前確認に限定する。

## docs / tutorial / examples

tutorial は現行 NEPLg2 仕様と stdlib owner contract に対して古い。Actions では getting started の序盤から `stdio_write_fd_mem_result` の owner failure を拾っている。tutorial を単に文面更新するだけでは不十分で、stdlib assert / stdout report / stdio Result API の移行と一緒に直す必要がある。

examples は obsolete collection API への追従が進んだが、web examples sync と current stdlib API の継続確認が必要である。source policy regression が Actions で成功していることは良い防壁だが、policy の対象外に新しい selfhost / Resource IR / stdlib unsafe pattern が出た場合は runner を追加する。

docs は `doc/neplg2/` に設計文書が増えており、static check、diagnostics、stdlib memory model、selfhost plan の方向性は揃っている。今後の review doc では次を守る。

- 対象 commit を明記する。
- test 状況は GitHub Actions run ID と artifact で示す。
- local test は review evidence として扱わない。
- new issue を追加したら Discord に issue 内容を直接報告する。
- 最終再レビュー後に入った変更は、その review の追従対象外にする。

## 既存 issue との対応

| issue | review 判断 |
|---|---|
| `ISS-20260429T040748194Z-RUST-COMPILER-DIAGNOSTICS-ARE-NOT-AL-1617747D` | Rust/selfhost diagnostic enum parity の親 issue。 |
| `ISS-20260429T102425370Z-N-MD-TESTS-RELY-ON-RETURN-VALUES-INS-9B49EDAD` | stdout assertion report / exit code separation の中心 issue。 |
| `ISS-20260430T020255446Z-GETTING-STARTED-DOCTESTS-RELY-ON-EXI-8902362D` | tutorial doctest の stdout report 移行。 |
| `ISS-20260430T025134838Z-WEB-DIST-TS-LANGUAGE-ANALYSIS-DROPS--D70C7D62` | web/editor diagnostic stable code の伝搬問題。 |
| `ISS-20260430T064057030Z-STATIC-CHECK-SOURCE-POLICY-RUNNER-MI-812E7A30` | source policy regression runner の範囲拡張。 |

今回の diagnostics / tests / docs review では、上記で追跡できない新規 issue は確認していない。
