---
id: ISS-20260515T223330574Z-VEC-STORAGE-TAG-AND-REGIONTOKEN-OWNE-DDDAD134
title: "Vec storage tag and RegionToken owner are split, so Empty cleanup cannot be proven by type"
area: stdlib
status: open
resolved: false
priority: P1
type: architecture
created: 2026-05-15
updated: 2026-05-16
target: "stdlib/alloc/collections/vec/types.nepl, stdlib/alloc/collections/vec/storage/cleanup.nepl, nepl-core/src/resource"
---

# ISS-20260515T223330574Z-VEC-STORAGE-TAG-AND-REGIONTOKEN-OWNE-DDDAD134: Vec storage tag and RegionToken owner are split, so Empty cleanup cannot be proven by type

## 概要

Vec stores VecStorageState and RegionToken as independent fields. A cleanup helper that no-ops on VecStorageState::Empty is not type-safe because the signature also admits (Empty, owned RegionToken). Resource IR correctly reports owner leaks when Empty does not consume the token.

## 対象

- `stdlib/alloc/collections/vec/types.nepl, stdlib/alloc/collections/vec/storage/cleanup.nepl, nepl-core/src/resource`

## 根拠

- `vec_free_storage<T>(VecStorageState, RegionToken<T>)` を `Empty` no-op / `Owned` dealloc にすると、`Empty` branch で `RegionToken.raw` owner obligation が残り、Resource IR が `resource.owner.leak` / `resource.owner.maybe_leak` を報告する。
- この診断は false positive ではない。helper signature が storage tag と owner token を独立に受け取るため、source type system は `Empty` と allocated `RegionToken` の組み合わせを排除できない。
- `doc/neplg2/static_check_complexity_reduction_plan.md` の Stage 6 は、`MemPtr` を non-owning pointer、storage owner を owner token、initialized cell を Resource IR state に分離する方針である。Vec の現行 split field はその過渡形であり、owner state の相関を型で表せていない。

## 問題

Vec stores VecStorageState and RegionToken as independent fields. A cleanup helper that no-ops on VecStorageState::Empty is not type-safe because the signature also admits (Empty, owned RegionToken). Resource IR correctly reports owner leaks when Empty does not consume the token.

## 影響

The current representation cannot prove the intended Empty/Owned storage invariant from source alone. Keeping Empty no-op would weaken static correctness or require stdlib-specific assumptions. The final design must bind the owner token to the Owned storage variant, or otherwise provide a compiler-checked invariant mechanism from source constructors.

## 修正方針

Redesign Vec storage so the allocation owner is structurally tied to the owned state, e.g. VecStorage<T>::Empty | VecStorage<T>::Owned(RegionToken<T>), and update observers/mutations with compiler-supported borrowed enum payload access or an equivalent checked invariant representation. Until that lands, cleanup must consume the RegionToken through the existing owner destructor to avoid leaks.

## 2026-05-16 Agent 1 prerequisite update

調査の結果、`VecStorage<T>::Owned(RegionToken<T>)` へ移行するには、`&Vec` observer が `&VecStorage<T>` を match し、`Owned` payload の `RegionToken<T>` を移動せず `&RegionToken<T>` として束縛できる必要があることを確認した。これがないと `data_mem_ptr`、`get`、`replace`、sort などの borrowed API が owner payload を move/copy する設計になり、storage tag と owner token を結合しても実用的な source proof にならない。

この compiler prerequisite は `ISS-20260515T232029920Z-BORROWED-ENUM-MATCH-CANNOT-BIND-OWNE-FD64ED88` として分離し、`&Enum` match / borrowed payload binding / Resource IR Borrow seeding / wasm・LLVM codegen の実装対象にした。Vec 側の根本修正は、この機能を前提として `VecStorage<T>` への field 統合を行う次段階で継続する。

## 検証

Add Resource IR tests that reject an independent Empty + owned RegionToken helper, accept valid Vec free paths, and source policy tests that forbid reintroducing an Empty no-op unless the owner is structurally carried by the Owned variant.
