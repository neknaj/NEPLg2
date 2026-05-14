---
id: ISS-20260514T071055890Z-SELFHOST-REQ-DOCTESTS-STILL-RELY-ON--F9CC30E7
title: "selfhost_req doctests still rely on stale implicit imports"
area: stdlib
status: open
resolved: false
priority: P2
type: test
created: 2026-05-14
updated: 2026-05-14
target: tests/stdlib/selfhost_req.n.md
---

# ISS-20260514T071055890Z-SELFHOST-REQ-DOCTESTS-STILL-RELY-ON--F9CC30E7: selfhost_req doctests still rely on stale implicit imports

## 概要

Focused verification during the StringBuilder owner-boundary migration still leaves tests/stdlib/selfhost_req.n.md doctest#1 and doctest#2 failing with resolve.identifier.undefined for symbols such as len and unwrap_ok, followed by cascading collection helper errors. The StringBuilder doctest in the same file passes, so this is a stale fixture/import problem rather than the current owner-boundary change.

## 対象

- `tests/stdlib/selfhost_req.n.md`

## 根拠

- `node nodesrc/tests.js -i stdlib/alloc/string/builder -i stdlib/alloc/string/builder_ext.nepl -i tests/stdlib/nm.n.md -i tests/stdlib/selfhost_req.n.md -i tests/stdlib/neplg2_text.n.md --no-tree -o tmp/agent1-stringbuilder-bytebuilder-focused.json -j 1 --dist web/dist`: 15 total / 13 passed / 2 failed。
- `tests/stdlib/selfhost_req.n.md::doctest#1` は `consume_str` 内の `len s` が `resolve.identifier.undefined`。
- `tests/stdlib/selfhost_req.n.md::doctest#2` は `unwrap_ok new<u8>`、`unwrap_ok push<u8>`、`get<u8>` などが未定義または overload mismatch へ連鎖。
- 同じ run の `tests/stdlib/selfhost_req.n.md::doctest#5`、つまり `test_req_string_builder` は passed であり、今回の `StringBuilder` owner-boundary 変更とは切り分けられる。

## 問題

Focused verification during the StringBuilder owner-boundary migration still leaves tests/stdlib/selfhost_req.n.md doctest#1 and doctest#2 failing with resolve.identifier.undefined for symbols such as len and unwrap_ok, followed by cascading collection helper errors. The StringBuilder doctest in the same file passes, so this is a stale fixture/import problem rather than the current owner-boundary change.

## 影響

Broad stdlib verification can report unrelated failures and hide the actual safety regression under test. It also violates the doctest policy that examples must declare the APIs they use directly.

## 修正方針

Audit selfhost_req.n.md and add explicit imports or update the examples to current stdlib APIs. Keep the self-host requirements as executable doctests; do not delete examples to reduce failures.

## 検証

node nodesrc/tests.js -i tests/stdlib/selfhost_req.n.md --no-tree -o tmp/selfhost-req-imports-fixed.json -j 1 --dist web/dist should pass all doctests.
