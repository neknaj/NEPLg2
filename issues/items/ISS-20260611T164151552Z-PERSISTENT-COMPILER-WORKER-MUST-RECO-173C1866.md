---
id: ISS-20260611T164151552Z-PERSISTENT-COMPILER-WORKER-MUST-RECO-173C1866
title: "Persistent compiler worker must recover from asset initialization failure"
area: tools
status: open
resolved: false
priority: P1
type: bug
created: 2026-06-11
updated: 2026-06-11
target: "web/src/runtime/worker.ts; web/src/terminal/shell.ts"
---

# ISS-20260611T164151552Z-PERSISTENT-COMPILER-WORKER-MUST-RECO-173C1866: Persistent compiler worker must recover from asset initialization failure

## 概要

compilerInitPromise is cached even if dynamic import or wasm-bindgen initialization rejects. Compile errors are recoverable, so the persistent worker can be kept with a permanently rejected initialization promise.

## 対象

- `web/src/runtime/worker.ts; web/src/terminal/shell.ts`

## 根拠

- 未記入

## 問題

compilerInitPromise is cached even if dynamic import or wasm-bindgen initialization rejects. Compile errors are recoverable, so the persistent worker can be kept with a permanently rejected initialization promise.

## 影響

Transient asset load, cache, or service-worker failures can leave Playground compile broken until page reload.

## 修正方針

Reset compiler init/session state on initialization rejection or classify asset initialization as unrecoverable so the shell recreates the compiler worker.

## 検証

Worker test where first compiler import/init fails and second attempt succeeds without page reload.
