---
id: ISS-20260526T073859722Z-NEPLG2-1-SOURCE-POLICY-UNIT-AND-CONS-F81D1534
title: "NEPLg2.1 source policy unit and constructor cleanup drift"
area: tests
status: fixed
resolved: true
priority: P1
type: maintenance
created: 2026-05-26
updated: 2026-05-26
target: "nodesrc/test_*.js; nodesrc/source_policy/**"
---

# ISS-20260526T073859722Z-NEPLG2-1-SOURCE-POLICY-UNIT-AND-CONS-F81D1534: NEPLg2.1 source policy unit and constructor cleanup drift

## 概要

After the NEPLg2.1 unit keyword migration and constructor postfix cleanup, run_source_policy_regressions --warn-only reports stale source policy failures that still expect () unit spelling or typed constructor/helper postfix syntax.

## 対象

- `nodesrc/test_*.js; nodesrc/source_policy/**`

## 根拠

- `node nodesrc/run_source_policy_regressions.js --warn-only` は 2026-05-26 の constructor/helper postfix cleanup 後に完走したが、29 件の warning を報告した。
- 代表例として、`nodesrc/test_stdlib_io_bytebuf_owner_boundary.js` は `io_bytebuf_empty <()->ByteBuf> ():` を期待していたが、実 source の NEPLg2.1 view は `io_bytebuf_empty <(unit)->ByteBuf> (unit):` になっている。
- `nodesrc/test_stdlib_streamio_writer_boundary.js` は `Result<StreamWriter,str>::Ok` を期待していたが、constructor cleanup 後の実 source は `Result::Ok` を使う。
- collection / selfhost の今回変更範囲では、`test_stdlib_collection_cleanup_contract.js`、`test_selfhost_*_absence.js`、collection family の no-unsafe/update/storage policy を個別に更新して pass を確認した。

## 問題

After the NEPLg2.1 unit keyword migration and constructor postfix cleanup, run_source_policy_regressions --warn-only reports stale source policy failures that still expect () unit spelling or typed constructor/helper postfix syntax.

## 影響

Warn-only source policy noise hides real static-inspection regressions during the NEPLg2.1 cleanup and makes main-merge readiness harder to judge.

## 修正方針

Migrate the residual policy tests to legacyTypeSyntaxView, fnSignaturePattern, or direct NEPLg2.1 regexes while preserving owner-boundary and API-boundary assertions.

## 進捗

- 2026-05-26: collection helper postfix cleanup に直結する source policy は更新済み。`none<T>` / `some<T>` 前提を postfix-free constructor へ移し、`VecStorageInvariant` proof の検査は出現順に依存しない構造確認へ直した。
- 2026-05-26: root cause を再確認し、`unit` は実引数ではなく zero-argument marker なので、`legacyTypeSyntaxView` は `fn unit T` / `\unit` を旧 view の `()` として扱うように修正した。`fnSignaturePattern(..., [])` も NEPLg2.1 の `%fn unit T` を期待する。
- 2026-05-26: 5 worker に分割し、collection、stdio/streamio、ByteBuf/builder/Vec、documentation/tutorial/resource/selfhost、diag/std_test/kpgraph/wasix の source policy を並列更新した。
- 2026-05-26: documentation contract は baseline を緩めず、`adjacency_matrix_bit_index` に契約・計算量・doctest 付き doc comment を追加して `declarationNoDoc` を既存 baseline 530 へ戻した。
- 2026-05-26: `node nodesrc/run_source_policy_regressions.js --warn-only` は warning 0 件になった。

## 検証

- `node nodesrc/test_source_policy_nepl_source_view.js`
- `node nodesrc/run_source_policy_regressions.js --warn-only`
- `node nodesrc/test_stdlib_documentation_contract.js`
- `node nodesrc/tests.js -i stdlib/alloc/collections/adjacency_matrix/layout.nepl --no-tree -o tmp/neplg21-adjacency-layout-doc.json -j 1 --dist web/dist --assert-io` (compile timeout after 60000ms; 型診断なし)
- `node nodesrc/neplg21_syntax_migrate.js --check`
- `node nodesrc/issues.js check --dir issues`
- `git diff --check`
