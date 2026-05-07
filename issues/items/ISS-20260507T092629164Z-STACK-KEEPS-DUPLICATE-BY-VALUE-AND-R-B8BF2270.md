---
id: ISS-20260507T092629164Z-STACK-KEEPS-DUPLICATE-BY-VALUE-AND-R-B8BF2270
title: "Stack keeps duplicate by-value and *_ref observer APIs"
area: stdlib
status: open
resolved: false
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

- 未記入

## 問題

Stack still exposes len/is_empty/peek as owner-consuming observers while len_ref/is_empty_ref/peek_ref/get_ref are used for borrowed reads. This preserves the old workaround surface and makes examples rely on *_ref APIs.

## 影響

Parser and RPN-style stack workflows need frequent read-only observation before mutation. Keeping duplicate observer names lets examples hide owner movement instead of expressing borrowed reads directly through the primary API.

## 修正方針

Make Stack primary read-only observers borrow the owner, remove duplicate *_ref observer APIs where covered by primary names, and update examples/tests/source-policy. Keep true mutating APIs such as pop_top owner-preserving and explicit.

## 検証

node nodesrc/tests.js -i stdlib/alloc/collections/stack.nepl -i stdlib/tests/stack.n.md -i tests/stdlib/stack_collections.n.md -i examples/rpn.nepl -i examples/bf.nepl --no-tree -o tmp/stack-primary-borrowed-observers.json -j 1 --dist web/dist
