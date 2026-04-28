---
id: ISS-20260428T193442835Z-WASM-CODEGEN-BLOCK-WRAPPED-ELSE-IF-CH-5C1BA9D4
title: "wasm codegen stack overflows on block-wrapped else-if chains"
area: core
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-04-29
updated: 2026-04-29
target: "nepl-core/src/codegen_wasm.rs; stdlib/neplg2/core/syntax/lexer.nepl"
---

# ISS-20260428T193442835Z-WASM-CODEGEN-BLOCK-WRAPPED-ELSE-IF-CH-5C1BA9D4: wasm codegen stack overflows on block-wrapped else-if chains

## 概要

`lex_keyword_kind` を `match` based classifier に直す途中で、通常 Node stack の wasm compiler が `RangeError: Maximum call stack size exceeded` になりました。`node --stack_size=32768` では同じ入力が pass するため、入力の意味論ではなく wasm-host 上の codegen traversal stack 消費が原因です。

## 対象

- `nepl-core/src/codegen_wasm.rs`
- `stdlib/neplg2/core/syntax/lexer.nepl`

## 根拠

- `node nodesrc\run_test.js` に `tests/stdlib/neplg2_lexer.n.md::doctest#1` を渡すと、通常 Node stack で `codegen_wasm::gen_expr` / `gen_block` / `gen_if_else_chain` の往復により `RangeError: Maximum call stack size exceeded`。
- `node --stack_size=32768 ... nodesrc\run_test.js` では同じ doctest が pass。
- 既存の `gen_if_else_chain` は直接の `else if` chain は loop 化していたが、`match` lowering などが作る「else branch が単一式 block、その中身が If」の形を direct chain として扱えなかった。

## 問題

wasm codegen が block-wrapped else-if chain を通常の `Block` として再帰的に下げるため、有限分岐を `match` で書き直しただけで self-host lexer doctest が artifact 生成段階で host stack overflow になります。これは stdlib 側の自然な `match` 化を妨げる compiler 側の根本問題です。

## 影響

self-host stdlib の classifier を match-first 方針へ寄せる作業が、通常 Node runner では検証不能になります。同型の block-wrapped conditional chain が増えると、入力が正当でも診断ではなく compiler runtime stack exhaustion で失敗します。

## 修正方針

`gen_if_else_chain` に、単一非 drop 行の block が `If` だけを包んでいる場合に限ってその wrapper を剥がす処理を追加しました。block scope や drop semantics を変えない安全な形だけを対象にし、else-if lowering の loop に合流させます。

## 検証

- `rustfmt --check nepl-core\src\codegen_wasm.rs`: pass
- `trunk build`: pass
- `node nodesrc\tests.js -i stdlib\neplg2\core\syntax\lexer.nepl -i tests\stdlib\neplg2_lexer.n.md --no-tree -o tmp\selfhost-lexer-keyword-match.json -j 1`: total=13 passed=13
