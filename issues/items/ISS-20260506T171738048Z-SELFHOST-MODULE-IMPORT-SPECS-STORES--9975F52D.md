---
id: ISS-20260506T171738048Z-SELFHOST-MODULE-IMPORT-SPECS-STORES--9975F52D
title: "selfhost module_import_specs stores owned str payloads in Vec under strict ResourceIR"
area: selfhost
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-05-06
updated: 2026-05-07
target: "stdlib/neplg2/core/module/import_spec.nepl, stdlib/neplg2/core/module/graph.nepl, stdlib/alloc/collections/vec.nepl"
---

# ISS-20260506T171738048Z-SELFHOST-MODULE-IMPORT-SPECS-STORES--9975F52D: selfhost module_import_specs stores owned str payloads in Vec under strict ResourceIR

## 概要

After lexer stale Vec construction and direct import-spec fixture leaks are fixed, tests/stdlib/neplg2_import_spec.n.md reaches selfhost_import_specs_loop and Resource IR reports use_after_move/maybe_leak for Vec<SelfhostImportSpec>. SelfhostImportSpec contains owned str path/alias values produced by str_slice, and pushing those owner payloads into raw Vec storage relies on generic Vec element drop/transfer support that is still not represented.

## 対象

- `stdlib/neplg2/core/module/import_spec.nepl, stdlib/neplg2/core/module/graph.nepl, stdlib/alloc/collections/vec.nepl`

## 根拠

- `ISS-20260506T155806325Z-SELFHOST-LEXER-AND-IMPORT-SPEC-FIXTU-68BADCC8` の直接修正後、`tests/stdlib/neplg2_import_spec.n.md` が `selfhost_import_specs_loop` まで進み、`resource.owner.use_after_move` と `resource.owner.maybe_leak` を報告した。
- 問題箇所は単なる fixture leak ではなく、`SelfhostImportSpec` が `str` owner field を保持したまま `Vec<SelfhostImportSpec>` へ push される構造にある。
- 現在の generic `Vec` は raw storage 上の要素 Drop/transfer contract を ResourceIR に十分表現できていないため、このまま owner aggregate を入れると memory safety の検査を弱める方向になる。

## 問題

After lexer stale Vec construction and direct import-spec fixture leaks are fixed, tests/stdlib/neplg2_import_spec.n.md reaches selfhost_import_specs_loop and Resource IR reports use_after_move/maybe_leak for Vec<SelfhostImportSpec>. SelfhostImportSpec contains owned str path/alias values produced by str_slice, and pushing those owner payloads into raw Vec storage relies on generic Vec element drop/transfer support that is still not represented.

## 影響

The selfhost module graph cannot safely use a Vec of import specs with owned string payloads under mandatory memory-safety checking. Keeping this shape would either hide leaks in Vec element storage or force the checker to weaken owner diagnostics.

## 修正方針

Redesign module import collection so the Vec element is Copy-only or has explicit element ownership semantics: store item indexes/ranges and resolve path/alias against the AST while it is alive, or implement typed Vec element Drop/transfer support before storing owned str payloads. Do not keep SelfhostImportSpec as a Copy aggregate containing owned str fields.

## 対応

- `SelfhostImportSpec` から `path <str>` / `alias <str>` を削除し、`item_index`、`path_start` / `path_end`、`alias_start` / `alias_end`、`is_wildcard` を持つ Copy-only range spec に再設計した。
- `selfhost_import_spec_path` / `selfhost_import_spec_alias` を追加し、path/alias が必要な境界でだけ元 lexeme から `str_slice` するようにした。
- `selfhost_module_import_specs` は `Vec<SelfhostImportSpec>` に owner payload を入れず、AST item index と範囲だけを保存するようにした。
- `graph.nepl` は import traversal 中に `SelfhostModuleAst` を保持し、range-only spec の `item_index` から元 item lexeme を参照して path を切り出すようにした。
- `stdlib_map.nepl` の import-spec resolver は元 lexeme を受け取り、range-only spec から path を切り出して通常の path resolver に渡すようにした。
- `nodesrc/test_selfhost_string_helpers_boundary.js` に、`SelfhostImportSpec` が owned `str` field を持たないこと、path/alias は lexeme slice で取り出すこと、module graph が AST を保持して traversal することの source-policy を追加した。

## 検証

- `node nodesrc/test_selfhost_string_helpers_boundary.js`: passed
- `node nodesrc/tests.js -i stdlib/neplg2/core/module/import_spec.nepl --no-tree -o tmp/selfhost-import-spec-module-after-trunk.json -j 1`: total=1, passed=1
- 一時 smoke `tmp/selfhost_import_spec_ast_smoke.n.md`: `selfhost_module_import_specs` が手組み AST から range-only spec を `Vec` に入れ、元 item lexeme から path/alias を切り出せることを確認。total=1, passed=1。検証後に tmp file は削除。
- `trunk build`: passed
- `origin/main` `824ada60` 取り込み後に `trunk build`: passed
- `node nodesrc/tests.js -i stdlib/neplg2/core/module/import_spec.nepl -i tests/stdlib/neplg2_import_spec.n.md --no-tree -o tmp/selfhost-import-spec-ranges-after-rebase-trunk.json -j 1`: total=4, passed=4
- `node nodesrc/run_source_policy_regressions.js --warn-only`: selfhost import-spec source-policy は passed。remote main で `lower_raw_address.rs` blocker は解消済み。既知別件 `ISS-20260506T180609091Z-RESOURCE-INITIALIZED-ALIAS-MODULE-EX-BA05D57A` の `initialized_alias.rs has 624 lines; responsibility split limit is 520` warning は継続。
- `node nodesrc/tests.js -i stdlib/neplg2/core/module/stdlib_map.nepl --no-tree -o tmp/selfhost-stdlib-map-baseline-head.json -j 1`: 今回差分を stash した状態でも timeout。`ISS-20260506T175807290Z-SELFHOST-STDLIB-MAP-AND-MODULE-GRAPH-981662BF` として分離。
