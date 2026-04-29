---
id: ISS-20260429T101530928Z-N-MD-SHARED-TEST-OPERATION-FOR-RUST--52938450
title: ".n.md shared test operation for Rust and selfhost is undefined"
area: TEST
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-04-29
updated: 2026-04-29
target: "doc/neplg2, nodesrc/tests.js, nodesrc/run_doctest.js, stdlib/neplg2"
---

# ISS-20260429T101530928Z-N-MD-SHARED-TEST-OPERATION-FOR-RUST--52938450: .n.md shared test operation for Rust and selfhost is undefined

## 概要

`.n.md` doctest は現行 Rust compiler の回帰と self-host compiler の parity fixture の両方で使える形式だが、Rust 実装・selfhost 実装・nodesrc runner の役割分担、期待値の正、skip/tag 運用、移行順序が未定義である。

## 対象

- `doc/neplg2, nodesrc/tests.js, nodesrc/run_doctest.js, stdlib/neplg2`

## 根拠

- `nodesrc/tests.js` / `nodesrc/run_doctest.js` は `.n.md` と `.nepl` doc comment から `neplg2:test` を抽出し、Rust-built web compiler bundle で compile/run する。
- `nepl-core/tests/*.rs` は Rust integration test として個別 harness を持ち、同じ言語仕様の fixture が `.n.md` と重複している。
- `stdlib/neplg2/` の selfhost 実装は、lexer/parser/module などの stage ごとに Rust 実装との parity fixture を必要とする。
- `doc/neplg2/self_host_plan.md` は S1/S2/S3/S4/S5/S7 で Rust 実装との parity を成功条件にしているが、`.n.md` を共通 source of truth にする運用はまだ文書化されていない。
- 調査中に `ISS-20260429T101413560Z-NODESRC-DOCTEST-PARSER-RUNTIME-IGNOR-6E5E5A79` も見つかっており、共通化の前提として metadata parser の信頼性を固定する必要がある。

## 問題

現状のままでは、Rust integration test、`.n.md` doctest、selfhost parity test が別々に増え、仕様変更時に期待値がずれる。特に診断 code、stdout/stderr、return value、stage JSON snapshot をどの runner が正として読むかが曖昧で、selfhost の実装開始後に「Rust では通るが selfhost parity では別 fixture」という重複負債を生む。

## 影響

- selfhost の lexer/parser/checker/backend が Rust と同じ fixture を使えず、parity の信頼性が下がる。
- diagnostic redesign で stable string code を外部 contract にしたにもかかわらず、`.n.md` の期待値検査が各 runner で揃わない。
- Rust 側修正が `.n.md` へ反映されず、selfhost 側だけ古い期待値へ合わせる危険がある。
- selfhost bootstrap 比較 S7 の準備として必要な stage-level trace / artifact 比較の形式が決まらない。

## 修正方針

`doc/neplg2/` に `.n.md` 共通テスト運用計画を作成し、`.n.md` を「ケース定義と外部期待値の正」にする。Rust runner、selfhost runner、stage parity runner は同じ manifest を読み、実行 backend だけを差し替える。初期段階では既存 nodesrc runner を manifest parser と Rust execution backend として使い、selfhost は lexer/parser/module/checker/backend の stage が実装された順に同じ case を consume する。

## 検証

- 計画書を `doc/neplg2/` に追加し、現状調査、設計方針、移行フェーズ、blocker、検証コマンドを明記する。
- `node nodesrc/issues.js check` と doc-focused `git diff --check` を通す。
- 計画書の要点を Discord に本文で報告する。

## 2026-04-29 解決メモ

`doc/neplg2/shared_nmd_test_plan.md` を追加し、`.n.md` を Rust / selfhost 共通の case manifest として扱う方針を定義した。

計画では、既存 `nodesrc/tests.js` / `run_doctest.js` / `run_test.js` を Rust compile/run backend として整理し、selfhost は lexer/parser/module/check/resource/codegen の stage backend を段階追加する。期待値は diagnostic stable string code、span、return value、stdout/stderr、stable stage JSON に限定し、Rust 内部 debug 表示や未正規化順序には依存しない。

調査中に見つかった `diag_code` metadata parser drift は `ISS-20260429T101413560Z-NODESRC-DOCTEST-PARSER-RUNTIME-IGNOR-6E5E5A79` として分離した。この issue は計画書内で共通化前の blocker として扱う。

追加調査で、assertion 系 `.n.md` が `ret:` だけに依存して stdout report を固定しない問題も確認した。これは `ISS-20260429T102425370Z-N-MD-TESTS-RELY-ON-RETURN-VALUES-INS-9B49EDAD` として分離し、`doc/neplg2/nmd_assert_output_plan.md` に stdout report / exit code / `std/test` assert 再設計の計画をまとめた。

### 2026-04-29 検証

- `node nodesrc/issues.js check`: passed
- `git diff --check`: passed
- `trunk build`: passed
- `node nodesrc/cli.js -i doc/neplg2/shared_nmd_test_plan.md -o html=tmp/shared-nmd-test-plan-html`: generated 1 html file
- `node nodesrc/cli.js -i doc/neplg2/nmd_assert_output_plan.md -o html=tmp/nmd-assert-output-plan-html`: generated 1 html file
