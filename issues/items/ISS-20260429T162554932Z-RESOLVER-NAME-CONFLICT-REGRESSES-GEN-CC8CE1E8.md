---
id: ISS-20260429T162554932Z-RESOLVER-NAME-CONFLICT-REGRESSES-GEN-CC8CE1E8
title: "Resolver name conflict regresses generics doctests"
area: core
status: open
resolved: false
priority: P1
type: bug
created: 2026-04-29
updated: 2026-04-29
target: "nepl-core/src/resolve, tests/compiler/generics.n.md, tests/compiler/shadowing.n.md"
---

# ISS-20260429T162554932Z-RESOLVER-NAME-CONFLICT-REGRESSES-GEN-CC8CE1E8: Resolver name conflict regresses generics doctests

## 概要

On current main after MemPtr explicit Clone merge, node nodesrc/tests.js -i tests/compiler/generics.n.md --no-tree -o tmp/memptr-explicit-clone-generics-after-merge.json -j 1 --dist web/dist reports total=24 passed=16 failed=8. The failing generics doctests compile-fail early with resolve.item.name_conflict. The wider tests/compiler run also reports tests/compiler/shadowing.n.md::doctest#22 as expected compile_fail, but compiled successfully. These failures are not on the MemPtr explicit Clone path and indicate a resolver/name-scope regression in the current compiler gate.

## 対象

- `nepl-core/src/resolve, tests/compiler/generics.n.md, tests/compiler/shadowing.n.md`

## 根拠

- 未記入

## 問題

On current main after MemPtr explicit Clone merge, node nodesrc/tests.js -i tests/compiler/generics.n.md --no-tree -o tmp/memptr-explicit-clone-generics-after-merge.json -j 1 --dist web/dist reports total=24 passed=16 failed=8. The failing generics doctests compile-fail early with resolve.item.name_conflict. The wider tests/compiler run also reports tests/compiler/shadowing.n.md::doctest#22 as expected compile_fail, but compiled successfully. These failures are not on the MemPtr explicit Clone path and indicate a resolver/name-scope regression in the current compiler gate.

## 影響

The compiler doctest gate is no longer clean for generic examples that previously served as broad typecheck coverage. Self-host work relies on generics and shadowing semantics, so resolver regressions must be tracked separately instead of being hidden by the MemPtr clone fix.

## 修正方針

Trace declaration and import name registration for generic doctest modules and shadowing compile_fail cases. Preserve strict duplicate item diagnostics, but ensure generated/imported prelude items do not collide with legitimate local generic examples, and ensure shadowing compile_fail fixtures still emit the intended diagnostic.

## 検証

node nodesrc/tests.js -i tests/compiler/generics.n.md --no-tree -o tmp/generics-resolver-regression.json -j 1 --dist web/dist; node nodesrc/tests.js -i tests/compiler/shadowing.n.md --no-tree -o tmp/shadowing-resolver-regression.json -j 1 --dist web/dist; then node nodesrc/tests.js -i tests/compiler --no-tree -o tmp/compiler-after-resolver-regression.json -j 4 --dist web/dist
