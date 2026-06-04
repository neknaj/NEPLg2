---
id: ISS-20260604T033643672Z-DIAGS-BY-VALUE-OBSERVER-MUST-CLOSE-O-C6D3EAEA
title: "Diags by-value observer must close owner after borrowed observation"
area: stdlib
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-06-04
updated: 2026-06-04
target: stdlib/alloc/diag/error/diags.nepl, nodesrc/test_stdlib_diag_error_no_unsafe_unwraps.js
---

# ISS-20260604T033643672Z-DIAGS-BY-VALUE-OBSERVER-MUST-CLOSE-O-C6D3EAEA: Diags by-value observer must close owner after borrowed observation

## 概要

`node nodesrc/test_stdlib_diag_error_no_unsafe_unwraps.js` reports that by-value `diags_has_errors` must close the `Diags` owner after observing via `&Diags`. The stdlib implementation already uses the borrow-then-free-then-return pattern, but the source policy was tied to a local variable name and therefore failed to prove the ownership contract. This weakened the Zenn policy requirement that static checks verify the real invariant rather than incidental spelling.

## 対象

- `stdlib/alloc/diag/error/diags.nepl`
- `nodesrc/test_stdlib_diag_error_no_unsafe_unwraps.js`

## 根拠

- `stdlib/alloc/diag/error/diags.nepl` defines the by-value overload as `let has_error %bool diags_has_errors &ds`, then `diags_free ds`, then returns `has_error`.
- The failing policy expected `let ok <bool> ...` exactly, so a correct implementation became invisible to the policy solely because the local variable was named `has_error`.
- `diags_push` had the same policy smell: it required the recovered `Vec<Diag>` local to be named `items`, while the implementation used `diag_items` consistently.

## 問題

`nodesrc/test_stdlib_diag_error_no_unsafe_unwraps.js` did not model the ownership contract directly. It overfit variable names in NEPL source and could fail on correct code or encourage meaningless renames. That is a static-check precision problem: the policy must inspect the borrow, owner close, and returned observation value as a relation.

## 影響

Diagnostic containers can become a precedent for observer APIs that consume owners without an explicit cleanup contract, weakening ownership conventions across selfhost diagnostics.

## 修正方針

Keep the stdlib by-value overload contract as borrow-then-free-then-return. Replace the source policy checks with relation-based regular expressions that capture the observed local and verify that the same local is returned after `diags_free ds`. Apply the same idea to `diags_push`, where the recovered `Vec<Diag>` owner must be the value passed to `vec_push::push`.

## 検証

- `node nodesrc/test_stdlib_diag_error_no_unsafe_unwraps.js`: pass
- `node nodesrc/tests.js -i stdlib/alloc/diag/error/diags.nepl -i stdlib/tests/error.n.md -i stdlib/tests/diag.n.md --no-tree -o tmp/agent2-diags-has-errors-tests.json -j 1 --dist web/dist --assert-io`: total=6, passed=6, failed=0
- `node nodesrc/run_source_policy_regressions.js --warn-only`: `nodesrc/test_stdlib_diag_error_no_unsafe_unwraps.js` pass。既存 warning は 13 件から 12 件へ減少
- `node nodesrc/issues.js index --dir issues && node nodesrc/issues.js check --dir issues`: pass
- `trunk build`: pass
- `node nodesrc/cli.js -i tests/playground_editor --playground-editor-tests -o json=tmp/agent2-diags-has-errors-playground-editor.json`: 13/13 pass
