---
id: ISS-20260611T164151841Z-NEPLG2-TERMINAL-COMMAND-NEEDS-REAL-S-C21C6BD7
title: "neplg2 terminal command needs real subcommand and argv parsing"
area: tools
status: open
resolved: false
priority: P1
type: bug
created: 2026-06-11
updated: 2026-06-11
target: "web/src/terminal/shell.ts; web/src/main.ts; nodesrc/playground_shell_worker_test_runner.js"
---

# ISS-20260611T164151841Z-NEPLG2-TERMINAL-COMMAND-NEEDS-REAL-S-C21C6BD7: neplg2 terminal command needs real subcommand and argv parsing

## 概要

The shell detects run/build with args.includes and always passes empty runArgs. Filenames or arguments named run/build can alter command behavior, and Playground cannot run programs that depend on argv.

## 対象

- `web/src/terminal/shell.ts; web/src/main.ts; nodesrc/playground_shell_worker_test_runner.js`

## 根拠

- 未記入

## 問題

The shell detects run/build with args.includes and always passes empty runArgs. Filenames or arguments named run/build can alter command behavior, and Playground cannot run programs that depend on argv.

## 影響

Compile/run UX is ambiguous, paths with special tokens are unreliable, and WASI argv examples cannot be tested in Playground.

## 修正方針

Parse a small command grammar: first positional token is subcommand, reject both run and build, support -- for program argv, and quote UI-generated paths safely.

## 検証

Tests for /run.nepl as a file, paths with spaces, neplg2 run -i /x.nepl -- a b, and WASI argv visibility.
