---
id: ISS-20260426T055122421Z-STREAMIO-DOCTEST-KEEPS-OBSOLETE-PIPE-F37DE397
title: "streamio doctest keeps obsolete pipe cast workaround"
area: stdlib
status: verified
resolved: true
priority: P2
type: maintenance
created: 2026-04-26
updated: 2026-04-26
target: tests/stdlib/streamio.n.md
---

# ISS-20260426T055122421Z-STREAMIO-DOCTEST-KEEPS-OBSOLETE-PIPE-F37DE397: streamio doctest keeps obsolete pipe cast workaround

## 概要

`tests/stdlib/streamio.n.md` の `stream_writer_space_and_i64` は、旧 `ISS-20260426T023624387Z-PIPE-004372E8` の pipe 右辺 bug を避けるため `let two <i64> cast 2` という中間変数を置いたままになっている。core 側の pipe 修正後は `|> writeln <i64> cast 2` が自然な書き方として通るため、stdlib 側に古い workaround が残っている。

## 対象

- `tests/stdlib/streamio.n.md`

## 根拠

- `issues/items/ISS-20260426T023624387Z-PIPE-004372E8.md` は verified で、pipe 右辺の `|> writeln <i64> cast 2` を回帰テストとして固定済み。
- `note.n.md` の `ISS-20260426T023624387Z-PIPE-004372E8` 記録では、`let two <i64> cast 2; |> writeln two` が当時の workaround だったことを明記している。
- `tests/stdlib/streamio.n.md` にはその workaround が残り、現行 compiler が受け付ける自然な inline cast を stdlib doctest が使っていない。

## 問題

`tests/stdlib/streamio.n.md` の `stream_writer_space_and_i64` は、旧 `ISS-20260426T023624387Z-PIPE-004372E8` の pipe 右辺 bug を避けるため `let two <i64> cast 2` という中間変数を置いたままになっている。core 側の pipe 修正後は `|> writeln <i64> cast 2` が自然な書き方として通るため、stdlib 側に古い workaround が残っている。

## 影響

stdlib の doctest が現在の言語として推奨すべき書き方ではなく、過去の compiler bug 回避を見本として残してしまう。self-host や tutorial で同じ不要な中間変数が増える原因になる。

## 修正方針

`stream_writer_space_and_i64` を `|> writeln <i64> cast 2` に戻し、pipe 右辺で inline cast を使えることを stdlib 側の fixture でも固定する。

## 検証

`node nodesrc/tests.js -i tests/stdlib/streamio.n.md --no-tree -o tmp/stdlib-streamio-pipe-cast-workaround.json -j 1` を通す。

## 対応結果

`stream_writer_space_and_i64` の不要な `let two <i64> cast 2` を削除し、pipe 右辺に `|> writeln <i64> cast 2` を直接置く形へ戻した。

これにより、解決済みの pipe 右辺 inline cast を stdlib doctest 側でも自然な書き方として固定した。

## 確認結果

- `node nodesrc/tests.js -i tests/stdlib/streamio.n.md --no-tree -o tmp/stdlib-streamio-pipe-cast-workaround.json -j 1`: `total=13`, `passed=13`, `failed=0`
