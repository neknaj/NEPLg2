---
id: ISS-20260505T091240393Z-MEMORY-SAFETY-DOCTESTS-STILL-CALL-RA-7A3E26CD
title: "memory_safety doctests still call raw memory from pure main"
area: TEST
status: fixed
resolved: true
priority: P1
type: test
created: 2026-05-05
updated: 2026-05-05
target: tests/stdlib/memory_safety.n.md
---

# ISS-20260505T091240393Z-MEMORY-SAFETY-DOCTESTS-STILL-CALL-RA-7A3E26CD: memory_safety doctests still call raw memory from pure main

## 概要

After UnsafeMemoryInPureFunction became an enforced Resource IR diagnostic, memory_safety positive doctests that intentionally exercise raw memory wrappers still declare main as pure and now fail with resource.raw.unsafe_memory_boundary before reaching the memory safety behavior under test.

## 対象

- `tests/stdlib/memory_safety.n.md`

## 根拠

- `resource.raw.unsafe_memory_boundary` の有効化後、`tests/stdlib/memory_safety.n.md` の positive doctest 複数が raw memory wrapper を呼び出しているにもかかわらず `fn main <()->i32>` のままだった。
- これらは pure surface の検査を回避したいテストではなく、alloc/load/store/fill/RegionToken projection など raw-memory-backed API の挙動を確認するテストである。
- compile_fail の型境界テストは raw operation 実行前の型検査を確認する目的があるため pure のまま残す必要がある。

## 問題

After UnsafeMemoryInPureFunction became an enforced Resource IR diagnostic, memory_safety positive doctests that intentionally exercise raw memory wrappers still declare main as pure and now fail with resource.raw.unsafe_memory_boundary before reaching the memory safety behavior under test.

## 影響

CI can report the static boundary enforcement as a test regression, and the memory_safety suite no longer separates pure-surface enforcement from raw memory API behavior.

## 修正方針

Mark only the positive memory_safety doctests that intentionally perform raw memory operations as impure, keeping compile_fail type-boundary tests pure so their original diagnostics remain visible.

## 対応

- `tests/stdlib/memory_safety.n.md` の positive doctest のうち、raw memory wrapper を実行する `main` だけを `fn main <()*>i32>` に変更した。
- `compile_fail` の型境界テストは pure signature を維持し、型検査の失敗理由が effect 境界の診断で隠れないようにした。
- focused run で残った `doctest#8` の `resource.cell.uninit` は unsafe memory boundary ではなく、`MemPtr<i32>` projection と checked `fill_i32` の initialized range/provenance が caller 側へ十分に伝わらない core Resource IR 残件であるため、`ISS-20260427T152958303Z-MEMPTR-AND-REGIONTOKEN-LACK-COMPILER-0BC8ECDF` に追記して分離した。

## 検証

- 変更前: `node nodesrc/tests.js -i tests/stdlib/memory_safety.n.md --no-tree -o tmp/memory-safety-agent1-current.json -j 1 --dist web/dist` は 12 total / 5 passed / 7 failed。失敗の主因は `resource.raw.unsafe_memory_boundary`。
- 変更後: `node nodesrc/tests.js -i tests/stdlib/memory_safety.n.md --no-tree -o tmp/memory-safety-agent1-after-impure.json -j 1 --dist web/dist` は 12 total / 11 passed / 1 failed。unsafe boundary 起因の失敗は解消済み。
- `node nodesrc/run_doctest.js -i tests/stdlib/memory_safety.n.md -n 1 --dist web/dist`: pass
- `node nodesrc/run_doctest.js -i tests/stdlib/memory_safety.n.md -n 2 --dist web/dist`: pass
- `node nodesrc/run_doctest.js -i tests/stdlib/memory_safety.n.md -n 3 --dist web/dist`: pass
- `node nodesrc/run_doctest.js -i tests/stdlib/memory_safety.n.md -n 4 --dist web/dist`: pass
- `node nodesrc/run_doctest.js -i tests/stdlib/memory_safety.n.md -n 6 --dist web/dist`: pass
- `node nodesrc/run_doctest.js -i tests/stdlib/memory_safety.n.md -n 7 --dist web/dist`: pass
- 残る 1 件は `doctest#8` の `resource.cell.uninit` であり、この issue ではなく MemPtr/RegionToken provenance と initialized cell summary の未完了事項として追跡する。
