---
id: ISS-20260428T003718356Z-NM-PARSER-AND-HTML-GEN-RAW-MEMORY-DE-99175378
title: "nm parser and html_gen raw memory detours fail under strict move checking"
area: stdlib
status: open
resolved: false
priority: P1
type: bug
created: 2026-04-28
updated: 2026-04-28
target: "stdlib/nm/parser.nepl, stdlib/nm/html_gen.nepl, tests/stdlib/nm.n.md"
---

# ISS-20260428T003718356Z-NM-PARSER-AND-HTML-GEN-RAW-MEMORY-DE-99175378: nm parser and html_gen raw memory detours fail under strict move checking

## 概要

`trunk build` 後の最新 move checker で `stdlib/nm` の scoped doctest が D3100 になる。`html_gen.nepl::doctest#2` は non-Copy 値を保持する raw memory place `$memptr:grown_data+?` の overwrite、`parser.nepl::doctest#2/#3` は raw memory place `sec_mem+20` の moved 後利用として失敗する。

`stdlib/nm/parser.nepl` と `stdlib/nm/html_gen.nepl` は、aggregate 値を分解するために `alloc_raw` / `store` / `load` の raw memory detour をまだ使っている。move checker が raw provenance を厳密化したことで、この detour が所有権の曖昧さとして表面化した。

## 対象

- `stdlib/nm/parser.nepl, stdlib/nm/html_gen.nepl, tests/stdlib/nm.n.md`

## 根拠

- `node nodesrc/tests.js -i stdlib/nm --no-tree -o tmp/stdlib-nm-after-trunk-20260428.json -j 2`: `total=5`, `passed=2`, `failed=3`
- `stdlib/nm/html_gen.nepl::doctest#2`: D3100 `overwriting raw memory place containing non-Copy value: $memptr:grown_data+?`
- `stdlib/nm/parser.nepl::doctest#2/#3`: D3100 `use of moved raw memory place: sec_mem+20`

## 問題

nm parser / html generator が raw memory detour によって non-Copy aggregate の field を何度も取り出しているため、compiler がどの field owner を consume 済みかを安全に追跡できない。D3100 を弱めると hidden shallow move / live payload overwrite を再び許すため、stdlib 側の detour を消すか、compiler 側に安全な aggregate decomposition / borrowed field projection を用意する必要がある。

## 影響

`stdlib/nm` の parser / html generation が scoped stdlib test で通らない。stdlib doc は nm 拡張 markdown で書かれるため、selfhost の標準ライブラリ文書生成にも影響する。全体 stdlib 検証も、この D3100 を解消するまで clean と判断しにくい。

## 修正方針

`nm` parser / html_gen の aggregate raw-memory detour を、安全な owned decomposition または borrowed field projection へ置き換える。必要な compiler operation が不足している場合は、既存の owned aggregate decomposition / borrowed field projection issue に接続し、D3100 の検査を緩めない。修正後は section nesting と HTML rendering の focused regression を追加する。

## 検証

- `node nodesrc/tests.js -i stdlib/nm --no-tree -o tmp/stdlib-nm-after-fix.json -j 2`
- `node nodesrc/tests.js -i tests/stdlib/nm.n.md --no-tree -o tmp/nm-focused-after-fix.json -j 1`
