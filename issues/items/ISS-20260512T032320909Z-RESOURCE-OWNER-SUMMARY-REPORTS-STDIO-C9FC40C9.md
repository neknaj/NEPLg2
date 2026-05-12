---
id: ISS-20260512T032320909Z-RESOURCE-OWNER-SUMMARY-REPORTS-STDIO-C9FC40C9
title: "Resource owner summary reports stdio/ANSI string temporaries as maybe leaks"
area: static-check
status: open
resolved: false
priority: P1
type: bug
created: 2026-05-12
updated: 2026-05-12
target: "stdlib/std/stdio.nepl, stdlib/platforms/wasix/tui.nepl, nepl-core/src/resource, examples"
---

# ISS-20260512T032320909Z-RESOURCE-OWNER-SUMMARY-REPORTS-STDIO-C9FC40C9: Resource owner summary reports stdio/ANSI string temporaries as maybe leaks

## 概要

After origin/main 99433272, examples fail on main with resource.owner.maybe_leak in stdio/ANSI helpers such as print_i32__i32__unit__imp and ansi_text_style_code__AnsiTextStyle__str__pure. The failure reproduces on main without Fenwick changes.

## 対象

- `stdlib/std/stdio.nepl, stdlib/platforms/wasix/tui.nepl, nepl-core/src/resource, examples`

## 根拠

- 未記入

## 問題

After origin/main 99433272, examples fail on main with resource.owner.maybe_leak in stdio/ANSI helpers such as print_i32__i32__unit__imp and ansi_text_style_code__AnsiTextStyle__str__pure. The failure reproduces on main without Fenwick changes.

## 影響

Examples and downstream stdlib users cannot compile under the stricter resource checker even when only using stdio printing or ANSI styling. This blocks reliable CI validation after the owner summary changes.

## 修正方針

Investigate whether the new owner summary/drop storage-origin analysis is treating copy/static str temporaries or stdio helper return values as owning resources. Fix the checker or stdlib signatures so non-owning string temporaries are represented without owner obligations, without suppressing real leaks.

## 検証

Run trunk build, node nodesrc/tests.js -i examples -o tmp/examples-resource-owner-stdio-fixed.json -j 4 --dist web/dist, and focused stdio/ANSI tests.
