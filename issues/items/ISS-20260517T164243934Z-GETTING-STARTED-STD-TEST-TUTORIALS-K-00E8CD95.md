---
id: ISS-20260517T164243934Z-GETTING-STARTED-STD-TEST-TUTORIALS-K-00E8CD95
title: "Getting started std/test tutorials keep ret metadata despite stdout reports"
area: tutorials
status: fixed
resolved: true
priority: P1
type: test
created: 2026-05-17
updated: 2026-05-18
target: "tutorials/getting_started/*.n.md"
---

# ISS-20260517T164243934Z-GETTING-STARTED-STD-TEST-TUTORIALS-K-00E8CD95: Getting started std/test tutorials keep ret metadata despite stdout reports

## 概要

getting_started の std/test doctest は checks_print_report による stdout report を既に出しているが、多くが neplg2:test のまま ret: 0 を exit code 代用として残し、stdio / normalize_newlines tag と exit_code metadata を持たない。そのため stdout report fixture と process exit-code の責務分離が manifest 上で固定されず、selfhost runner と Rust runner の互換性検査が弱い。

## 対象

- `tutorials/getting_started/*.n.md`

## 根拠

- `tutorials/getting_started/02_test_harness.n.md` から `24_project_byte_output.n.md` までの `std/test` doctest 21 件が、`checks_print_report` による stdout report と `stdout:` fixture を持っていた。
- その一方で、`13_vec_basics.n.md` 以外は `neplg2:test` + `ret: 0` のままで、process exit-code と言語戻り値の metadata が分離されていなかった。
- `stdio` / `normalize_newlines` tag も欠けていたため、stdout report を検査する tutorial であることが manifest 上の契約になっていなかった。

## 問題

getting_started の std/test doctest は checks_print_report による stdout report を既に出しているが、多くが neplg2:test のまま ret: 0 を exit code 代用として残し、stdio / normalize_newlines tag と exit_code metadata を持たない。そのため stdout report fixture と process exit-code の責務分離が manifest 上で固定されず、selfhost runner と Rust runner の互換性検査が弱い。

## 影響

tutorial の検査結果は stdout に出ているように見えても、runner は ret と stdout の混在として扱うため、将来の exit_code 統一や selfhost runner で同じ挙動を保証しにくい。

## 修正方針

std/test report を持つ getting_started doctest を neplg2:test[stdio, normalize_newlines] + exit_code: 0 + stdout: へ移行し、ret: を残さない source policy を追加する。

## 検証

対象 tutorial の source policy と focused doctest を実行し、stdout report と exit_code が一致することを確認する。

## 修正内容

- `std/test` report を持つ getting_started doctest 21 件を `neplg2:test[stdio, normalize_newlines]` + `exit_code: 0` + `stdout:` へ移行した。
- `ret:` は tutorial の std/test assertion suite から削除し、言語戻り値検査と process exit-code 検査の混在を解消した。
- `nodesrc/test_tutorial_getting_started_current_style.js` に parser-based policy を追加し、getting_started の `std/test` doctest が `ret:` へ戻ること、stdout report を固定しないこと、report を出さずに exit code だけ返すことを拒否するようにした。

## 検証結果

- `node nodesrc/test_tutorial_getting_started_current_style.js`: passed
- `node nodesrc/tests.js -i tutorials/getting_started/02_test_harness.n.md -i tutorials/getting_started/03_values_and_types.n.md -i tutorials/getting_started/04_prefix_calls.n.md -i tutorials/getting_started/05_functions_and_blocks.n.md -i tutorials/getting_started/06_if_and_match.n.md -i tutorials/getting_started/07_option.n.md -i tutorials/getting_started/08_result.n.md -i tutorials/getting_started/09_validation_project.n.md -i tutorials/getting_started/10_string_and_text.n.md -i tutorials/getting_started/11_bytebuf_and_text_io.n.md -i tutorials/getting_started/12_char_and_ascii.n.md -i tutorials/getting_started/14_collection_reads.n.md -i tutorials/getting_started/16_drop_and_cleanup.n.md -i tutorials/getting_started/17_imports_and_modules.n.md -i tutorials/getting_started/18_generics.n.md -i tutorials/getting_started/19_traits_and_bounds.n.md -i tutorials/getting_started/20_namespace_and_methods.n.md -i tutorials/getting_started/21_project_fizzbuzz.n.md -i tutorials/getting_started/22_project_parser_small.n.md -i tutorials/getting_started/23_project_config_validator.n.md -i tutorials/getting_started/24_project_byte_output.n.md --no-tree -o tmp/agent1-getting-started-report-metadata.json -j 2 --dist web/dist --assert-io`: total=21, passed=21
