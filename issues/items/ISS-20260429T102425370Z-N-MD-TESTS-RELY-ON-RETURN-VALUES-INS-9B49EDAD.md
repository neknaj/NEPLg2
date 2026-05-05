---
id: ISS-20260429T102425370Z-N-MD-TESTS-RELY-ON-RETURN-VALUES-INS-9B49EDAD
title: ".n.md tests rely on return values instead of stdout assertion reports"
area: TEST
status: open
resolved: false
priority: P1
type: architecture
created: 2026-04-29
updated: 2026-04-30
target: "tests/**/*.n.md, stdlib/**/*.nepl, nodesrc/tests.js, nodesrc/run_doctest.js, stdlib/std/test.nepl"
---

# ISS-20260429T102425370Z-N-MD-TESTS-RELY-ON-RETURN-VALUES-INS-9B49EDAD: .n.md tests rely on return values instead of stdout assertion reports

## 概要

`.n.md` の assertion 系 test が `main` の返す `i32` だけで可否を表し、stdout に検査内容を出さないケースが多い。

`main` の `i32` は runner では exit code 相当として扱えるが、失敗時に「どの assertion が、どの expected/actual で落ちたか」を fixture の期待値として確認できない。selfhost でも同じ `.n.md` を使うには、assertion report を stdout に出し、exit code は可否だけを表す運用に統一する必要がある。

## 対象

- `tests/**/*.n.md, stdlib/**/*.nepl, nodesrc/tests.js, nodesrc/run_doctest.js, stdlib/std/test.nepl`

## 根拠

- 2026-04-29 時点の調査では、`tests` / `tutorials` / `stdlib` の doctest 1481 件中、`ret:` を持つものは 719 件、stdout 期待値を持つものは 98 件である。
- `ret:` だけで stdout/stderr を持たない doctest は 710 件ある。
- `std/test`、`checks_exit_code`、`assert_*` などを使う assertion 系 doctest は 227 件ある。そのうち 116 件は `ret:` だけで、stdout report を期待していない。
- `checks_exit_code` は 171 箇所で使われている一方、`checks_print_report` は 101 箇所であり、検査結果の表示を伴わない assertion suite が残っている。
- `nodesrc/tests.js` と `nodesrc/run_doctest.js` は stdout が未指定の `std/test` case について `FAIL:` 行を検出する ad hoc な保険を持つが、これは成功時の詳細 report を仕様として固定するものではない。

## 問題

- exit code 相当の `i32` は 0/1 しか表現せず、失敗内容の情報量が不足する。
- `ret:` が「言語仕様としての戻り値期待」と「テスト成功/失敗の exit code 期待」を兼ねており、`.n.md` manifest の意味が曖昧である。
- stdout report を fixture に固定しないため、Rust runner と selfhost runner の assertion 表示、集約順、failure formatting の差異を検出できない。
- `std/test` を使っていても `checks_print_report` を呼ばない test があり、CI 上の失敗時に詳細確認がしにくい。

## 影響

- selfhost runner へ `.n.md` を共通利用するとき、Rust 側と selfhost 側が同じ exit code を返しても、失敗 detail や report format の互換性が確認できない。
- test failure の原因調査が runner log や local reproduction に依存し、`.n.md` 単体を読んでも期待される assertion report が分からない。
- 将来 `ret:` の意味を拡張すると、言語戻り値 test と exit code test が衝突する。

## 修正方針

- `.n.md` manifest に `exit_code:` を追加し、process / WASI / selfhost CLI の終了可否は `exit_code:` で表す。`ret:` は言語レベルの戻り値を検証する場合に限定する。
- assertion suite は stdout に deterministic な report を出す。標準形は `std/test` の report helper を通し、最後に `test_report_exit_code` 相当の helper で 0/1 を返す。
- `std/test` を import する assertion-style doctest について、stdout report なしの `ret:` だけ運用を runner か lint で検出する。
- 既存 fixture は一括置換ではなく、`std/test` 再設計後の安定 API に合わせて段階的に移行する。
- `core` target のように stdout を持たない層は、assertion report 必須の対象から分ける。core-only の primitive semantics は `ret:` または compile diagnostic で扱い、std stdout report と混同しない。

## 検証

- `parser.ts` / `parser.js` の metadata parser が `exit_code:` と `diag_code:` を保持する regression を追加する。
- `nodesrc/tests.js` と `nodesrc/run_doctest.js` が同じ expectation logic で `exit_code` / `stdout` / `stderr` / `diag_code` を検査することを確認する。
- `std/test` を使う代表 fixture を stdout report + exit code 期待に移行し、失敗時に stdout diff で assertion detail が見えることを確認する。

## 2026-04-29 進捗メモ

`ISS-20260429T105448222Z-N-MD-RUNNER-LACKS-EXIT-CODE-METADATA-5A5AEFD1` で、`.n.md` parser / focused runner / aggregate runner に `exit_code:` metadata を追加した。

この issue の残りは、既存 assertion suite の `ret:` 代用を stdout report + `exit_code:` へ移行することと、stdout report 省略を検出する lint / runner policy の追加である。

## 2026-04-29 進捗メモ 2

`ISS-20260429T102809685Z-STDLIB-ASSERT-API-MIXES-ASSERTION-RE-0F17011A` で、`std/test` に `AssertionStatus` / `AssertionKind` / `TestAssertion` / `TestReport` ベースの structured report API を追加し、`tests/stdlib/std_test_collect.n.md` を canonical `test_report_*` + `stdout:` + `exit_code:` fixture へ更新した。

この issue の残りは、既存の `checks_*` / `check_*` 中心の assertion suite を canonical `test_report_*` API へ移行し、report 省略を lint / runner policy で拒否することである。

## 2026-04-30 direct std/test assertion discard subcase

`tests/stdlib/hash_collection_rehash.n.md` の HashMap doctest 修正中に、direct assertion へ `std/test::assert_eq_i32` を使って戻り値を捨てると `TestAssertion` owner obligation が残ることを確認した。

原因:

- `std/test::assert_eq_i32` は report 集約用 API で、`TestAssertion` を返す。
- 返した `TestAssertion` は `checks_push` / `test_report_add` / `run_checks` などに渡して消費する必要がある。
- 即時失敗型の assertion を書きたい doctest では `core/test::assert_eq_i32` を import すべきで、`std/test` の assertion 値を単に捨てるのは API 誤用である。

今回の HashMap rehash doctest は `core/test` に切り替えて解消した。残件として、`std/test` を import して `assert_*` / `check_*` の戻り値を report へ集約しない fixture を lint で検出する方針を追加する。

## 2026-04-30 std/test assertion discard source policy

`ISS-20260429T231611047Z-STD-TEST-ASSERTION-DISCARD-SOURCE-PO-B9226736` で、direct discard subcase の source policy を追加した。

この policy は `std/test` を import する `.n.md` / NEPL doc-comment doctest で、`assert_*` / `check_*` を semicolon-terminated bare statement として捨てる書き方を禁止する。helper 関数が assertion を末尾式として返し、caller が report へ集約する書き方は許可する。

policy 追加時に見つかった既存の direct discard は、該当 stdlib doc-comment doctest 側も `checks_push` / `checks_exit_code` へ移行した。

この issue 全体の残件は、既存 assertion suite の `ret:` 代用を stdout report + `exit_code:` へ段階的に移行すること、および report 省略のより広い lint / runner policy を整備することである。

## 2026-04-30 getting_started tutorial exit_code migration

`ISS-20260430T123220209Z-GETTING-STARTED-TUTORIALS-USE-RET-FO-0BE9531F` で、`tutorials/getting_started` の assertion-style doctest metadata を `ret: 0` から `exit_code: 0` へ移行した。

各 tutorial は既に `checks_print_report` で stdout に検査結果を出していたため、今回の変更は `ret:` を runner success value として使う曖昧さを取り除くものに限定した。`rg -n "^ret:" tutorials/getting_started` で残存なし、代表 tutorial doctest 4 件で `exit_code` metadata が runner に解釈されることを確認した。

## 2026-04-30 getting_started ret metadata source policy

`ISS-20260430T124444101Z-GETTING-STARTED-TUTORIAL-RET-METADAT-8284AF7A` で、`tutorials/getting_started` に `ret:` metadata が再混入しないよう `nodesrc/test_tutorial_getting_started_current_style.js` へ source policy を追加した。

これにより getting_started tutorial は stdout report + `exit_code:` の契約を維持し、`ret:` を process success/failure の代用へ戻す regression は CI の source policy で検出できる。

検証:

- `node nodesrc/test_tutorial_getting_started_current_style.js`
- `rg -n "^ret:" tutorials/getting_started`: no matches

## 2026-04-30 stdlib option report migration

`ISS-20260430T124826467Z-STDLIB-OPTION-DOCTEST-USES-STD-TEST--93C68BCD` で、`stdlib/tests/option.n.md` を stdout assertion report + `exit_code: 0` へ移行した。

この fixture は `std/test` の `checks_push` で10件の assertionを集約していたが、`checks_print_report` を呼ばず `ret: 0` だけで成功を表していた。今回の対応で `checks_print_report` のstdoutをfixtureに固定し、`checks_exit_code shown` を返す形にした。

検証:

- `node nodesrc/tests.js -i stdlib/tests/option.n.md --no-tree -o tmp/stdlib-option-report-agent1.json -j 1 --dist web/dist`: total=1, passed=1

## 2026-04-30 stdlib result report migration

`ISS-20260430T125244690Z-STDLIB-RESULT-DOCTEST-USES-STD-TEST--F99DF5C9` で、`stdlib/tests/result.n.md` を stdout assertion report + `exit_code: 0` へ移行した。

この fixture も `std/test` の `checks_push` で13件の assertionを集約していたが、`checks_print_report` を呼ばず `ret: 0` だけで成功を表していた。今回の対応で `checks_print_report` のstdoutをfixtureに固定し、`checks_exit_code shown` を返す形にした。

検証:

- `node nodesrc/tests.js -i stdlib/tests/result.n.md --no-tree -o tmp/stdlib-result-report-agent1.json -j 1 --dist web/dist`: total=1, passed=1

## 2026-04-30 stdlib math report metadata migration

`ISS-20260430T125624124Z-STDLIB-MATH-DOCTEST-KEEPS-RET-METADA-35B82C86` で、`stdlib/tests/math.n.md` の metadata を `exit_code: 0` へ移行し、既存の `checks_print_report` stdout をfixtureに固定した。

この fixture は report 出力自体は行っていたが、`ret: 0` のままで stdout expectation がなかったため、report format の regression を `.n.md` で比較できなかった。今回の対応で27件の assertion reportを stdout として固定した。

検証:

- `node nodesrc/tests.js -i stdlib/tests/math.n.md --no-tree -o tmp/stdlib-math-report-agent1.json -j 1 --dist web/dist`: total=1, passed=1

## 2026-04-30 stdlib cast report metadata migration

`ISS-20260430T125946580Z-STDLIB-CAST-DOCTEST-KEEPS-RET-METADA-B6A0E1B5` で、`stdlib/tests/cast.n.md` の metadata を `exit_code: 0` へ移行し、既存の `checks_print_report` stdout をfixtureに固定した。

この fixture も report 出力自体は行っていたが、`ret: 0` のままで stdout expectation がなかったため、report format の regression を `.n.md` で比較できなかった。今回の対応で11件の assertion reportを stdout として固定した。

検証:

- `node nodesrc/tests.js -i stdlib/tests/cast.n.md --no-tree -o tmp/stdlib-cast-report-agent1.json -j 1 --dist web/dist`: total=1, passed=1

## 2026-04-30 stdlib list report metadata migration

`ISS-20260430T130514815Z-STDLIB-LIST-DOCTESTS-KEEP-RET-METADA-7DD9F2F5` で、`stdlib/tests/list.n.md` の2件の doctest metadata を `exit_code: 0` へ移行し、既存の `checks_print_report` stdout をfixtureに固定した。

この fixture も report 出力自体は行っていたが、`ret: 0` のままで stdout expectation がなかったため、report format の regression を `.n.md` で比較できなかった。今回の対応で14件と10件の assertion reportを stdout として固定した。

検証:

- `node nodesrc/tests.js -i stdlib/tests/list.n.md --no-tree -o tmp/stdlib-list-report-agent1.json -j 1 --dist web/dist`: total=2, passed=2

## 2026-04-30 stdlib vec report and runtime migration

`ISS-20260430T131836940Z-STDLIB-VEC-DOCTESTS-STILL-EXCEED-RUN-44D56F6B` で、`stdlib/tests/vec.n.md` の大きい `std/test` doctest を10件の focused doctest へ分割し、各 case を stdout assertion report + `exit_code: 0` へ移行した。

この fixture は以前の分割後も `doctest#2` が約127秒、`doctest#3` が約68秒かかり、aggregate runner の 60秒 per-case budget を超えていた。今回の対応で全 case を 22.9秒から34.0秒に収め、`ret:` と `checks_exit_code checks` の残存をなくした。

検証:

- `node nodesrc/tests.js -i stdlib/tests/vec.n.md --no-tree -o tmp/stdlib-vec-report-agent1.json -j 1 --dist web/dist`: total=10, passed=10

## 2026-04-30 stdlib hashset report metadata migration

`ISS-20260430T132753960Z-STDLIB-HASHSET-DOCTESTS-KEEP-RET-MET-4238B95D` で、`stdlib/tests/hashset.n.md` の2件の doctest metadata を `exit_code: 0` へ移行した。

main case は既存の8件 assertion reportを stdout として固定し、free smoke case は `ret: 0` だけの smoke ではなく、free 後に最小の `std/test` report を出す形へ揃えた。

検証:

- `node nodesrc/tests.js -i stdlib/tests/hashset.n.md --no-tree -o tmp/stdlib-hashset-report-agent1.json -j 1 --dist web/dist`: total=2, passed=2

## 2026-04-30 stdlib string report and helper contract migration

`ISS-20260430T133512620Z-STDLIB-STRING-DOCTESTS-KEEP-OLD-STD--4220AD4F` で、`stdlib/tests/string.n.md` の `std/test` doctest 5件を stdout assertion report + `exit_code: 0` へ移行した。

調査時点で `string_find_byte_index` は `Result<(),str>` helper から `assert_eq_i32` の `TestAssertion` を返しており、structured assertion API との型不整合で compile fail していた。これは `check_eq_i32` に戻し、helper signature と `checks_push` の Result overload に合う形へ修正した。

検証:

- `node nodesrc/tests.js -i stdlib/tests/string.n.md --no-tree -o tmp/stdlib-string-report-agent1.json -j 1 --dist web/dist`: total=9, passed=9

## 2026-04-30 string char report migration and owner-model split note

`ISS-20260430T134239951Z-STRING-CHAR-DOCTESTS-REUSE-MOVED-STR-200F01C2` で、`tests/stdlib/string_char.n.md` の4件を stdout assertion report + `exit_code: 0` へ移行した。

調査時点で先頭2件は同じ `str` local を by-value char observer API に繰り返し渡しており、現行 Resource IR の unresolved fallible owner effect により `resource.owner.reserved` で compile fail していた。fixture は各 assertion に fresh literal を渡す形にして、現在の owner gate を弱めずに回帰テストを通した。

timeout 調査では compile-only 計測が runtime 付き duration とほぼ一致し、原因は生成 wasm の実行や UTF-8 algorithm ではなく、stdlib 込み compile だった。builder case は並列 compiler load で60秒 case timeout に届いたため、string builder と byte builder の2件へ分割した。

一方で、`str` は stdlib 上 Copy な非所有 view と説明されているため、この owner model との不一致は `ISS-20260430T135134835Z-STR-COPY-VIEW-CONTRACT-CONFLICTS-WIT-0998304C` として切り出した。

検証:

- `node nodesrc/tests.js -i tests/stdlib/string_char.n.md --no-tree -o tmp/tests-stdlib-string-char-agent1-j4.json -j 4 --dist web/dist`: total=4, passed=4

## 2026-05-05 selfhost CLI reporter report migration

`ISS-20260505T065154326Z-SELFHOST-CLI-REPORTER-DOCTESTS-OMIT--BEB95AC0` で、`tests/stdlib/selfhost_cli_reporter.n.md` の assertion-style doctest 2 件を stdout assertion report + `exit_code: 0` へ移行した。

この fixture は selfhost diagnostics の human/json reporter 境界を検証するため、Rust runner と selfhost runner の parity 上、成功時 report も固定する必要がある。`std/test` の `checks_print_report` を通した `Checked [ok,ok]` stdout を fixture 化し、writer doctest は既存の JSON stdout / human stderr を維持しつつ `ret: 0` を `exit_code: 0` に変更した。

検証:

- `node nodesrc/tests.js -i tests/stdlib/selfhost_cli_reporter.n.md --no-tree -o tmp/selfhost-cli-reporter-report-agent1.json -j 1 --dist web/dist`: total=3, passed=3

## 2026-05-05 selfhost CLI driver report migration blocker

`tests/stdlib/selfhost_cli_driver.n.md` にも同種の `ret:` / report 省略が残っているため、`ISS-20260505T065610900Z-SELFHOST-CLI-DRIVER-DOCTESTS-OMIT-ST-E638CB58` として分離した。

この fixture は移行試作後の focused run が 3 件とも 60 秒 timeout し、個別 `run_doctest -n 1` も `NEPL_TEST_CASE_TIMEOUT_MS=180000` で shell 240 秒 timeout になった。未検証の fixture 変更は commit せず、先に selfhost driver fixture の粒度または compile/static-check cost を切り分ける。
