---
id: ISS-20260426T182345763Z-SHA256-PUBLIC-API-TRAPS-ON-ALLOCATIO-57C4D357
title: "sha256 public API traps on allocation failure through unsafe unwraps"
area: stdlib
status: verified
resolved: true
priority: P2
type: bug
created: 2026-04-26
updated: 2026-04-27
target: "stdlib/alloc/hash/sha256.nepl, stdlib/tests/hash.n.md"
---

# ISS-20260426T182345763Z-SHA256-PUBLIC-API-TRAPS-ON-ALLOCATIO-57C4D357: sha256 public API traps on allocation failure through unsafe unwraps

## 概要

new_sha256, sha256_update, sha256_finalize and internal digest/schedule builders call unwrap_ok/unwrap on Vec allocation and indexing paths. Allocation failure becomes unreachable instead of Result, contradicting the stdlib Result API policy and the broader RV-STDLIB-010 unsafe helper cleanup.

## 対象

- `stdlib/alloc/hash/sha256.nepl, stdlib/tests/hash.n.md`

## 根拠

- `stdlib/alloc/hash/sha256.nepl` の `new_sha256` / `sha256_update` / `sha256_finalize` が、allocation-bearing な `Vec` API を `unwrap_ok` で受けていた。
- message schedule / digest builder も `get_ref` の `Option` と `push` / `with_capacity` の `Result` を unsafe helper で潰していた。
- self-host 計画では source / artifact fingerprint 用の hash 基盤が必要であり、通常入力で `OutOfMemory` を値として返せない API は compiler pipeline の診断方針と合わない。

## 問題

new_sha256, sha256_update, sha256_finalize and internal digest/schedule builders call unwrap_ok/unwrap on Vec allocation and indexing paths. Allocation failure becomes unreachable instead of Result, contradicting the stdlib Result API policy and the broader RV-STDLIB-010 unsafe helper cleanup.

## 影響

Self-host source/artifact fingerprinting would be unable to report OutOfMemory from SHA-256 construction/update/finalize and can trap in normal input-dependent code.

## 修正方針

Make SHA-256 allocation-bearing APIs and internal builders return Result<..., StdErrorKind>, propagate errors with match, keep bounded schedule access structurally safe, and update tests to exercise the Result API.

## 検証

- `node nodesrc/tests.js -i stdlib/tests/hash.n.md --no-tree -o tmp/hash-sha256-result-api-after-rebase.json -j 1`: 1/1 pass
- `node nodesrc/tests.js -i stdlib --no-tree -o tmp/stdlib-sha256-result-api-after-rebase.json -j 4`: 414/414 pass
- `node nodesrc/tests.js -i tests/stdlib --no-tree -o tmp/tests-stdlib-sha256-result-api-after-rebase.json -j 4`: 282/282 pass
- `node nodesrc/test_stdlib_sha256_no_unsafe_unwraps.js`: pass
- `node nodesrc/test_stdlib_match_decision_trees.js`: pass
- `trunk build`: pass
- `node nodesrc/cli.js -i tests/playground_editor --playground-editor-tests -o json=tmp/playground-sha256-result-api-after-rebase.json`: 13/13 pass
- `node nodesrc/issues.js check`: pass (`files=139`)
- `git diff --check`: pass

## 修正内容

- SHA-256 の公開 API を `Result<..., StdErrorKind>` に変更し、allocation failure を `StdErrorKind::OutOfMemory` として伝播するようにした。
- `sha256_k` / length field helper / schedule / round / digest builder の内部前提崩れも `StdErrorKind::InvalidOperation` または `IndexOutOfBounds` として返すようにした。
- hash doctest を `Result` API に更新し、digest byte 取得も `Option` match に変更した。
- `nodesrc/test_stdlib_sha256_no_unsafe_unwraps.js` を追加し、CI build job と `doc/testing.md` に source policy regression として登録した。
