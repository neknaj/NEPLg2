---
id: ISS-20260519T003305752Z-STDLIB-HASH-DOCTEST-PRINTS-CHECKS-RE-3BC249ED
title: "stdlib hash doctest prints checks report without stdout fixture"
area: TEST
status: fixed
resolved: true
priority: P1
type: test
created: 2026-05-19
updated: 2026-05-19
target: "stdlib/tests/hash.n.md, nodesrc/test_stdlib_hash_nmd_report_contract.js, issues/items/ISS-20260429T102425370Z-N-MD-TESTS-RELY-ON-RETURN-VALUES-INS-9B49EDAD.md"
---

# ISS-20260519T003305752Z-STDLIB-HASH-DOCTEST-PRINTS-CHECKS-RE-3BC249ED: stdlib hash doctest prints checks report without stdout fixture

## 概要

stdlib/tests/hash.n.md verifies FNV-1a, Hash trait dispatch, and SHA-256 known digests through checks_print_report and checks_exit_code, but the manifest does not pin stdout / exit_code metadata and still uses the legacy checks_* report path.

## 対象

- `stdlib/tests/hash.n.md, nodesrc/test_stdlib_hash_nmd_report_contract.js, issues/items/ISS-20260429T102425370Z-N-MD-TESTS-RELY-ON-RETURN-VALUES-INS-9B49EDAD.md`

## 根拠

- `stdlib/tests/hash.n.md` は FNV-1a、`Hash` trait dispatch、SHA-256 の既知 digest 3種を検査していた。
- 旧実装は `checks_print_report` / `checks_exit_code` を呼んでいたが、manifest に `stdio` / `normalize_newlines` / `stdout:` / `exit_code:` がなく、CI の入出力 fixture として hash 検査の内容が固定されていなかった。
- SHA-256 は selfhost 側でも安定 digest が必要な基盤機能なので、単なる exit status ではなく、どの digest family を検査したかが stdout report に残る必要がある。

## 問題

stdlib/tests/hash.n.md verifies FNV-1a, Hash trait dispatch, and SHA-256 known digests through checks_print_report and checks_exit_code, but the manifest does not pin stdout / exit_code metadata and still uses the legacy checks_* report path.

## 影響

Hash algorithm and trait hashing regressions can pass or fail with only a process status, without fixture-checked assertion labels for the digest families that selfhost code depends on.

## 修正方針

Migrate hash_main to a named TestReport stdout fixture with exit_code metadata, keep byte-level SHA-256 proof in source code, and add a source policy contract rejecting ret-only or legacy checks_* regression.

## 検証

Run the hash source policy contract, focused hash doctest, source policy regressions, issue check, and diff whitespace check.

## 対応結果

- `hash_main` を `neplg2:test[stdio, normalize_newlines]`、`exit_code: 0`、deterministic `stdout:` fixture へ移行した。
- FNV-1a の既知値、Hash trait の同一入力安定性、別入力差分、SHA-256 empty / abc / multi-block digest を named `TestReport` の9 assertionとして固定した。
- SHA-256 digest は stdout を過度に肥大化させず、source 側の `sha256_digest_matches_loop` で32 byteすべてを照合する形にした。
- `nodesrc/test_stdlib_hash_nmd_report_contract.js` を追加し、`ret:` 代用、stdout fixture 欠落、旧 `checks_*` 形式、byte-level digest proof の欠落への退行を source policy で拒否するようにした。
- `nodesrc/run_source_policy_regressions.js` に同 contract を登録した。

## 関連

- [`.n.md tests rely on return values instead of stdout assertion reports`](./ISS-20260429T102425370Z-N-MD-TESTS-RELY-ON-RETURN-VALUES-INS-9B49EDAD.md)
