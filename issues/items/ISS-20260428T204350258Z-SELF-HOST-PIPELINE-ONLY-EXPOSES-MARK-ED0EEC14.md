---
id: ISS-20260428T204350258Z-SELF-HOST-PIPELINE-ONLY-EXPOSES-MARK-ED0EEC14
title: "self-host pipeline only exposes marker API"
area: selfhost
status: fixed
resolved: true
priority: P2
type: architecture
created: 2026-04-28
updated: 2026-04-28
target: stdlib/neplg2/core/pipeline.nepl
---

# ISS-20260428T204350258Z-SELF-HOST-PIPELINE-ONLY-EXPOSES-MARK-ED0EEC14: self-host pipeline only exposes marker API

## 概要

core/pipeline.nepl still exposes only selfhost_pipeline_stage0 and does not define a compile request or a root module loading boundary. Even though VFS loader and core compile options exist, a future driver has no single core API to pass root path plus options into the pipeline.

## 対象

- `stdlib/neplg2/core/pipeline.nepl`

## 根拠

- `core/module/loader.nepl` は VFS から `SelfhostLoadedModule` を返せる。
- `core/options.nepl` は `SelfhostCompileOptions` を持つ。
- 修正前の `core/pipeline.nepl` は `selfhost_pipeline_stage0` だけで、root path と options を束ねて loader に渡す pipeline 入口がなかった。

## 問題

core/pipeline.nepl still exposes only selfhost_pipeline_stage0 and does not define a compile request or a root module loading boundary. Even though VFS loader and core compile options exist, a future driver has no single core API to pass root path plus options into the pipeline.

## 影響

Driver/check/lowering work would wire loader, options, and later stages ad hoc. This keeps RV-STDLIB-008 blocked and risks breaking the core/CLI separation because CLI code could start calling lower-level loader internals directly.

## 修正方針

`SelfhostCompileRequest` を追加し、root module の logical path と `SelfhostCompileOptions` を 1 value にまとめました。

さらに `SelfhostPipelineLoadedRoot` と `selfhost_pipeline_load_root` を追加し、VFS を borrow して root module を読み、loaded module と compile options を一緒に返す pipeline 境界を作りました。返る値は loaded module の AST buffer を所有するため、`selfhost_pipeline_loaded_root_free` で解放する API も追加しています。

## 検証

- `node nodesrc\tests.js -i stdlib\neplg2\core\pipeline.nepl --no-tree -o tmp\selfhost-pipeline-root-load.json -j 1`: total=1 passed=1
