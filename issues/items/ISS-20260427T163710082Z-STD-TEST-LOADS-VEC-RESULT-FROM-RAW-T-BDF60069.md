---
id: ISS-20260427T163710082Z-STD-TEST-LOADS-VEC-RESULT-FROM-RAW-T-BDF60069
title: "std/test raw temp ownership and Result Copy regression"
area: stdlib
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-04-27
updated: 2026-04-28
target: "stdlib/std/test.nepl, stdlib/core/result.nepl, nepl-core/src/typecheck.rs, nepl-core/src/types.rs, tests/stdlib/byte_builder.n.md"
---

# ISS-20260427T163710082Z-STD-TEST-LOADS-VEC-RESULT-FROM-RAW-T-BDF60069: std/test raw temp ownership and Result Copy regression

## 概要

`checks_print_machine` と `finish_checks` は `Vec<Result<(),str>>` を temporary raw memory に退避した後、同じ raw place から `checks_summary` / `checks_has_err` / 最終 read-back のために複数回 by-value load している。raw memory ownership 検査が正確になると、各 `load<Vec<Result<(),str>>> checks_mem` は同じ owner を再生成するため `D3100` になる。

## 対象

- `stdlib/std/test.nepl, stdlib/core/result.nepl, tests/stdlib/byte_builder.n.md`

## 根拠

- `stdlib/std/test.nepl:705` / `707` / `712` で `checks_print_machine` が同じ `checks_mem` から `Vec<Result<(),str>>` を複数回 load している。
- `stdlib/std/test.nepl:777` / `779` でも `finish_checks` が同じ pattern を持つ。
- `tests/stdlib/byte_builder.n.md` の 3 doctest は、この helper 経由で `error[D3100]: use of moved raw memory place: checks_mem` になった。
- compiler 側では raw aggregate field read の誤検出と branch merge の悪化は解消済みであり、この D3100 は non-Copy `Vec` owner の by-value load が複数回行われている実際の所有権違反として残る。

## 問題

`checks_summary` と `checks_has_err` が `Vec<Result<(),str>>` を by-value で受け取るため、呼び出し側は raw temp から Vec owner を何度も取り出している。これは shallow owner の複製であり、最後に返す owner と helper に渡した owner が同じ backing storage を指し得る。

## 影響

`byte_builder` doctest と、この std/test helper を使う test path が compile-time D3100 で停止する。検査を迂回すると、同じ Vec storage に対する複数 owner が作られ、drop/free 設計の前提を壊す。

## 修正方針

`std/test` helper を、Vec owner を複数回 by-value load しない形へ設計し直す。候補は以下。

- `len` / `data` を一度だけ取り出して summary / has_err loop へ渡す。
- observer helper を参照または raw slice 相当で表現し、Vec owner は最後まで 1 つだけ保持する。
- どうしても by-value helper を維持する場合は、owner を線形に受け渡して返す API に変える。

## 対応結果

- `checks_has_err_parts` / `checks_summary_parts` を追加し、`Vec<Result<(),str>>` owner を raw memory から読み出さず、取り出し済みの `data` pointer と `len` だけで観察できるようにした。
- `checks_print_machine` と `finish_checks` は `checks_mem` から owner を複数回 load せず、field read で `len` / `data` を取得して parts helper を呼ぶようにした。
- `Result<T,E>` は payload がどちらも Copy なら Copy として扱えるよう、`impl<T: Copy,E: Copy> Clone/Copy for Result<T,E>` を追加した。`Result<(),str>` をテスト status として複数回観察する用途はこれで shallow owner move ではなく Copy read になる。
- non-Copy payload の `Result<LocalToken,str>` は引き続き二重利用で move error になる回帰テストを追加した。

## 2026-04-28 再発確認と追加対応

関数戻り値・aggregate field・dealloc の raw ownership 検査が強化された後、`finish_checks` の `checks_mem` は owner を複数回 load しなくなっていた一方で、`Vec<Result<(),str>>` owner を raw temp 内に initialized のまま残して `dealloc_raw` していることが再検出された。

- 変更前再現: `tests/stdlib/byte_builder.n.md` の doctest 3 件が `error[D3100]: deallocating raw memory place containing non-Copy value: checks_mem` で失敗した。
- 根本原因: `len` / `data` の field read は観察であり、raw temp 内の `Vec` owner を consume しない。したがって storage-only `dealloc_raw` は live owner discard になる。
- 追加対応: `checks_has_err` / `checks_summary` / `finish_checks` は観察完了後に raw temp から owner を 1 回だけ読み戻し、temporary storage を解放してから `Vec` の data storage を `v::free<Result<(),str>>` で解放する。
- `checks_print_machine` / `checks_print_human` は従来どおり owner を 1 回だけ読み戻して返すため、呼び出し側が `checks_exit_code` へ線形に渡せる。

## 検証

stdlib 修正後に `node nodesrc/tests.js -i tests/stdlib/byte_builder.n.md --no-tree -o tmp/std-test-vec-result-raw-temp.json -j 1` を実行し、`D3100` が消えることを確認する。

## 実施した検証

- 変更前再現: `node nodesrc/tests.js -i tests/stdlib/byte_builder.n.md --no-tree -o tmp/std-test-vec-result-before.json -j 1`: `D3100` で 3 failed
- `node nodesrc/tests.js -i stdlib/core/result.nepl --no-tree -o tmp/result-copy-impl-regression6.json -j 1`: `total=7`, `passed=7`
- `node nodesrc/tests.js -i stdlib/std/test.nepl --no-tree -o tmp/std-test-single-owner-final.json -j 1`: `total=12`, `passed=12`
- `node nodesrc/tests.js -i tests/stdlib/byte_builder.n.md --no-tree -o tmp/std-test-vec-result-after-final.json -j 1`: `total=3`, `passed=3`
- 追加再現: `node nodesrc/tests.js -i tests/stdlib/byte_builder.n.md --no-tree -o tmp/std-test-checks-mem-regression-before.json -j 1`: `total=3`, `failed=3`, D3100 `deallocating raw memory place containing non-Copy value: checks_mem`
- 追加確認: `node nodesrc/tests.js -i tests/stdlib/byte_builder.n.md --no-tree -o tmp/std-test-checks-mem-regression-after.json -j 1`: `total=3`, `passed=3`
- 追加確認: `node nodesrc/tests.js -i stdlib/std/test.nepl --no-tree -o tmp/std-test-checks-mem-stdlib-test-after.json -j 1`: `total=12`, `passed=12`
- 追加確認: `node nodesrc/tests.js -i tests/stdlib/std_test_collect.n.md --no-tree -o tmp/std-test-collect-checks-mem-after.json -j 1`: `total=3`, `passed=3`

## 2026-04-28 core / stdlib 全体レビューでの再オープン

`origin/main` の `8d0c6ab` 取り込み後に `stdlib/core/result.nepl` の回帰テストを再実行したところ、この issue で追加した non-Copy payload の `Result<LocalToken,str>` 二重利用 compile_fail が再び成功していた。

- 再現: `node nodesrc/tests.js -i stdlib/core/result.nepl --no-tree -o tmp/result-audit-20260428.json -j 1`
- 結果: `total=7`, `passed=6`, `failed=1`
- 失敗: `stdlib\core\result.nepl::doctest#6`
- エラー: `expected compile_fail, but compiled successfully`

この doctest は `Result<T,E>` の `impl<T: Copy,E: Copy> Copy` 追加時に、non-Copy payload では `Result` 全体も non-Copy のままであることを監視する目的で入れたもの。現状では `LocalToken` fixture の Copy 判定前提が compiler 側変更で変わった可能性と、generic `Copy` impl の適用条件が広すぎる可能性の両方があるため、表面上 doctest を緩めず、`Result<LocalToken,str>` が本当に Copy でよい型なのかを `is_copy_impl_eligible` / trait bound 解決 / function pointer field の仕様から確認する。

同時に `node nodesrc/tests.js -i stdlib --no-tree -o tmp/stdlib-audit-20260428.json -j 4` でも `total=422`, `passed=421`, `failed=1` としてこの 1 件だけが stdlib 全体の失敗になっている。self-host compiler の AST / diagnostic / collection は `Result` を大量に使うため、payload の線形性が崩れると selfhost 着手後の所有権バグを検出できなくなる。

## 2026-04-28 trunk build 後の訂正

上記の再オープン判断は、remote main の compiler 修正を取り込んだ後に `trunk build` を実行していない古い `web/dist` で確認していたための誤検出だった。

- `trunk build`: 成功
- `node nodesrc/tests.js -i stdlib\core\result.nepl --no-tree -o tmp\result-after-trunk-20260428.json -j 1`: `total=7`, `passed=7`, `failed=0`

したがって `Result<LocalToken,str>` の non-Copy 回帰テストは、最新 build では再発していない。この issue は再度 fixed / resolved とする。今後 Rust 側の `nepl-core` を merge した後は、stdlib doctest を判定する前に `trunk build` を必ず挟む。
