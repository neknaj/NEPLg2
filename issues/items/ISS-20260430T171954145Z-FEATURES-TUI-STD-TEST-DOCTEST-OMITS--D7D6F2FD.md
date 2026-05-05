---
id: ISS-20260430T171954145Z-FEATURES-TUI-STD-TEST-DOCTEST-OMITS--D7D6F2FD
title: "features_tui std/test doctest omits stdout assertion report"
area: TEST
status: open
resolved: false
priority: P1
type: bug
created: 2026-04-30
updated: 2026-04-30
target: tests/stdlib/features_tui.n.md
---

# ISS-20260430T171954145Z-FEATURES-TUI-STD-TEST-DOCTEST-OMITS--D7D6F2FD: features_tui std/test doctest omits stdout assertion report

## 概要

features_tui_box_helpers_clamp_narrow_widths imports std/test and aggregates many checks, but it returns checks_exit_code without printing the deterministic assertion report and still uses ret: 0 as process success metadata.

## 対象

- `tests/stdlib/features_tui.n.md`

## 根拠

- `features_tui_box_helpers_clamp_narrow_widths` は `std/test` で 15 件の assertion を集約しているが、stdout の assertion report を fixture として固定していなかった。
- 修正中に WASIX doctest runner が `ret` / `exit_code` を安定して公開していないことも分かったため、process metadata の修正は `ISS-20260430T172357987Z-WASIX-DOCTEST-RUNNER-DOES-NOT-EXPOSE-800443AF` に分離する。
- 同 doctest のコンパイル確認中に Resource IR の Result payload owner summary false positive も露出したため、core 側は `ISS-20260505T010322571Z-RESOURCE-IR-VARIANT-OWNER-SUMMARY-KE-EB3C4EAC` で先に修正する。

## 問題

features_tui_box_helpers_clamp_narrow_widths imports std/test and aggregates many checks, but it returns checks_exit_code without printing the deterministic assertion report and still uses ret: 0 as process success metadata.

## 影響

A failure only appears as an exit-code mismatch, so Rust and self-host doctest runners cannot compare assertion detail or report formatting for this stdout-capable std test.

## 修正方針

Change the doctest to stdout report, call checks_print_report before checks_exit_code, and pin the expected report text. Keep WASIX exit-code metadata out of this fixture until the runner exposes it consistently.

## 検証

Run the focused features_tui doctest when a WASIX runner is available, and run source policy regressions for doctest report metadata.

## 2026-05-05 状況

- stdout report fixture の形へ変更中。
- `node nodesrc/tests.js -i tests/stdlib/features_tui.n.md --no-tree -o tmp/features-tui-after-owner-summary.json -j 1 --dist web/dist` は total=4, passed=2, failed=2。
- stdout report 対象の doctest#4 はまだ StringBuilder owner leak で compile fail するため、fixture の最終確認は `ISS-20260429T142213822Z-BYTEBUILDER-AND-STRINGBUILDER-RESULT-4EB1D1EB` の解消後に行う。
