---
id: ISS-20260512T142631679Z-STDLIB-DOCUMENTATION-CONTRACT-IS-NOT-1FB48841
title: "Stdlib documentation contract is not globally enforced"
area: stdlib
status: fixed
resolved: true
priority: P1
type: doc
created: 2026-05-12
updated: 2026-05-12
target: "stdlib/**/*.nepl; nodesrc/test_stdlib_documentation_contract.js; doc/neplg2/stdlib_documentation_contract_plan.md"
---

# ISS-20260512T142631679Z-STDLIB-DOCUMENTATION-CONTRACT-IS-NOT-1FB48841: Stdlib documentation contract is not globally enforced

## 概要

Stdlib policy requires sufficiently detailed Japanese documentation and doctests for modules, functions, enums, structs, and traits, but there is no global source policy that measures this contract. A scan of stdlib/core, stdlib/alloc, and stdlib/std found 385 module files with module docs, but 309 module docs without doctest, 547 declarations without adjacent doc comments, and 1032 declarations without adjacent doctests.

## 対象

- `stdlib/**/*.nepl; nodesrc/test_stdlib_documentation_contract.js; doc/neplg2/stdlib_documentation_contract_plan.md`

## 根拠

- AGENTS.md の stdlib コメント方針は、ファイル先頭に module documentation を置き、各関数の前に目的・アルゴリズム・注意点・計算量・制約を丁寧に書くことを要求している。
- 2026-05-12 の監査で、`stdlib/core`、`stdlib/alloc`、`stdlib/std` の 385 file は全て module doc を持つ一方、module doctest missing 309、declaration doc missing 547、declaration doctest missing 1032 を確認した。
- 既存の source policy は個別 module の unsafe unwrap や facade 分割を監視しているが、stdlib 全体の documentation contract を測定する global policy はなかった。
- 詳細計画: [NEPLg2 stdlib documentation contract plan](../../doc/neplg2/stdlib_documentation_contract_plan.md)

## 問題

Stdlib policy requires sufficiently detailed Japanese documentation and doctests for modules, functions, enums, structs, and traits, but there is no global source policy that measures this contract. A scan of stdlib/core, stdlib/alloc, and stdlib/std found 385 module files with module docs, but 309 module docs without doctest, 547 declarations without adjacent doc comments, and 1032 declarations without adjacent doctests.

## 影響

Documentation can silently regress or be removed to keep files small, violating the stdlib maintenance contract and weakening executable API examples. Users and selfhost work lose reliable behavior examples near the code.

## 修正方針

Add a stdlib documentation contract plan and a nodesrc source policy that freezes the current audit baseline, rejects module docs disappearing, rejects documentation coverage getting worse, and provides a staged path to reduce missing declaration docs/doctests to zero without deleting documentation to shrink files.

## 対応記録

- `doc/neplg2/stdlib_documentation_contract_plan.md` を追加し、module / declaration documentation と doctest の必須契約、禁止事項、baseline、stage 別実装計画を明文化した。
- `nodesrc/test_stdlib_documentation_contract.js` を追加し、現時点の不足数を baseline として固定した。これは最終状態ではなく、documentation を削って悪化させる退行を拒否する凍結線である。
- `nodesrc/run_source_policy_regressions.js` に documentation contract policy を追加した。
- 残る 309 module doctest gap、547 declaration doc gap、1032 declaration doctest gap は計画に沿って段階的に 0 へ下げる。

## 検証

- `node nodesrc/test_stdlib_documentation_contract.js`
- `node nodesrc/run_source_policy_regressions.js --warn-only`
- `node nodesrc/issues.js index --dir issues`
- `node nodesrc/issues.js check --dir issues`

## 2026-05-13 bytebuilder helper doctest regression 修正

`stdlib/alloc/io/bytebuilder.nepl` に `byte_builder_dealloc_owned_u8` が追加された際、関数 doc はあるが関数 doctest が無く、documentation contract の `declarationNoDoctest` baseline が `1032 -> 1033` へ悪化していた。

この helper は `ByteBuilder` / `ByteBuf` が所有する byte storage の解放責務を集約する内部境界であり、doc を削って baseline に合わせるべきではない。`alloc_ptr<u8>` で得た所有 pointer を `byte_builder_dealloc_owned_u8` が閉じる正常系 doctest を追加し、source policy の悪化を解消した。

あわせて `byte_builder_dealloc_owned_u8` が `dealloc_ptr` の Err 分岐を `unreachable` で潰していた点も修正した。Resource IR は Err 分岐で owner が解放されない可能性を正しく検出するため、この helper は raw-memory-boundary 内の所有 pointer invariant 済み storage-only cleanup として、direct `dealloc_raw mem_ptr_addr ptr size` に下げる。これにより unsafe helper policy と owner obligation check の両方に沿う。

検証:

- `node nodesrc/test_stdlib_documentation_contract.js`
- `node nodesrc/test_stdlib_no_unsafe_helpers.js`
- `node nodesrc/test_stdlib_builder_owner_boundary.js`
- `node nodesrc/tests.js -i stdlib/alloc/io/bytebuilder.nepl --no-tree -o tmp/agent1-bytebuilder-doc-contract.json -j 1 --dist web/dist`
- `node nodesrc/run_source_policy_regressions.js`

## 2026-05-13 Agent 1 解決整理

この issue の主題である「stdlib documentation contract が global source policy として enforcement されていない」問題は解消済みと判断する。

現在の `nodesrc/test_stdlib_documentation_contract.js` は次を監視している。

- `stdlib/**/*.nepl` の module doc が 0 欠落であること。
- module doctest / declaration doc / declaration doctest の不足数が baseline より悪化しないこと。
- 不足数を隠すために doc comment を削る退行を拒否すること。
- `nodesrc/run_source_policy_regressions.js` から通常の source policy として実行されること。

2026-05-13 時点の確認では、`files=395`、`moduleNoDoc=0`、`moduleNoDoctest=309`、`declarationNoDoc=537`、`declarationNoDoctest=1027` で、global enforcement は機能している。残る不足数はこの issue の未解決ではなく、`doc/neplg2/stdlib_documentation_contract_plan.md` に沿って baseline を段階的に 0 へ下げる継続改善として扱う。

検証:

- `node nodesrc\test_stdlib_documentation_contract.js`
- `node nodesrc\issues.js check --dir issues`
