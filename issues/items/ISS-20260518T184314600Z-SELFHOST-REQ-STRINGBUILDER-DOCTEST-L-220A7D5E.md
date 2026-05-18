---
id: ISS-20260518T184314600Z-SELFHOST-REQ-STRINGBUILDER-DOCTEST-L-220A7D5E
title: "selfhost_req StringBuilder doctest leaks built string owner"
area: stdlib
status: open
resolved: false
priority: P1
type: bug
created: 2026-05-18
updated: 2026-05-18
target: "tests/stdlib/selfhost_req.n.md, stdlib/alloc/string/builder/**, stdlib/alloc/io/bytebuilder/**, nepl-core/src/resource/**"
---

# ISS-20260518T184314600Z-SELFHOST-REQ-STRINGBUILDER-DOCTEST-L-220A7D5E: selfhost_req StringBuilder doctest leaks built string owner

## 概要

Focused verification for the HashMap/HashSet update-error work still leaves tests/stdlib/selfhost_req.n.md::doctest#5 failing with resource.owner.leak after sb_build returns a str and the fixture only observes its length. The StringBuilder/str contract is inconsistent: str is documented as a non-owning view, but Resource IR reports an owned storage obligation with no clear public cleanup path in the doctest.

## 対象

- `tests/stdlib/selfhost_req.n.md, stdlib/alloc/string/builder/**, stdlib/alloc/io/bytebuilder/**, nepl-core/src/resource/**`

## 根拠

- `node nodesrc/tests.js -i tests/stdlib/selfhost_req.n.md --no-tree -o tmp/agent1-selfhost-req-hash-update-error.json -j 1 --dist web/dist --assert-io`: total=6, passed=5, failed=1。
- 失敗箇所は `tests/stdlib/selfhost_req.n.md::doctest#5` で、`sb_build` の戻り値 `res: str` を `len res` で観測した後、Resource IR が `resource.owner.leak` を報告する。

## 問題

Focused verification for the HashMap/HashSet update-error work still leaves tests/stdlib/selfhost_req.n.md::doctest#5 failing with resource.owner.leak after sb_build returns a str and the fixture only observes its length. The StringBuilder/str contract is inconsistent: str is documented as a non-owning view, but Resource IR reports an owned storage obligation with no clear public cleanup path in the doctest.

## 影響

selfhost_req cannot be used as a clean focused regression gate while StringBuilder-built strings expose unresolved owner obligations. Treating this as a doctest-only workaround would hide an ownership contract mismatch between StringBuilder, str observers, and Resource IR.

## 修正方針

Audit StringBuilder build, ByteBuilder finish, str ownership representation, and string len/observer contracts. The fix must be source-derived and generic: either prove built str owner transfer and cleanup through the compiler/resource model or expose an explicit safe cleanup/ownership contract, then update selfhost_req doctest#5 to exercise that contract.

## 検証

node nodesrc/tests.js -i tests/stdlib/selfhost_req.n.md --no-tree -o tmp/selfhost-req-stringbuilder-owner.json -j 1 --dist web/dist --assert-io should pass all six doctests without resource.owner.leak.
