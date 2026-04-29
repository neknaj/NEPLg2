---
id: ISS-20260429T024412130Z-RESOURCE-OWNER-GATE-REPORTS-D3100-IN-7A19FECC
title: "Resource owner gate reports D3100 in self-host lexer and import scanner helpers"
area: core
status: open
resolved: false
priority: P1
type: bug
created: 2026-04-29
updated: 2026-04-29
target: "nepl-core/src/resource, stdlib/neplg2/core/syntax/lexer.nepl, stdlib/neplg2/core/module/import_spec.nepl, tests/stdlib/neplg2_lexer.n.md, tests/stdlib/neplg2_import_spec.n.md"
---

# ISS-20260429T024412130Z-RESOURCE-OWNER-GATE-REPORTS-D3100-IN-7A19FECC: Resource owner gate reports D3100 in self-host lexer and import scanner helpers

## 概要

After latest Resource owner changes, self-host lexer and import spec fixtures fail before runtime with D3100 owner obligation leaks in lex_all_loop and import parser helper paths. This blocks scanner regression tests even when string helper structure tests pass.

## 対象

- `nepl-core/src/resource, stdlib/neplg2/core/syntax/lexer.nepl, stdlib/neplg2/core/module/import_spec.nepl, tests/stdlib/neplg2_lexer.n.md, tests/stdlib/neplg2_import_spec.n.md`

## 根拠

- `node nodesrc/tests.js -i tests\stdlib\neplg2_lexer.n.md --no-tree -o tmp\neplg2-lexer-prefix-at-rebased.json -j 1` は 13 件すべて compile phase で失敗し、top issue は `lex_all_loop__...` の `D3100 resource ir owner obligation may leak`。
- `node nodesrc/tests.js -i tests\stdlib\neplg2_import_spec.n.md --no-tree -o tmp\neplg2-import-prefix-at-rebased.json -j 1` は 3 件すべて compile phase で失敗し、`hash32(str)` と `main` temporary の D3100 が出る。
- 同じ checkout で `node nodesrc/test_selfhost_string_helpers_boundary.js` と `node nodesrc/tests.js -i stdlib\alloc\string.nepl --no-tree -o tmp\string-prefix-at-rebased.json -j 1` は成功しているため、prefix helper 実装ではなく self-host scanner fixtures を覆う Resource owner gate 残件として扱う。

## 問題

After latest Resource owner changes, self-host lexer and import spec fixtures fail before runtime with D3100 owner obligation leaks in lex_all_loop and import parser helper paths. This blocks scanner regression tests even when string helper structure tests pass.

## 影響

Self-host scanner refactors cannot be validated through behavioral fixtures; new lexer/import parsing regressions may be hidden behind Resource IR owner false positives or real temporary ownership leaks.

## 修正方針

Trace the Resource IR owner flow for scanner Result/Vec temporaries, branch values, and string helper call results without weakening D3100. Add focused regression coverage for lex_all_loop and import_spec once the owner flow is corrected.

## 検証

node nodesrc/tests.js -i tests/stdlib/neplg2_lexer.n.md --no-tree -o tmp/neplg2-lexer-resource-owner-fixed.json -j 1; node nodesrc/tests.js -i tests/stdlib/neplg2_import_spec.n.md --no-tree -o tmp/neplg2-import-spec-resource-owner-fixed.json -j 1; D3100 should not mask scanner behavior.
