---
id: ISS-20260426T023638576Z-FEATURES-TUI-DOCTEST-HELPER-988788D1
title: "features_tui doctest が未定義 helper 参照で失敗する"
area: stdlib
status: fixed
resolved: true
priority: P1
type: test
created: 2026-04-26
updated: 2026-04-26
target: "tests/stdlib/features_tui.n.md, stdlib/features/tui.nepl, nepl-core/src/typecheck.rs"
---

# ISS-20260426T023638576Z-FEATURES-TUI-DOCTEST-HELPER-988788D1: features_tui doctest が未定義 helper 参照で失敗する

## 概要

tests/stdlib/features_tui.n.md が tui::line_pad_to_cols、tui::repeat_text、tui::get_terminal_size の D3001 undefined identifier で失敗する。

## 対象

- `tests/stdlib/features_tui.n.md, stdlib/features/tui.nepl, nepl-core/src/typecheck.rs`

## 根拠

- `node nodesrc/tests.js -i tests/stdlib --no-tree -o tmp/rv-stdlib-018-final-tests-stdlib-crlf.json -j 4` で `tests/stdlib/features_tui.n.md::doctest#1` と `doctest#2` が compile failure になった。
- `doctest#1` は `tui::line_pad_to_cols` と `tui::repeat_text` が `D3001 undefined identifier` になり、後続で `D3016` が連鎖する。
- `doctest#2` は `tui::get_terminal_size` が `D3001 undefined identifier` になり、`cols` / `rows` の取得と条件式で `D3016` / `D3039` が連鎖する。

## 問題

tests/stdlib/features_tui.n.md が tui::line_pad_to_cols、tui::repeat_text、tui::get_terminal_size の D3001 undefined identifier で失敗する。

## 影響

tests/stdlib 全体の green 化を阻害し、TUI feature の公開 API と fixture のどちらが正しいか検証できない。

## 修正方針

stdlib/features/tui.nepl の公開 API と doctest の意図を照合し、仕様上必要な helper は実装し、既存 API へ統合済みなら doctest を現行名へ更新する。

## 対応結果

`features/tui` の facade は `platforms/wasix/tui` を `@merge` しているが、typecheck の qualified import 解決は alias 先ファイルの直接定義だけを見ていた。
そのため、`#import "features/tui" as tui` の `tui::line_pad_to_cols` / `tui::repeat_text` / `tui::get_terminal_size` が、facade に merge された定義まで到達できなかった。

`nepl-core/src/typecheck.rs` の qualified import target 構築を、alias 先ファイルから direct `@merge` import 先へ展開するように修正した。
通常の `as *` import は qualified alias へ漏らさない回帰テストを追加し、facade だけを名前空間として扱えるようにした。
`stdlib/features/tui.nepl` は facade の意図に合わせて公開 merge import として明記した。

修正後、`features_tui` の undefined identifier は解消し、doctest は compile phase を通過するようになった。
ただしローカル環境では次に `wasmer run --volume` 非対応による run phase failure が出たため、別 Issue `ISS-20260426T030615554Z-WASIX-DOCTEST-RUNNER-USES-WASMER-VOL-8527FD91` として分離した。

## 検証

- `node nodesrc/tests.js -i tests/stdlib/features_tui.n.md --no-tree -o tmp/features-tui-issue.json -j 1`
- `node nodesrc/tests.js -i tests/stdlib --no-tree -o tmp/features-tui-tests-stdlib.json -j 4`
- `cargo test -p nepl-core --test import_clause` (`8 passed`)
- `trunk build` 成功
- `node nodesrc/tests.js -i tests/stdlib/features_tui.n.md --no-tree -o tmp/features-tui-issue.json -j 1` は compile phase の D3001 が解消し、run phase で `wasmer --volume` 非対応により失敗。runner 互換性は `ISS-20260426T030615554Z-WASIX-DOCTEST-RUNNER-USES-WASMER-VOL-8527FD91` で追跡。
