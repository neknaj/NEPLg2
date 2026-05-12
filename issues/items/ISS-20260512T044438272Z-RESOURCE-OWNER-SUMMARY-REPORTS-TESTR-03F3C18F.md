---
id: ISS-20260512T044438272Z-RESOURCE-OWNER-SUMMARY-REPORTS-TESTR-03F3C18F
title: "Resource owner summary reports TestReport print machine result as maybe leak"
area: static-check
status: verified
resolved: true
priority: P1
type: bug
created: 2026-05-12
updated: 2026-05-12
target: "stdlib/std/test/report.nepl, nepl-core/src/resource, stdlib/tests/btreemap.n.md"
---

# ISS-20260512T044438272Z-RESOURCE-OWNER-SUMMARY-REPORTS-TESTR-03F3C18F: Resource owner summary reports TestReport print machine result as maybe leak

## 概要

On current main, stdlib/tests/btreemap.n.md fails all 5 doctests at compile phase with resource.owner.maybe_leak in checks_print_machine__TestReport__TestReport__imp. The failure reproduces without the BTreeMap API split changes, so it is a ResourceIR/TestReport owner-flow issue rather than a collection regression.

## 対象

- `stdlib/std/test/report.nepl, nepl-core/src/resource, stdlib/tests/btreemap.n.md`

## 根拠

- 2026-05-12: `stdlib/tests/btreemap.n.md` の 5 doctest は現 main で再現せず、`checks_print_machine__TestReport__TestReport__imp` の `resource.owner.maybe_leak` は発生しないことを確認した。
- 直前の owner summary 修正で、値コピーと owner transfer に巻き込まれていた summary/fact の欠落が解消され、`TestReport` の表示後 return owner が一時 projection leak と誤認されなくなった。
- 再発防止として `checks_print_machine` を直接呼ぶ ResourceIR owner 回帰を追加し、`checks_print_machine__` と `main__` に owner 診断が出ないことを検査する。

## 問題

修正前は `stdlib/tests/btreemap.n.md` の 5 doctest が compile phase で失敗し、`checks_print_machine__TestReport__TestReport__imp` に `resource.owner.maybe_leak` が出ていた。BTreeMap API split 変更を戻しても再現していたため、collection 実装ではなく ResourceIR/TestReport owner-flow の問題として扱う。

## 影響

Stdlib collection suites that use std/test report printing cannot be used as broad regression checks under the strict owner gate. Weakening owner checks would hide real leaks, so the TestReport return/print owner boundary must be represented accurately.

## 修正方針

`TestReport` API の所有権契約は変更しない。`checks_print_machine` は表示用の参照 projection を作った後、入力された `TestReport` owner をそのまま返す single-owner helper であり、compiler 側の ResourceIR owner summary がこの return owner を正しく扱うべきである。

現 main では直前の owner summary 修正により、値コピーと owner transfer の混同で失われていた summary 情報が保持されるようになり、この issue の失敗は解消している。stdlib 側に release や clone を追加して診断を黙らせる修正は行わず、compiler の owner model を正とした。

## 検証

- `cargo test -p nepl-core --test resource_ir resource_ir_owner_check_accepts_checks_print_machine_report_return -- --nocapture`
- `node nodesrc/tests.js -i stdlib/tests/btreemap.n.md --no-tree -o tmp/agent1-testreport-btreemap-repro.json -j 1 --dist web/dist`

## 解決内容

- `nepl-core/tests/resource_ir.rs` に `checks_print_machine` 経路の owner 回帰を追加した。
- この回帰は、表示後に返る `TestReport` owner が leak / maybe leak と診断されないことを直接確認する。
