---
id: ISS-20260429T102809685Z-STDLIB-ASSERT-API-MIXES-ASSERTION-RE-0F17011A
title: "stdlib assert API mixes assertion, reporting, and exit-code responsibilities"
area: STDLIB-TEST
status: open
resolved: false
priority: P1
type: architecture
created: 2026-04-29
updated: 2026-04-29
target: "stdlib/std/test.nepl, stdlib/core/test.nepl, tests/stdlib/std_test_collect.n.md, doc/neplg2/nmd_assert_output_plan.md"
---

# ISS-20260429T102809685Z-STDLIB-ASSERT-API-MIXES-ASSERTION-RE-0F17011A: stdlib assert API mixes assertion, reporting, and exit-code responsibilities

## 概要

`stdlib/std/test.nepl` の assert API は、assertion の評価、失敗表示、集約、exit code 変換の責務が混在している。

`.n.md` を Rust と selfhost の共通 test manifest にするには、assertion は構造化された検査結果を作り、report helper が stdout を出し、exit helper が可否だけを返す設計に分離する必要がある。

## 対象

- `stdlib/std/test.nepl, stdlib/core/test.nepl, tests/stdlib/std_test_collect.n.md, doc/neplg2/nmd_assert_output_plan.md`

## 根拠

- `assert_*` は `check_*` を呼んだ後、失敗時に `test_fail` を経由して `FAIL:` を stdout に出しつつ `Result<(),str>::Err` を返す。
- `checks_push` は `Result<(),str>` を集約するだけの API だが、`checks_push assert_eq_i32 ...` のように使うと、集約前に stdout が出る。
- `check_*` は quiet、`assert_*` は printing、`checks_print_report` は summary/human report、`checks_exit_code` は 0/1 変換という層が同じ module にあるが、API 名から責務境界が読み取りにくい。
- `checks_exit_code` は stdout を出さないため、test author が `checks_print_report` を呼び忘れると、exit code だけの不透明な failure になる。
- `core/test.nepl` の assert は `unreachable` による最小 trap で、std stdout report と同じ API 名を持つが failure detail の性質が異なる。

## 問題

- `assert_*` が値を返す関数でありながら副作用として stdout を出すため、集約 test の出力順と report 形式が不安定になりやすい。
- failure detail が `str` に整形済みで保存されており、assertion kind、label、expected、actual、status を enum/struct として静的検査できない。
- stdout report と exit code の関係が API で強制されないため、`.n.md` test が詳細 report なしの `ret: 0/1` に戻りやすい。
- `core/test` と `std/test` の assert 名が似ているため、trap 型 assertion と report 型 assertion の違いが曖昧である。

## 影響

- `.n.md` の stdout expectation を安定 contract にできず、Rust/selfhost 共通 test の比較対象が exit code に偏る。
- assertion 失敗時に詳細が runner の生ログにだけ出るか、または全く出ない case が残る。
- selfhost 側で同じ test library を実装するとき、現在の曖昧な責務境界を再現してしまうリスクがある。

## 修正方針

- `std/test` を、構造化 assertion、集約 report、stdout rendering、exit code 変換の 4 層へ再設計する。
- `AssertionStatus`、`AssertionKind`、`AssertionFailure`、`TestReport` のような enum/struct を導入し、数値や自由文字列ではなく静的検査が効く形で状態を保持する。
- `assert_*` は stdout を出さず、`TestAssertion` 相当の値を返す pure API とする。即時 trap が必要な helper は `panic_assert_*` や `core_assert_*` のように別名へ分ける。
- stdout は `test_report_print_stdout` 相当の明示 API だけが担当する。exit code は `test_report_exit_code` 相当の API だけが担当する。
- 既存 `check_*` / `checks_*` / `assert_*` は後方互換を理由に残さず、migration issue で一括置換する。
- `core/test` は stdout を持たないため、trap 用最小 helper として別設計にし、std report API と同名にしない。

## 検証

- `tests/stdlib/std_test_collect.n.md` を新 API の canonical fixture に更新し、成功/失敗 report の stdout と exit code を固定する。
- `std/test` の failure report で assertion kind、label、expected、actual が欠落しないことを `.n.md` で確認する。
- `std/test` を import する doctest が report helper を通さず exit code だけ返す場合に、runner/lint で検出できることを確認する。
