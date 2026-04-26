---
id: ISS-20260426T134706492Z-MOVE-CHECKER-ALLOWS-LOCAL-BORROW-REF-337A78B2
title: "move checker allows local borrow references to escape their scope"
area: core
status: verified
resolved: true
priority: P1
type: bug
created: 2026-04-26
updated: 2026-04-26
target: "nepl-core/src/passes/move_check.rs, nepl-core/tests/move_check.rs, tests/compiler/move_check.n.md"
---

# ISS-20260426T134706492Z-MOVE-CHECKER-ALLOWS-LOCAL-BORROW-REF-337A78B2: move checker allows local borrow references to escape their scope

## 概要

move checker は live borrow を一部の直接参照束縛だけで扱っており、ローカル値への参照が関数返り値、block 返り値、外側 scope の変数、または参照を含む struct へ入った場合に lifetime escape を十分に拒否できていなかった。

## 対象

- `nepl-core/src/passes/move_check.rs, nepl-core/tests/move_check.rs, tests/compiler/move_check.n.md`

## 根拠

- `fn leak <()->&T>` の末尾で `&local` を返す形が compile 可能だった。
- `let r <&T> block: ... &local` や、外側の `r` へ内側 block の `&local` を `set` する形も destination scope と borrow source scope の比較がなかった。
- `RefBox { inner: &T }` のような参照を含む集約値では、直接参照だけを見ても不十分で、`let b <RefBox> RefBox &local; b` のように一度変数へ格納してから escape する経路を追跡できていなかった。
- `fn id_ref <(&T)->&T>` / `fn box_ref <(&T)->RefBox>` のように参照引数から参照を含む返り値が作られる場合も、caller 側で返り値 lifetime を引数の borrow origin に結びつける必要があった。

## 問題

borrow source の宣言 scope depth を記録していなかったため、参照値の利用先がどの scope まで生存するかを比較できなかった。また、式評価の結果が保持する borrow origin を返していなかったため、参照を含む集約値や参照返り値関数を経由した lifetime escape が形だけの直接 `&local` 検査では漏れる構造だった。

## 影響

scope 終了後の local への参照を含むプログラムが compile され、borrow/lifetime safety を破る。drop insertion や codegen は無効な参照を正当な値として扱うことになり、self-host compiler 実装時の所有権・型安全の前提が崩れる。

## 修正方針

各 binding の宣言 scope depth と、その binding が保持する borrow origin を記録する。式走査は `ExprBorrow` を返し、`let` / `set` で保持 borrow として確定する。block / if / match / constructor / 関数返り値 / 参照引数の返り値伝播では expected escape depth を渡し、borrow source の depth が destination より深い場合は `D3099 TypeBorrowEscapesScope` を出す。

## 解決内容

- `nepl-core/src/passes/move_check.rs` に scope depth stack と複数 borrow origin stack を追加し、参照を含む値の move/copy/assignment で origin を保持するようにした。
- `AddrOf` は一時 borrow として検査し、`let` / `set` でのみ保持 borrow に確定するよう整理した。
- block 末尾式、if/match branch、struct/tuple/enum constructor、参照を含む返り値を持つ call/call_indirect に escape depth を伝播した。
- 参照引数から参照を含む返り値が作られる関数呼び出しでは、caller 側で返り値の borrow origin を参照引数へ結びつけるようにした。
- `DiagnosticId::TypeBorrowEscapesScope = D3099` を追加した。

## 検証結果

- `cargo fmt --all --check`: pass
- `cargo check -p nepl-core --test move_check`: pass
- `cargo test -p nepl-core --test move_check -- --nocapture`: 22 passed
- `trunk build`: pass
- `node nodesrc/tests.js -i tests/compiler/move_check.n.md --no-tree -o tmp/move-check-borrow-escape.json -j 1`: `total=23`, `passed=23`, `failed=0`
