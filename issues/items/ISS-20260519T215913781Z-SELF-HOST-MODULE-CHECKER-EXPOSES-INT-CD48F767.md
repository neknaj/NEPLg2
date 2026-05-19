---
id: ISS-20260519T215913781Z-SELF-HOST-MODULE-CHECKER-EXPOSES-INT-CD48F767
title: "Self-host module checker exposes internal proof adapters as public API"
area: selfhost
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-05-19
updated: 2026-05-20
target: "stdlib/neplg2/core/check/module.nepl, nodesrc/test_selfhost_proof_entry_contract.js"
---

# ISS-20260519T215913781Z-SELF-HOST-MODULE-CHECKER-EXPOSES-INT-CD48F767: Self-host module checker exposes internal proof adapters as public API

## 概要

check/module.nepl publishes internal proof adapters, diagnostic builders, item classifiers, and recursion helpers. External code can couple to checker-local phases instead of using selfhost_check_module_ast and typed summary accessors.

## 対象

- `stdlib/neplg2/core/check/module.nepl, nodesrc/test_selfhost_proof_entry_contract.js`

## 根拠

- `check/module.nepl` は module item stream の public checker entry を持つ一方で、raw backend / directive proof adapter、item kind classifier、diagnostic builder、loop state constructor、recursive loop まで `pub fn` として公開していた。
- external module がこれらの helper に依存すると、`selfhost_check_module_ast` の検査順序と `core/proof` の typed boundary を通らず、checker の途中 state へ直接結合できる。
- `SelfhostModuleCheckStep` は 1 item 検査中の transient state であり、後続 stage が参照すべき stable artifact ではない。

## 問題

check/module.nepl publishes internal proof adapters, diagnostic builders, item classifiers, and recursion helpers. External code can couple to checker-local phases instead of using selfhost_check_module_ast and typed summary accessors.

## 影響

Future resolve, type, effect, lifetime, owner, and Resource IR stages can depend on unstable checker internals, making the static-check pipeline harder to audit and easier to bypass.

## 修正方針

Keep only selfhost_check_module_ast and intentional SelfhostModuleCheckSummary accessors public; make proof adapters, item classifiers, diagnostic builders, loop, and state constructors private. Add source-policy regression for the public surface.

## 検証

Run selfhost proof/checker source policy and focused doctests, issue check, and git diff check.

## 修正内容

- `SelfhostModuleCheckStep` を private struct にした。
- `selfhost_check_module_ast` と `SelfhostModuleCheckSummary` の読み取り accessor だけを public API として残した。
- summary constructor / record、proof adapter、raw backend / directive item classifier、diagnostic builder、finish check、recursive loop は private `fn` に変更した。
- `nodesrc/test_selfhost_proof_entry_contract.js` に module checker public API allowlist を追加し、内部 proof adapter や state helper が再公開される退行を拒否するようにした。

## 検証結果

- `node nodesrc/test_selfhost_proof_entry_contract.js`: pass
- `node nodesrc/tests.js -i stdlib/neplg2/core/check/module.nepl --no-tree -o tmp/agent1-selfhost-module-check-public-surface-source.json -j 1 --dist web/dist --assert-io`: total=1, passed=1
- `node nodesrc/tests.js -i tests/stdlib/neplg2_checker.n.md --no-tree -o tmp/agent1-selfhost-module-check-public-surface-checker-nmd.json -j 1 --dist web/dist --assert-io`: total=3, passed=3
- `node nodesrc/tests.js -i tests/stdlib/neplg2_proof.n.md --no-tree -o tmp/agent1-selfhost-module-check-public-surface-proof-nmd.json -j 1 --dist web/dist --assert-io`: total=3, passed=3
