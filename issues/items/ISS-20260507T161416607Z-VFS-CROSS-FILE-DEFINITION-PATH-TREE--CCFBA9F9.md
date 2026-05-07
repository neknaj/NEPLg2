---
id: ISS-20260507T161416607Z-VFS-CROSS-FILE-DEFINITION-PATH-TREE--CCFBA9F9
title: "VFS cross-file definition path tree tests fail in tutorials CI"
area: TEST
status: open
resolved: false
priority: P1
type: test
created: 2026-05-07
updated: 2026-05-07
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
