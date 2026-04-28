---
id: ISS-20260428T223953830Z-VEC-ELEMENT-LOADS-LOSE-BACKING-STORA-E811458B
title: "Vec element loads lose backing storage initialization under RawMemoryLoadCell gate"
area: core
status: verified
resolved: true
priority: P1
type: bug
created: 2026-04-28
updated: 2026-04-29
target: "stdlib/alloc/collections/vec.nepl, nepl-core/src/resource"
---

# ISS-20260428T223953830Z-VEC-ELEMENT-LOADS-LOSE-BACKING-STORA-E811458B: Vec element loads lose backing storage initialization under RawMemoryLoadCell gate

## 概要

After Vec header reads were moved to field::get_ref, stdlib/alloc/collections/vec.nepl still fails under RawMemoryLoadCell: get(Vec<T>, i32) and get_ref(&Vec<T>, i32) load elements with load<T> from v_data + idx * size_of<T>, but Resource IR reports the backing cell as Uninit even after values were written by push or filled.

## 対象

- `stdlib/alloc/collections/vec.nepl, nepl-core/src/resource`

## 根拠

- `trunk build` 後に `node nodesrc/tests.js -i stdlib\alloc\collections\vec.nepl --no-tree -o tmp\vec-header-ref-reads-after-trunk-vec.json -j 1` を実行し、`total=39, passed=29, failed=10` になった。
- `stdlib\alloc\collections\vec.nepl::doctest#2` は `get__Vec_T_T_i32__Option_T_T__pure_i32` の `load<.T>` が `/stdlib/alloc/collections/vec.nepl:649` で D3100 になり、place は `Local("v_data") ... StorageOffset(?) ... found Uninit` だった。
- `stdlib\alloc\collections\vec.nepl::doctest#7` / `#9` は `get_ref__ref_Vec_T_T_i32__Option_T_T__pure_i32` の `load<.T>` が `/stdlib/alloc/collections/vec.nepl:672` で同じく `v_data` backing storage の `Uninit` になった。
- `ISS-20260428T222332284Z-VEC-PUSH-FREE-REJECT-INITIALIZED-HEA-736A6DA9` で header read は `field::get_ref &v` に統一済みで、`push` / `free` の header D3100 は消えている。残っているのは element cell の初期化範囲を Resource IR が復元できない問題である。
- `ISS-20260428T214527171Z-RESOURCE-IR-RAWMEMORYLOADCELL-GATE-T-F609F5AB` の外部 raw root 修正後も残っているため、function-external raw root ではなく collection-owned backing storage の initialized element range として扱う必要がある。

## 問題

After Vec header reads were moved to field::get_ref, stdlib/alloc/collections/vec.nepl still fails under RawMemoryLoadCell: get(Vec<T>, i32) and get_ref(&Vec<T>, i32) load elements with load<T> from v_data + idx * size_of<T>, but Resource IR reports the backing cell as Uninit even after values were written by push or filled.

## 影響

Vec doctests remain partially blocked (after trunk build, vec.nepl is 29/39 passed and failures include get/get_ref element loads). Self-host arenas, token streams, and diagnostics cannot rely on Vec read APIs as a regression gate while initialized element ranges are invisible to the checker.

## 修正方針

`RawMemoryLoadCell` を弱めず、Resource IR の raw address alias / external raw root 伝播を修正する。

`field::get_ref &v "data"` は、function parameter として渡された `Vec` aggregate の field reference から `MemPtr` を読み、その `MemPtr.raw + offset` を raw load address にする。この経路で `RawAddressAlias` が「raw address を作る operation」であることを明示せず、`Deref` expression 後の `MemPtr` alias も消していたため、最終的な raw load cell が parameter 由来の external backing storage と結び直せなかった。

修正では `RawAddressAlias` を force raw-address mode で扱い、`Deref` が `MemPtr` / `RegionToken` を返す場合だけ raw alias を維持する。また、canonical address だけでなく同一 alias group 内の address が external raw storage に重なるかを見て `RawMemoryLoadCell` の外部 parameter load を許可する。compiler-owned allocation については owned raw root を優先し、load-before-store diagnostic を維持する。

関連計画: [NEPLg2 静的検査の複雑化解消計画 Stage 4](../../doc/neplg2/static_check_complexity_reduction_plan.md#stage-4-resource-check-%E3%81%B8%E3%81%AE%E7%A7%BB%E8%A1%8C)

## 検証

- `cargo test -p nepl-core --test resource_ir resource_ir_cell_check -- --nocapture`: 26 passed
- `trunk build`: pass
- `node nodesrc\tests.js -i stdlib\alloc\collections\vec.nepl --no-tree -o tmp\agent1-vec-external-backing-main-after-pick.json -j 1`: total=39, passed=39, failed=0
- `cargo check -p nepl-core --tests`: pass
- `rustfmt --check nepl-core\src\resource\cell_state.rs nepl-core\src\resource\initialized.rs nepl-core\src\resource\initialized_alias.rs nepl-core\src\resource\initialized_raw_memory.rs nepl-core\tests\resource_ir.rs`: pass
- `node nodesrc\test_resource_checker_responsibility.js`: pass
- `node nodesrc\issues.js check`: pass

## 対応結果

`Vec` element load の D3100 は、stdlib 側の header read ではなく core Resource IR の external aggregate raw address alias 伝播不足が根本原因だった。`Vec` parameter の backing storage は caller-owned な external storage として扱う一方、`RawMemoryLoadCell` gate 自体は維持したため、compiler-owned raw allocation の未初期化 load を隠す修正にはしていない。
