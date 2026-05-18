---
id: ISS-20260518T004559104Z-SELFHOST-MODULE-GRAPH-KEEPS-OBSOLETE-8DDB7A8E
title: "selfhost module graph keeps obsolete AST import traversal path"
area: selfhost
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-05-18
updated: 2026-05-18
target: "stdlib/neplg2/core/module/graph.nepl, nodesrc/test_selfhost_string_helpers_boundary.js"
---

# ISS-20260518T004559104Z-SELFHOST-MODULE-GRAPH-KEEPS-OBSOLETE-8DDB7A8E: selfhost module graph keeps obsolete AST import traversal path

## 概要

After the module graph default path moved to source-scanned SelfhostImportRecord values, graph.nepl still imports module_ast and keeps public AST-based import traversal helpers. The source policy also requires the obsolete AST traversal, so future cleanup is blocked and static-check input can still widen.

## 対象

- `stdlib/neplg2/core/module/graph.nepl, nodesrc/test_selfhost_string_helpers_boundary.js`

## 根拠

- `stdlib/neplg2/core/module/graph.nepl` は前段で `selfhost_scan_module_imports_with_file_id` を使う通常経路へ移った後も、`neplg2/core/syntax/ast/module_ast` を import し、`selfhost_module_graph_visit_imports` / `selfhost_module_graph_extract_imports` など AST + `SelfhostImportSpec` 経由の traversal helper を保持していた。
- `nodesrc/test_selfhost_string_helpers_boundary.js` はその旧経路を要求しており、source-scanned import record への移行後も AST traversal を消せない policy になっていた。

## 問題

After the module graph default path moved to source-scanned SelfhostImportRecord values, graph.nepl still imports module_ast and keeps public AST-based import traversal helpers. The source policy also requires the obsolete AST traversal, so future cleanup is blocked and static-check input can still widen.

## 影響

Self-host graph compilation can continue to pull AST/import-spec traversal code into modules that only need import edges, and the policy regression encourages retaining technical debt instead of the lightweight proof path.

## 修正方針

Remove the AST-based graph traversal helpers and the module_ast import from graph.nepl. Update the source policy to require the source-scanned import path and to reject reintroduction of SelfhostModuleAst traversal in module graph.

## 検証

Run the selfhost string helper boundary policy and focused neplg2 module graph / stdlib_map doctests with --assert-io.

## 2026-05-18 修正

`graph.nepl` から旧 AST / `SelfhostImportSpec` traversal API を削除し、通常 graph build の入力を `SelfhostImportRecord` に一本化した。`module_ast` / `import_spec` import も graph module から外し、parser AST を必要とする処理は loader/parser 側に閉じる。

`nodesrc/test_selfhost_string_helpers_boundary.js` は旧 AST traversal を要求するのではなく、次を検査する policy に更新した。

- `graph.nepl` が `module/import_scan` を import する。
- `selfhost_scan_module_imports_with_file_id file.source file.file_id` を使う。
- `selfhost_module_graph_visit_import_records` が `Vec<SelfhostImportRecord>` を traversal input にする。
- `SelfhostModuleAst` / `SelfhostImportSpec` / `selfhost_module_graph_visit_imports` / `selfhost_module_graph_extract_imports` / `selfhost_module_graph_import_item` を graph module に戻さない。

この修正は互換 API を残さず、前段で導入した lightweight import proof path を静的検査・source policy の両方で唯一の graph traversal 境界として固定する。

検証:

- `node nodesrc\test_selfhost_string_helpers_boundary.js`: passed
- `node nodesrc\tests.js -i stdlib\neplg2\core\module\graph.nepl -i tests\stdlib\neplg2_module_graph.n.md -i tests\stdlib\neplg2_stdlib_map.n.md --no-tree -o tmp\agent1-selfhost-graph-no-ast-path.json -j 1 --dist web\dist --assert-io`: total=7, passed=7
- `node nodesrc\issues.js check --dir issues`: passed
