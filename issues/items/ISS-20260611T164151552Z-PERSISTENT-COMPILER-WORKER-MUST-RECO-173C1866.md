---
id: ISS-20260611T164151552Z-PERSISTENT-COMPILER-WORKER-MUST-RECO-173C1866
title: "Persistent compiler worker must recover from asset initialization failure"
area: tools
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-06-11
updated: 2026-06-12
target: "web/src/runtime/worker.ts; web/src/terminal/shell.ts"
---

# ISS-20260611T164151552Z-PERSISTENT-COMPILER-WORKER-MUST-RECO-173C1866: Persistent compiler worker must recover from asset initialization failure

## 概要

compilerInitPromise is cached even if dynamic import or wasm-bindgen initialization rejects. Compile errors are recoverable, so the persistent worker can be kept with a permanently rejected initialization promise.

## 対象

- `web/src/runtime/worker.ts; web/src/terminal/shell.ts`

## 根拠

- Persistent compiler worker は同一 asset URL の compile request 間で再利用される。
- `import()` または wasm-bindgen `default()` の初期化失敗はユーザーソース由来の compile error ではなく、compiler worker/session lifecycle の失敗である。
- rejected `compilerInitPromise` を保持すると、次回 request が同じ rejected promise を再利用して page reload まで復旧できない。

## 問題

compilerInitPromise is cached even if dynamic import or wasm-bindgen initialization rejects. Compile errors are recoverable, so the persistent worker can be kept with a permanently rejected initialization promise.

## 影響

Transient asset load, cache, or service-worker failures can leave Playground compile broken until page reload.

## 修正方針

Reset compiler init/session state on initialization rejection or classify asset initialization as unrecoverable so the shell recreates the compiler worker.

## 実装

- runtime worker に `CompilerInitializationError` と `resetCompilerInitializationState` を追加し、`import()` / wasm-bindgen init reject 後に init promise、session、session checked flag を消す。
- compiler init failure は `phase: 'compiler-init'`, `recoverable: false` として shell へ返し、shell が persistent compiler worker を破棄して次回 compile で再作成する。
- `CompilerSession` 生成失敗も init lifecycle failure として扱い、checked state を汚さない。
- analysis worker 側の compiler init promise も reject 後に reset し、semantic analysis worker が rejected promise を保持し続けないようにした。
- recoverable な user compile error は `phase: 'compile'`, `recoverable: true` のままにし、persistent compiler worker を維持する。

## 検証

Worker test where first compiler import/init fails and second attempt succeeds without page reload.

- `npm --prefix web run build:ts`
- `node nodesrc/test_playground_compiler_session_policy.js`
- `node nodesrc/test_playground_worker_init_recovery.js`
- `node nodesrc/playground_shell_worker_test_runner.js`
