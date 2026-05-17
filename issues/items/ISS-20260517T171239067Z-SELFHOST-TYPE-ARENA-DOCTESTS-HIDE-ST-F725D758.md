---
id: ISS-20260517T171239067Z-SELFHOST-TYPE-ARENA-DOCTESTS-HIDE-ST-F725D758
title: "selfhost type arena doctests use owner-backed allocation fields and hide reports"
area: selfhost
status: fixed
resolved: true
priority: P1
type: test
created: 2026-05-17
updated: 2026-05-18
target: "stdlib/neplg2/core/ty/ty.nepl, tests/stdlib/neplg2_type_arena.n.md"
---

# ISS-20260517T171239067Z-SELFHOST-TYPE-ARENA-DOCTESTS-HIDE-ST-F725D758: selfhost type arena doctests use owner-backed allocation fields and hide reports

## 概要

self-host type arena doctest は `SelfhostTypeArenaAlloc` の `arena` / `type_id` field を直接読んでいた。現行の owner-backed aggregate field restriction ではこれは正しく拒否されるため、doctest は compile 不能になっていた。さらに 5 件の doctest は `checks_print_report` を呼ぶ一方で、manifest が `ret: 0` のままで stdout fixture、`exit_code:`、`stdio` / `normalize_newlines` tag を持っていなかった。

## 対象

- `stdlib/neplg2/core/ty/ty.nepl, tests/stdlib/neplg2_type_arena.n.md`

## 根拠

- focused run で 5 件すべてが `type.owner_aggregate.field_access_restricted` を起点に compile failure になった。
- 原因箇所は `alloc1.arena` / `alloc1.type_id` など、owner-backed aggregate wrapper の direct field projection だった。
- accessor 修正後、5 件は stdout に deterministic `Checked [...]` report を出したが、変更前 manifest は `ret: 0` だけを検査していた。

## 問題

self-host type arena doctest は `SelfhostTypeArenaAlloc` の `arena` / `type_id` field を直接読んでいた。現行の owner-backed aggregate field restriction ではこれは正しく拒否されるため、doctest は compile 不能になっていた。さらに 5 件の doctest は `checks_print_report` を呼ぶ一方で、manifest が `ret: 0` のままで stdout fixture、`exit_code:`、`stdio` / `normalize_newlines` tag を持っていなかった。

## 影響

- `SelfhostTypeArenaAlloc` が public API として安全に分解できず、通常 source が owner-backed aggregate field restriction に依存して壊れる。
- type arena の primitive / function shape / invalid access 検査が compile 不能になり、selfhost type model の回帰を検出できない。
- stdout report が fixture ではないため、Rust runner と selfhost runner の assertion count / report format 互換性を固定できない。

## 修正方針

- `SelfhostTypeArenaAlloc` には direct field access ではなく、Copy な `type_id` を borrow から読む accessor と、wrapper を消費して `arena` owner を取り出す accessor を追加する。
- doctest はこの public accessor だけを使い、owner-backed aggregate field を直接読まない。
- 5 件の doctest を `neplg2:test[stdio, normalize_newlines]` + stdout report + `exit_code: 0` へ移行し、`ret:` を削除する。
- source policy で direct field access と quiet exit-code-only metadata の退行を拒否する。

## 検証

source policy と focused `neplg2_type_arena` doctest を `--assert-io` 付きで実行する。

## 修正内容

- `selfhost_type_arena_alloc_type_id` を追加し、`SelfhostTypeArenaAlloc` の borrow から Copy な `SelfhostTypeId` だけを読む public API を用意した。
- `selfhost_type_arena_alloc_into_arena` を追加し、`SelfhostTypeArenaAlloc` を消費して `SelfhostTypeArena` owner を取り出す public API を用意した。
- `selfhost_ty_stage0` と `tests/stdlib/neplg2_type_arena.n.md` を direct field access から accessor 利用へ移行した。
- 5 件の doctest を `neplg2:test[stdio, normalize_newlines]` + `exit_code: 0` + deterministic `stdout:` へ移行した。
- `nodesrc/test_selfhost_type_arena_report_contract.js` を追加し、metadata、stdout report 件数、`allocN.arena` / `allocN.type_id` 退行を拒否するようにした。

## 検証結果

- `node nodesrc/test_selfhost_type_arena_report_contract.js`: passed
- `node nodesrc/tests.js -i tests/stdlib/neplg2_type_arena.n.md --no-tree -o tmp/agent1-neplg2-type-arena-report-metadata.json -j 1 --dist web/dist --assert-io`: total=5, passed=5
