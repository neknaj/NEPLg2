---
id: ISS-20260507T092629164Z-STACK-KEEPS-DUPLICATE-BY-VALUE-AND-R-B8BF2270
title: "Stack keeps duplicate by-value and *_ref observer APIs"
area: stdlib
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-05-07
updated: 2026-05-07
target: "stdlib/alloc/collections/stack.nepl, stdlib/tests/stack.n.md, tests/stdlib/stack_collections.n.md, examples/rpn.nepl, examples/bf.nepl"
---

# ISS-20260507T092629164Z-STACK-KEEPS-DUPLICATE-BY-VALUE-AND-R-B8BF2270: Stack keeps duplicate by-value and *_ref observer APIs

## 概要

Stack still exposes len/is_empty/peek as owner-consuming observers while len_ref/is_empty_ref/peek_ref/get_ref are used for borrowed reads. This preserves the old workaround surface and makes examples rely on *_ref APIs.

## 対象

- `stdlib/alloc/collections/stack.nepl, stdlib/tests/stack.n.md, tests/stdlib/stack_collections.n.md, examples/rpn.nepl, examples/bf.nepl`

## 根拠

- `stdlib/alloc/collections/stack.nepl` の `len` / `is_empty` / `peek` は `Stack<T>` を値で受け取り、観測だけで owner を閉じる API になっていた。
- そのため実利用側は `len_ref` / `is_empty_ref` / `peek_ref` / `get_ref` に逃げ、primary API と borrowed observer API が重複していた。
- RPN / BF examples は stack observer の `*_ref` 依存も持っており、現在の primary API と examples の所有権表現が一致していなかった。

## 問題

Stack still exposes len/is_empty/peek as owner-consuming observers while len_ref/is_empty_ref/peek_ref/get_ref are used for borrowed reads. This preserves the old workaround surface and makes examples rely on *_ref APIs.

## 影響

Parser and RPN-style stack workflows need frequent read-only observation before mutation. Keeping duplicate observer names lets examples hide owner movement instead of expressing borrowed reads directly through the primary API.

## 修正方針

Make Stack primary read-only observers borrow the owner, remove duplicate *_ref observer APIs where covered by primary names, and update examples/tests/source-policy. Keep true mutating APIs such as pop_top owner-preserving and explicit.

## 検証

node nodesrc/tests.js -i stdlib/alloc/collections/stack.nepl -i stdlib/tests/stack.n.md -i tests/stdlib/stack_collections.n.md -i examples/rpn.nepl -i examples/bf.nepl --no-tree -o tmp/stack-primary-borrowed-observers.json -j 1 --dist web/dist

## 2026-05-07 対応結果

- `Stack` の primary read-only observer を `&Stack<T>` 受け取りへ統一し、`len_ref` / `is_empty_ref` / `peek_ref` / `get_ref` を削除した。
- `pop_top` / `push` / `clear` / `free` など owner を更新または解放する API は明示的な owner flow のまま維持した。
- stack doctest / collection regression / pipe overload regression / RPN / BF examples を borrowed primary observer 名へ更新した。
- remote main の `ISS-20260507T100459865Z-EXAMPLES-RELY-ON-QUALIFIED-ALLOC-STR-B6D68CEA` と同期し、examples の string import 整理を維持した上で Stack observer 呼び出しだけを primary borrowed names へ移行した。
- `nodesrc/test_stdlib_stack_no_unsafe_unwraps.js` に、Stack observer が owner を消費しないこと、削除済み `*_ref` API が戻らないこと、tests/examples が primary borrowed names を使うことを固定する回帰検査を追加した。
- focused verification:
  - `node nodesrc/test_stdlib_stack_no_unsafe_unwraps.js`
  - `node nodesrc/tests.js -i stdlib/alloc/collections/stack.nepl -i stdlib/tests/stack.n.md -i tests/stdlib/stack_collections.n.md -i examples/rpn.nepl -i examples/rpn_legacy.nepl -i examples/bf.nepl --no-tree -o tmp/stack-primary-borrowed-observers-stack-examples.json -j 1 --dist web/dist`
  - `node nodesrc/run_doctest.js -i tests/stdlib/pipe_collections.n.md -n 2 --dist web/dist`
  - `node nodesrc/run_doctest.js -i tests/compiler/overload.n.md -n 15 --dist web/dist`
  - `node nodesrc/run_doctest.js -i tests/compiler/overload.n.md -n 19 --dist web/dist`
  - `node nodesrc/run_doctest.js -i tests/compiler/overload.n.md -n 20 --dist web/dist`
