---
id: ISS-20260518T233744382Z-PIPE-COLLECTIONS-DOCTESTS-RELY-ON-RE-4299CCE8
title: "pipe_collections doctests rely on return codes or unpinned reports"
area: TEST
status: fixed
resolved: true
priority: P1
type: test
created: 2026-05-18
updated: 2026-05-18
target: tests/stdlib/pipe_collections.n.md
---

# ISS-20260518T233744382Z-PIPE-COLLECTIONS-DOCTESTS-RELY-ON-RE-4299CCE8: pipe_collections doctests rely on return codes or unpinned reports

## 概要

tests/stdlib/pipe_collections.n.md は pipe + collection の重要な回帰テストだが、4件は ret: 1 だけで合否を返し、4件は checks_print_report を呼ぶのに stdout/exit_code metadata を固定していない。

## 対象

- `tests/stdlib/pipe_collections.n.md`

## 根拠

- `pipe_list_alias_chain`、`pipe_stack_alias_usage`、`pipe_ringbuffer_usage`、`pipe_queue_usage` は `ret: 1` だけで合否を返していた。
- `pipe_btreemap_usage`、`pipe_btreeset_usage`、`pipe_hashmap_usage`、`pipe_hashset_usage` は `checks_print_report` を呼んでいたが、manifest に `stdio` / `normalize_newlines` / `stdout:` / `exit_code:` がなかった。
- この fixture は pipe 記法、collection alias、owner-preserving update、borrowed observer をまとめて監視するため、どの観測点が壊れたかを stdout report で固定する必要がある。

## 問題

tests/stdlib/pipe_collections.n.md は pipe + collection の重要な回帰テストだが、4件は ret: 1 だけで合否を返し、4件は checks_print_report を呼ぶのに stdout/exit_code metadata を固定していない。

## 影響

collection alias / pipe / owner-preserving update の退行が、どの assertion が壊れたか分からない exit status 依存になり、selfhost runner と Rust runner の stdout report 互換性も検査できない。

## 修正方針

8 doctest すべてを std/test Checks に揃え、stdio + normalize_newlines + exit_code: 0 + deterministic stdout fixture を固定する。専用 source policy regression で ret 代用と stdout 欠落を拒否する。

## 検証

node nodesrc/test_stdlib_pipe_collections_report_contract.js; node nodesrc/tests.js -i tests/stdlib/pipe_collections.n.md --no-tree -o tmp/agent1-pipe-collections-report.json -j 1 --dist web/dist --assert-io

## 2026-05-18 修正

8 doctest すべてを `neplg2:test[stdio, normalize_newlines]`、`exit_code: 0`、deterministic `stdout:` fixture に移行した。

- List / Stack / RingBuffer / Queue の ret-only case は `std/test::Checks` に変換し、len と get/pop/peek の 2 assertions を stdout に固定した。
- BTreeMap / BTreeSet / HashMap / HashSet の既存 report case は検査ロジックを維持し、manifest 側で 3 / 3 / 3 / 2 assertions の report を固定した。

`nodesrc/test_stdlib_pipe_collections_report_contract.js` を追加し、8 件が `ret:` 代用や stdout 欠落へ戻らないことを source policy regression に登録した。
