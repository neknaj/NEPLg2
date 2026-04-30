---
id: ISS-20260430T154405890Z-RESOURCE-IR-TUPLE-OWNER-PROJECTIONS--CCF76754
title: "Resource IR tuple owner projections leak after aggregate field extraction"
area: core
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-04-30
updated: 2026-05-01
target: "nepl-core/src/resource, tests/compiler/overload.n.md"
---

# ISS-20260430T154405890Z-RESOURCE-IR-TUPLE-OWNER-PROJECTIONS--CCF76754: Resource IR tuple owner projections leak after aggregate field extraction

## 概要

tests/compiler/overload.n.md::doctest#10 still fails with resource.owner.leak for `parts` tuple field owner projections after extracting Vec fields from an aggregate result.

## 対象

- `nepl-core/src/resource, tests/compiler/overload.n.md`

## 根拠

- `node nodesrc/tests.js -i tests/compiler/overload.n.md --no-tree -o tmp/overload-owner-pipeline-agent1.json -j 1 --dist web/dist` で `doctest#10` が `resource.owner.leak` を返した。
- diagnostic は `Local("parts")` の tuple field `0` / `1` 以下に残る `Vec` owner projection が `StorageId(0)` / `StorageId(1)` を保持したまま関数終了することを示している。

## 問題

tests/compiler/overload.n.md::doctest#10 still fails with resource.owner.leak for `parts` tuple field owner projections after extracting Vec fields from an aggregate result.

## 影響

Compiler fixture remains failing under strict Resource IR owner checking. The leak must be solved by precise tuple/field owner transfer, not by weakening owner diagnostics.

## 修正方針

Review aggregate field extraction and tuple projection owner transfer so moving fields out of tuple-like results retires the original tuple field obligations exactly once.

## 解決

- `get parts 0` / `get parts 1` の selector が `LiteralI32` の場合も Resource IR lowering が tuple field projection として直接扱うようにした。
- `ResourceOp::Read` が non-Copy source の tracked owner state を持つ場合、raw alias copy ではなく owner transfer として処理するようにした。
- non-owning raw-address view と copy raw pointer read は従来通り alias / marker copy として扱い、所有義務を誤って移さないように分離した。
- tuple field から取り出した nested owner を抽出先で dealloc でき、元 tuple 側に leak が残らない回帰テストを追加した。

## 検証

- `cargo fmt --check`: passed
- `cargo check -p nepl-core`: passed
- `cargo test -p nepl-core resource_ir_lowering_projects_tuple_get_numeric_selector`: passed
- `cargo test -p nepl-core resource_ir_owner_check_read_moves_tuple_field_owner_projection`: passed
- `trunk build`: passed
- `node nodesrc/run_doctest.js -i tests/compiler/overload.n.md -n 10 --dist web/dist`: passed
- `node nodesrc/tests.js -i tests/compiler/overload.n.md --no-tree -o tmp/overload-tuple-owner-agent1.json -j 1 --dist web/dist`: total=45, passed=44, failed=1。残る failure は `ISS-20260430T154415527Z-TYPED-BLOCK-CONTEXT-LOSES-VEC-LEN-RE-46F57311`。
