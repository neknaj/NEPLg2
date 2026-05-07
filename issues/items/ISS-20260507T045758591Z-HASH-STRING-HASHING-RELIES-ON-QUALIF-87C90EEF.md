---
id: ISS-20260507T045758591Z-HASH-STRING-HASHING-RELIES-ON-QUALIF-87C90EEF
title: "hash string hashing relies on qualified alloc/string facade re-export"
area: stdlib
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-07
updated: 2026-05-07
target: "stdlib/alloc/hash/hash32.nepl, stdlib/core/traits/hash.nepl, stdlib/tests/hash.n.md"
---

# ISS-20260507T045758591Z-HASH-STRING-HASHING-RELIES-ON-QUALIF-87C90EEF: hash string hashing relies on qualified alloc/string facade re-export

## 概要

alloc/hash/hash32.nepl and core/traits/hash.nepl import alloc/string as string and call string::len / string::string_byte_at_unchecked. After alloc/string became a facade, qualified calls through the broad facade are not stable, and the old alloc/diag/error transitive string import masked the problem in some suites.

## 対象

- `stdlib/alloc/hash/hash32.nepl, stdlib/core/traits/hash.nepl, stdlib/tests/hash.n.md`

## 根拠

- `tests/stdlib/collections_diag.n.md::doctest#1/#2` が、`alloc/diag/error` split 後に `alloc/hash/hash32.nepl` と `stdlib/core/traits/hash.nepl` の `string::len` / `string::string_byte_at_unchecked` 解決で失敗した。
- `alloc/string.nepl` は submodule re-export facade であり、qualified byte access は実装元の `alloc/string/access` を直接 import すべきだった。

## 問題

alloc/hash/hash32.nepl and core/traits/hash.nepl import alloc/string as string and call string::len / string::string_byte_at_unchecked. After alloc/string became a facade, qualified calls through the broad facade are not stable, and the old alloc/diag/error transitive string import masked the problem in some suites.

## 影響

HashMap/HashSet and Hash trait users can fail during name resolution depending on unrelated transitive imports, making stdlib collection tests sensitive to import order and blocking diagnostic facade cleanup.

## 修正方針

Import alloc/string/access directly for qualified byte access in hash32, Hash<str>, and the hash doctest. Add a source policy regression so hash code does not depend on the broad string facade for qualified accessors.

## 検証

- `node nodesrc/test_stdlib_hash_string_access_boundary.js`: passed
- `node nodesrc/tests.js -i tests/stdlib/collections_diag.n.md --no-tree -o tmp/diag-error-module-split-collections-diag-fixed.json -j 1 --dist web/dist`: total=4, passed=4
- `node nodesrc/tests.js -i stdlib/alloc/hash/hash32.nepl -i stdlib/core/traits/hash.nepl --no-tree -o tmp/hash-string-access-impl-doctests.json -j 1 --dist web/dist`: total=1, passed=1
- `stdlib/tests/hash.n.md` の full doctest は `sha256_rounds_loop` の `resource.cell.uninit` で失敗したため、`ISS-20260507T050025343Z-SHA256-HASH-DOCTEST-FAILS-RESOURCE-I-A4EE25CE` として分離した。

## 解決内容

`stdlib/alloc/hash/hash32.nepl`、`stdlib/core/traits/hash.nepl`、`stdlib/tests/hash.n.md` の qualified string byte access を `#import "alloc/string/access" as string` に変更した。

これにより `alloc/diag/error` など無関係な module が broad `alloc/string` を transitive import しているかどうかで hash/collection test の解決結果が変わる状態を解消した。再発防止として `nodesrc/test_stdlib_hash_string_access_boundary.js` を追加し、source policy に登録した。
