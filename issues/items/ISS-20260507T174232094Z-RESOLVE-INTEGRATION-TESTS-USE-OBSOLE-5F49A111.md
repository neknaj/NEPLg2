---
id: ISS-20260507T174232094Z-RESOLVE-INTEGRATION-TESTS-USE-OBSOLE-5F49A111
title: "resolve integration tests use obsolete qualified import target API"
area: core
status: verified
resolved: true
priority: P1
type: test
created: 2026-05-07
updated: 2026-05-07
target: "nepl-core/tests/resolve.rs, nepl-core/src/resolve.rs"
---

# ISS-20260507T174232094Z-RESOLVE-INTEGRATION-TESTS-USE-OBSOLE-5F49A111: resolve integration tests use obsolete qualified import target API

## 概要

After qualified alias imports were changed to visibility-aware lookup rules, resolve integration tests still called the removed file-set helper qualified_targets_for_alias. Cargo integration tests no longer compile.

## 対象

- `nepl-core/tests/resolve.rs, nepl-core/src/resolve.rs`

## 根拠

- `ImportResolution` は visibility-aware な `qualified_lookup_names(source_file, alias, member)` に移行済みである。
- `nepl-core/tests/resolve.rs` の integration test は旧 file-set API `qualified_targets_for_alias` を呼び続けていた。
- `cargo test -p nepl-core source_map::tests` の integration test compile 段階で、旧 API 参照が unresolved method として検出された。

## 問題

After qualified alias imports were changed to visibility-aware lookup rules, resolve integration tests still called the removed file-set helper qualified_targets_for_alias. Cargo integration tests no longer compile.

## 影響

nepl-core integration test builds fail before exercising ResourceIR or SourceCapabilities changes, hiding real regressions behind stale test API usage.

## 修正方針

Update resolve integration tests to assert the new qualified_lookup_names contract directly, including public facade reexports and direct alias targets.

## 検証

Run cargo test -p nepl-core --test resolve import_resolution -- --nocapture and cargo test -p nepl-core --lib source_map::tests -- --nocapture.

## 対応結果

resolve integration test を旧 target file set API ではなく、現行の `qualified_lookup_names` 契約を直接検査する形へ更新した。

- default alias import は `dep::allowed` が `(dep_file, "allowed")` に解決されることを確認する。
- merge facade import は direct facade target と merge 先 target の両方が qualified lookup candidate になることを確認する。
- facade / dep の file id 順序に依存せず、lookup result の内容を検査する。

compiler 側 API は戻していない。削除済み helper を復活させると visibility rule の意味が test 側に伝わらず、今回の qualified import 再設計を file set API へ戻す技術的負債になるためである。

## 検証結果

- `cargo test -p nepl-core --test resolve import_resolution -- --nocapture`: 3 passed
- `cargo test -p nepl-core --lib source_map::tests -- --nocapture`: 2 passed
