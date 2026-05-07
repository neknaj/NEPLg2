---
id: ISS-20260507T161416607Z-VFS-CROSS-FILE-DEFINITION-PATH-TREE--CCFBA9F9
title: "VFS cross-file definition path tree tests fail in tutorials CI"
area: TEST
status: fixed
resolved: true
priority: P1
type: test
created: 2026-05-07
updated: 2026-05-08
target: "tests/compiler/tree/16_semantics_vfs_cross_file.js, tests/compiler/tree/17_name_resolution_vfs_cross_file.js, nepl-web analysis API"
---

# ISS-20260507T161416607Z-VFS-CROSS-FILE-DEFINITION-PATH-TREE--CCFBA9F9: VFS cross-file definition path tree tests fail in tutorials CI

## 概要

GitHub Actions tutorials-test includes tree tests and the latest failure set shows semantics_vfs_cross_file_definition_path and name_resolution_vfs_cross_file_definition_path failing because resolved definitions for core/math::add no longer point to /stdlib/core/math.nepl.

## 対象

- `tests/compiler/tree/16_semantics_vfs_cross_file.js, tests/compiler/tree/17_name_resolution_vfs_cross_file.js, nepl-web analysis API`

## 根拠

- `gh run view 25507054306 --job 74856320520 --log` で、直前 completed old run の `tutorials-test` が `44 total / 40 passed / 4 failed` と出力していることを確認した。
- 同 log では `tests/compiler/tree/semantics_vfs_cross_file_definition_path` が `resolved definition should point to stdlib/core/math.nepl` の assertion で失敗している。
- 同 log では `tests/compiler/tree/name_resolution_vfs_cross_file_definition_path` が `resolved_def should point to stdlib/core/math.nepl` の assertion で失敗している。
- latest main run `25507326678` でも `tutorials-test` job は failure になっているため、completed latest run の log / artifact で再確認する必要がある。

## 問題

GitHub Actions tutorials-test includes tree tests and the latest failure set shows semantics_vfs_cross_file_definition_path and name_resolution_vfs_cross_file_definition_path failing because resolved definitions for core/math::add no longer point to /stdlib/core/math.nepl.

## 影響

Editor/navigation regression tests are failing inside the tutorial CI gate. If the compiler or web analysis API changed the canonical stdlib path, the tests are stale; if not, go-to-definition and hover references can point to the wrong source path.

## 修正方針

Root-cause whether the current canonical stdlib path contract changed or the analysis API is dropping the original source path. Update the tests only if the new path is intentionally canonical; otherwise fix the analysis result to preserve the stdlib/core/math.nepl definition path.

## 検証

Use gh logs or artifacts for the latest tutorials-test run and focused tree test execution to confirm both VFS cross-file definition path tests pass.

## 2026-05-08 Agent 2 修正

根本原因は 2 つあった。

- `stdlib/core/math` が facade 分割され、`core/math` 経由で見える `add` の実定義 path は root `stdlib/core/math.nepl` ではなく `stdlib/core/math/i32/arith.nepl` などの実装 module になっていた。旧 tree test は facade root を定義元として固定していたため、module split 後の正しい source path 契約に追従できていなかった。
- `nepl-web` の semantics API は型検査後の HIR call が持つ selected callee `DefId` を使わず、AST ベース name-resolution trace の先頭候補を `resolved_definition` として出していた。そのため `add 1 2` の `inferred_type` は `i32` なのに、`resolved_definition` は overload 候補先頭の `int128/i128.add` を指す不整合が起きていた。

修正内容:

- `SemanticExprTrace` に selected callee `DefId` を保持し、token が call callee に対応する場合は `token_resolution` / `token_hints` の `resolved_definition` を型検査後に選択された定義へ補正した。
- `NameDefTrace` は compiler `DefId` を保持し、HIR の selected callee と analysis trace の定義を span-derived `DefId` で対応付けるようにした。
- semantics VFS tree test は `add` の inferred type と selected definition が `stdlib/core/math/i32/arith.nepl` に一致することを固定した。
- name-resolution VFS tree test は型情報なしの段階であるため、`resolved_def` は stdlib math implementation file を指すこと、candidate list に `stdlib/core/math/i32/arith.nepl` が含まれることを固定した。

検証:

- `trunk build`: passed
- `NEPL_DIST=web/dist node tests/compiler/tree/run.js`: total=20, passed=20
- `node nodesrc/tests.js -i tutorials/getting_started --with-tree -o tmp/vfs-tree-tutorials-final.json -j 4 --dist web/dist`: total=44, passed=44
- inline API inspection: `add` token の `token_resolution.resolved_definition.span.file_path` と `token_hints.resolved_definition.span.file_path` が `/stdlib/core/math/i32/arith.nepl`、`token_hints.inferred_type` が `i32`
