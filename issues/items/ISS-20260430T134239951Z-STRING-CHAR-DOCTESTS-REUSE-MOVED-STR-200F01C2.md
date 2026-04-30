---
id: ISS-20260430T134239951Z-STRING-CHAR-DOCTESTS-REUSE-MOVED-STR-200F01C2
title: "string char doctests reuse unresolved str owner effects and hide reports"
area: TEST
status: fixed
resolved: true
priority: P1
type: test
created: 2026-04-30
updated: 2026-04-30
target: tests/stdlib/string_char.n.md
source: issues/items/ISS-20260429T102425370Z-N-MD-TESTS-RELY-ON-RETURN-VALUES-INS-9B49EDAD.md
---

# ISS-20260430T134239951Z-STRING-CHAR-DOCTESTS-REUSE-MOVED-STR-200F01C2: string char doctests reuse unresolved str owner effects and hide reports

## 概要

`tests/stdlib/string_char.n.md` reuses the same `str` local across by-value char observer APIs. Under the current Resource IR owner model, fallible `str`-returning/string-backed APIs may reserve the argument owner effect until the result is refined, so the first two doctests fail with `resource.owner.reserved`. The same file also kept `ret: 0` metadata and returned `checks_exit_code checks` without printing assertion reports.

## 対象

- `tests/stdlib/string_char.n.md`

## 根拠

- `node nodesrc/tests.js -i tests/stdlib/string_char.n.md --no-tree -o tmp/tests-stdlib-string-char-before-agent1.json -j 1 --dist web/dist` で 3件中2件が `resource.owner.reserved` により compile fail した。
- 失敗箇所は `str_char_at_result s ...` / `str_next_char_result s ...` の後に同じ `s` を再利用する流れだった。
- `str` 自体の Copy view contract と Resource IR の動的 string owner model の不一致は、別 issue `ISS-20260430T135134835Z-STR-COPY-VIEW-CONTRACT-CONFLICTS-WIT-0998304C` として切り出した。
- timeout 調査では compile-only 計測が doctest#1/#2/#3 で約 42.99秒 / 36.51秒 / 56.76秒となり、JSON の duration とほぼ一致した。実行時アルゴリズムや生成 wasm の長時間実行ではなく、stdlib 込み compile が支配的だった。
- 3件構成のまま `-j 3` で並列実行すると、builder case が compiler contention により 60秒 case timeout に達した。builder case を string builder と byte builder に分割すると、`-j 4` で 4件すべて 60秒以内に収まった。

## 問題

`tests/stdlib/string_char.n.md` reuses one `str` local as though all char/string observers were pure Copy reads. This hides the current Resource IR contract that an unresolved fallible result can reserve an owner-carrying argument. The doctests also rely on exit code only, so assertion report formatting is not pinned for the self-host runner.

## 影響

The char/string regression file was failing, and its successful assertions were not visible in stdout fixtures. Leaving this as-is blocks doctest cleanup and makes string-char behavior a weak CI signal.

## 修正方針

Use fresh string literals for by-value observer calls in this fixture so each assertion has an independent argument under the current owner model. Add `checks_print_report`, migrate the doctests to `exit_code: 0`, and pin stdout assertion reports. Split the builder coverage so no single doctest sits on the 60-second case timeout boundary under parallel compiler load. Track the broader `str` view/owner model conflict separately instead of weakening Resource IR reserved-owner diagnostics here.

## 検証

- `node nodesrc/tests.js -i tests/stdlib/string_char.n.md --no-tree -o tmp/tests-stdlib-string-char-agent1-j4.json -j 4 --dist web/dist`: total=4, passed=4
- final per-case durations: 53.4s / 49.2s / 51.7s / 57.0s
- `rg -n '^ret: 0|checks_exit_code checks|let s <str>' tests/stdlib/string_char.n.md`: no matches
- `node nodesrc/issues.js check`
- `git diff --check`
