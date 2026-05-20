---
id: ISS-20260520T095125772Z-STDIO-FD-WRITE-HELPERS-LACK-DECLARAT-ADAF46CD
title: "stdio fd boundary doctests exposed invalid qualified boolean chains"
area: stdlib
status: fixed
resolved: true
priority: P1
type: doc
created: 2026-05-20
updated: 2026-05-20
target: "stdlib/std/stdio/write/fd.nepl, stdlib/std/stdio/read/buffer.nepl, nodesrc/test_stdlib_documentation_contract.js"
---

# ISS-20260520T095125772Z-STDIO-FD-WRITE-HELPERS-LACK-DECLARAT-ADAF46CD: stdio fd boundary doctests exposed invalid qualified boolean chains

## 概要

The global stdlib documentation contract reported declarationNoDoctest=1035 while the baseline was 1032. The remaining increase came from public stdio fd write helpers added during the raw MemPtr boundary split. Adding executable doctests exposed that stdio read/write raw boundary checks still used invalid qualified prefix boolean chains such as `math::or math::lt ...`, so the examples could not be treated as reliable executable documentation until the boundary predicates were expressed as typed bool values.

## 対象

- `stdlib/std/stdio/write/fd.nepl`
- `stdlib/std/stdio/read/buffer.nepl`
- `nodesrc/test_stdlib_documentation_contract.js`

## 根拠

- `node nodesrc/test_stdlib_documentation_contract.js` reported declarationNoDoctest=1035 before this issue and 1032 after adding the missing fd write doctests.
- Focused doctest execution for `stdlib/std/stdio/write/fd.nepl` initially exposed an invalid `math::or math::or math::le ...` chain in the fd write raw scratch guard.
- Focused doctest execution for `stdlib/std/stdio/read/buffer.nepl` exposed the same pattern in the fd read raw scratch guard, read-buffer finalization, and read loop byte-count validation.

## 問題

The fd write helpers had documentation comments but no declaration doctests, so source-policy could not prove that the low-level fd APIs still had executable examples after the raw MemPtr boundary split. When doctests were added, the stdio boundary code revealed nested qualified prefix operator chains that were not valid typed source forms. This hid real type-check failures behind the missing-doc-test warning.

## 影響

The global documentation contract warned after unrelated compiler/static-check policy checks were clean. More importantly, stdio read/write raw boundary validation contained source that was harder for the compiler to type-check and for reviewers to audit, weakening the safety of the executable documentation around low-level fd I/O.

## 修正方針

Add meaningful neplg2 doctests to the public stdio fd write helpers instead of raising the baseline. Split all discovered nested qualified boolean chains in stdio read/write raw boundary checks into named `<bool>` intermediate values so the source is directly type-checkable and the predicates are reviewable.

## 検証

- `node nodesrc/test_stdlib_documentation_contract.js`: pass, declarationNoDoctest=1032
- `node nodesrc/tests.js -i stdlib/std/stdio/write/fd.nepl --no-tree --dist web/dist -o tmp/agent1-stdio-fd-write-doc.json -j 1 --assert-io`: 3 passed
- `node nodesrc/tests.js -i stdlib/std/stdio/read/buffer.nepl --no-tree --dist web/dist -o tmp/agent1-stdio-read-buffer-bool-chain.json -j 1 --assert-io`: 3 passed
