---
id: ISS-20260505T065610900Z-SELFHOST-CLI-DRIVER-DOCTESTS-OMIT-ST-E638CB58
title: "selfhost CLI driver doctests omit stdout assertion reports"
area: TEST
status: fixed
resolved: true
priority: P1
type: test
created: 2026-05-05
updated: 2026-05-05
target: tests/stdlib/selfhost_cli_driver.n.md
---

# ISS-20260505T065610900Z-SELFHOST-CLI-DRIVER-DOCTESTS-OMIT-ST-E638CB58: selfhost CLI driver doctests omit stdout assertion reports

## 概要

selfhost_cli_driver の std/test assertion doctests が checks_print_report を呼ばず ret: 0 だけで成功を表している。

## 対象

- `tests/stdlib/selfhost_cli_driver.n.md`

## 根拠

- `tests/stdlib/selfhost_cli_driver.n.md` の success / missing file doctest は `std/test` の `checks_push` で assertion suite を作るが、`checks_print_report` を呼ばず `ret: 0` だけで成功を表している。
- missing input doctest は JSON stdout fixture を持つが、process success metadata が `ret: 0` のままで、exit code と言語戻り値の意味が混ざっている。
- 2026-05-05 に stdout report + `exit_code:` へ移行する試作を行ったが、`node nodesrc/tests.js -i tests/stdlib/selfhost_cli_driver.n.md --no-tree -o tmp/selfhost-cli-driver-report-agent1.json -j 1 --dist web/dist` は 3 件とも 60 秒 timeout、個別 `run_doctest -n 1` も `NEPL_TEST_CASE_TIMEOUT_MS=180000` で shell 240 秒 timeout になった。未検証の fixture 変更は commit せず、timeout/粒度問題を含めてこの issue に残す。

## 問題

selfhost_cli_driver の std/test assertion doctests が checks_print_report を呼ばず ret: 0 だけで成功を表している。

## 影響

selfhost CLI driver の exit code / diagnostics contract が stdout assertion report に固定されず、selfhost runner parity と失敗時の詳細比較が弱い。

## 修正方針

assertion doctests を checks_print_report + stdout fixture + exit_code: 0 へ移行し、stdout/stderr を直接検証する doctest も ret: ではなく exit_code: を使う。ただし現状の 3 ケースは default timeout どころか長めの focused run でも検証できないため、先に fixture 粒度または selfhost driver compile/static-check cost を切り分け、検証可能な単位に分割してから移行する。

## 検証

tests/stdlib/selfhost_cli_driver.n.md を focused run し、3 doctest が通ることを確認する。移行後は `ret:` が残らず、assertion-style doctest には deterministic stdout report があることを確認する。

## 2026-05-05 codegen timeout 切り分け

未変更の `tests/stdlib/selfhost_cli_driver.n.md::doctest#2` を再計測したところ、`node nodesrc/run_doctest.js -i tests/stdlib/selfhost_cli_driver.n.md -n 2 --dist web/dist` は 180 秒で shell timeout した。

同じ doctest source を抽出して native CLI で確認すると、`target\debug\nepl-cli.exe --check -i <tmp> --target std --stdlib-root stdlib` は約 5.4 秒で `Check successful` になった。一方で `target\debug\nepl-cli.exe -i <tmp> --target std --stdlib-root stdlib --emit wasm` は 240 秒 timeout に到達した。

このため、stdout report 移行の未検証差分を入れるのではなく、post-check の monomorphize / wasm codegen / backend 側の性能・到達関数集合問題として `ISS-20260505T081814569Z-SELFHOST-CLI-DRIVER-DOCTEST-CODEGEN--052EB57C` を追加した。この issue は codegen timeout 解消後に再開する。

## 2026-05-06 対応

- `tests/stdlib/selfhost_cli_driver.n.md` の assertion-style doctest 2 件を `checks_print_report` + stdout fixture + `exit_code: 0` へ移行した。
- missing input doctest は既存 JSON stdout fixture を維持し、process success metadata を `ret: 0` から `exit_code: 0` へ移行した。
- stdout report 移行の再開時に露出した `stdlib/neplg2/cli/args/parse.nepl` の raw `load<str>` は、parser 側に raw boundary を追加せず、`Vec<str>` borrow と `alloc/collections/vec::get` による境界付き access へ修正した。

## 2026-05-06 検証結果

- `node nodesrc/run_doctest.js -i tests/stdlib/selfhost_cli_driver.n.md -n 1 --dist web/dist`: pass, stdout report `Checked [ok,ok]` を確認。
- `node nodesrc/run_doctest.js -i tests/stdlib/selfhost_cli_driver.n.md -n 2 --dist web/dist`: pass, JSON stdout fixture を確認。
- `node nodesrc/run_doctest.js -i tests/stdlib/selfhost_cli_driver.n.md -n 3 --dist web/dist`: pass, stdout report `Checked [ok,ok]` を確認。
- `node nodesrc/tests.js -i tests/stdlib/selfhost_cli_driver.n.md -o tmp/selfhost_cli_driver_after_notree.json -j1 --dist web/dist --no-tree`: total=3, passed=3, failed=0。
- `node nodesrc/tests.js -i tests/stdlib/selfhost_cliarg_parser.n.md -o tmp/selfhost_cliarg_parser_after_notree.json -j1 --dist web/dist --no-tree`: total=10, passed=10, failed=0。

補足: `--with-tree` 相当の同梱実行では既存の `tests/compiler/tree/semantics_vfs_cross_file_definition_path` / `name_resolution_vfs_cross_file_definition_path` が definition path assertion で失敗するが、今回の selfhost CLI parser / driver 変更とは独立している。
