---
id: ISS-20260426T023700894Z-TRAITS-TEXT-DOCTEST-RUNTIME-RETURN-M-D1631318
title: "traits_text doctest が runtime return mismatch になる"
area: core
status: verified
resolved: true
priority: P1
type: bug
created: 2026-04-26
updated: 2026-04-26
target: "nepl-core/src/codegen_wasm.rs, tests/compiler/reference_codegen.n.md, tests/stdlib/traits_text.n.md"
---

# ISS-20260426T023700894Z-TRAITS-TEXT-DOCTEST-RUNTIME-RETURN-M-D1631318: traits_text doctest が runtime return mismatch になる

## 概要

tests/stdlib/traits_text.n.md::doctest#1 が expected 14 に対して actual 131074 を返す。

## 対象

- `nepl-core/src/codegen_wasm.rs`
- `tests/compiler/reference_codegen.n.md`
- `tests/stdlib/traits_text.n.md`

## 根拠

- `node nodesrc/tests.js -i tests/stdlib --no-tree -o tmp/rv-stdlib-018-final-tests-stdlib-crlf.json -j 4` で `tests/stdlib/traits_text.n.md::doctest#1` が runtime failure になった。
- failure は `return value mismatch` で、期待値は `14`、実際の戻り値は `131074`。
- 同じ広域検証で streamio は 13/13 green になっているため、streamio 修正とは別原因として分離する。

## 問題

tests/stdlib/traits_text.n.md::doctest#1 が expected 14 に対して actual 131074 を返す。

## 影響

文字列関連 trait の doctest が信頼できず、Clone / text conversion / output helper の組み合わせで値表現が崩れている可能性を検出できない。

## 修正方針

期待値 14 の意味を分解し、どの式が 131074 を返しているか最小化する。fixture ずれなら期待値を根拠付きで更新し、runtime 表現の混入なら trait 実装または string helper を修正する。

## 調査結果

`Clone::clone &x` や文字列 trait 以前に、最小化した `*(&i32)` が `6` ではなく `131072` を返していた。
WASM backend の `HirExprKind::AddrOf` は inner expression をそのまま評価して reference 型の値として扱っており、scalar local の値 `6` を「アドレス 6」として `Deref` に渡していた。
aggregate は既に heap storage pointer を値として持つためこの経路で成立していたが、i32/u8/f32/i64/f64 などの scalar は参照として渡す前に addressable storage へ退避する必要があった。

## 対応結果

`nepl-core/src/codegen_wasm.rs` の `AddrOf` lowering を修正し、aggregate は従来どおり storage pointer を返し、scalar は値を一時localに保持してから型サイズ分のメモリを確保し、適切な store 命令で書き込んだうえで pointer を返すようにした。
unit など runtime 表現を持たない値への参照は inner の副作用だけ評価して `0` pointer を返す。

`tests/compiler/reference_codegen.n.md` を追加し、`*(&i32)` と `Clone::clone &i32` が正しい scalar 値を返すことを固定した。
元の `tests/stdlib/traits_text.n.md` も `14` を返すようになり、stdlib全体の doctest は `202/202` green になった。

## 検証

- `node nodesrc/tests.js -i tests/stdlib/traits_text.n.md --no-tree -o tmp/traits-text-issue.json -j 1`
- `node nodesrc/tests.js -i tests/stdlib --no-tree -o tmp/traits-text-tests-stdlib.json -j 4`
- `cargo test -p nepl-core --test move_check`
- `trunk build`
- `node nodesrc/tests.js -i tests/compiler/reference_codegen.n.md --no-tree -o tmp/reference-codegen-tests.json -j 1` (`total=2`, `passed=2`, `failed=0`)
- `node nodesrc/tests.js -i tests/stdlib/traits_text.n.md --no-tree -o tmp/traits-text-issue.json -j 1` (`total=3`, `passed=3`, `failed=0`)
- `node nodesrc/tests.js -i tests/stdlib --no-tree -o tmp/traits-text-tests-stdlib.json -j 4` (`total=202`, `passed=202`, `failed=0`)
