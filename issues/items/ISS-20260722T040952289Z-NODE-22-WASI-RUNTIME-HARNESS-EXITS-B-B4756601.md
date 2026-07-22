---
id: ISS-20260722T040952289Z-NODE-22-WASI-RUNTIME-HARNESS-EXITS-B-B4756601
title: "Node 22 WASI runtime harness exits before assertions"
area: tooling/test
status: open
resolved: false
priority: P1
type: bug
created: 2026-07-22
updated: 2026-07-22
target: nodesrc/run_test.js
---

# ISS-20260722T040952289Z-NODE-22-WASI-RUNTIME-HARNESS-EXITS-B-B4756601: Node 22 WASI runtime harness exits before assertions

## 概要

On Node v22.23.1, GUI runSingle runtime scripts terminate the host process after the first WASI guest exit, before result assertions and JSON reporting.

## 対象

- `nodesrc/run_test.js`

## 根拠

- Node v22.23.1で既存budget exhaustion runtimeとF5oay runtimeの双方がWASI警告後にJSONを出さず終了する。
- keepalive handle、WASI `returnOnExit`、import tableの`proc_exit`差し替えではJS assertionへ復帰しなかったため、これらの試行差分は残していない。

## 問題

On Node v22.23.1, GUI runSingle runtime scripts terminate the host process after the first WASI guest exit, before result assertions and JSON reporting.

## 影響

A zero process exit can be mistaken for a passed runtime contract even though JavaScript assertions after runSingle did not execute.

## 修正方針

Isolate WASI execution in a child process or worker and return the guest result to the assertion process; do not rely on WASI returnOnExit for direct in-process execution.

## 検証

Known existing and F5oay runtime scripts emit their JSON summaries and deliberately wrong assertions fail nonzero on Node 22.
