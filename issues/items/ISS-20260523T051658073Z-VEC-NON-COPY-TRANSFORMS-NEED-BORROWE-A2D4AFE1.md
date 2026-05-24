---
id: ISS-20260523T051658073Z-VEC-NON-COPY-TRANSFORMS-NEED-BORROWE-A2D4AFE1
title: "Vec non-Copy transforms need borrowed predicate move engine"
area: stdlib
status: open
resolved: false
priority: P1
type: architecture
created: 2026-05-23
updated: 2026-05-24
target: "stdlib/alloc/collections/vec/transform/**, nepl-core/src/resource/**"
---

# ISS-20260523T051658073Z-VEC-NON-COPY-TRANSFORMS-NEED-BORROWE-A2D4AFE1: Vec non-Copy transforms need borrowed predicate move engine

## 概要

Vec map/filter/take_while/drop_while/partition remain Copy-only and depend on by-value callbacks, get<T: Copy>, and raw storage views. Removing Copy bounds directly would move or shallow-copy non-Copy payloads outside the Resource IR slot lifecycle proof.

## 対象

- `stdlib/alloc/collections/vec/transform/**, nepl-core/src/resource/**`

## 根拠

- [ISS-20260520T152218366Z-NON-COPY-COLLECTION-PAYLOAD-SUPPORT--A6A88543](./ISS-20260520T152218366Z-NON-COPY-COLLECTION-PAYLOAD-SUPPORT--A6A88543.md) の残件監査で、`Vec` の push / grow / free / clear / pop / replace / borrowed query は Resource IR proof boundary へ接続済みだが、transform family はまだ Copy-by-value 境界に残っている。
- [NEPLg2 静的検査の複雑化解消計画](../../doc/neplg2/static_check_complexity_reduction_plan.md) Stage 6 は、collection lifecycle を stdlib module allowlist ではなく generic Resource IR proof として扱う方針である。
- [stdlib collection/mem/string と静的検査の安全設計](../../doc/neplg2/stdlib_collection_mem_string_static_safety_design.md) は、non-Copy payload observer / move-out / drop traversal を raw `MemPtr` helper の復活ではなく `BorrowRead` / `MoveOut` / `InitializeEmpty` / `DropInitialized` へ接続する方針を定めている。
- 現行の `stdlib/alloc/collections/vec/transform/map.nepl`, `filter/select.nepl`, `prefix.nepl`, `filter/partition/build.nepl` は、`.T: Copy`、`(.T)->bool` または値渡し transformer、`get<T: Copy>` / raw storage view に依存している。
- 2026-05-24 の実装検討で、`filter<T: Drop>` を stdlib-only direct raw drain にすると Resource IR が source `MoveOut` と output `InitializeEmpty` の range lifecycle を証明できず、`pop` / `push` 再帰案は Drop payload doctest が 240 秒でも timeout した。これを [ISS-20260524T020418962Z-RESOURCE-IR-NEEDS-TRANSFORM-RANGE-LI-77E29B37](./ISS-20260524T020418962Z-RESOURCE-IR-NEEDS-TRANSFORM-RANGE-LI-77E29B37.md) として切り出した。

## 問題

Vec map/filter/take_while/drop_while/partition remain Copy-only and depend on by-value callbacks, get<T: Copy>, and raw storage views. Removing Copy bounds directly would move or shallow-copy non-Copy payloads outside the Resource IR slot lifecycle proof.

## 影響

Self-host code cannot transform owning AST/HIR/diagnostic payload Vec values without manual arenas or bespoke cleanup. Leaving the old shape also invites ad hoc stdlib-specific proof exceptions.

## 修正方針

Design and implement a generic transform engine that uses borrowed predicates for observation, MoveOut for selected slots, InitializeEmpty for output slots, actual Drop proof for discarded slots, and owner-preserving VecTransformError / rollback cleanup on failure. Do not add stdlib function allowlists or per-transform proof engines.

`ISS-20260524T020418962Z-RESOURCE-IR-NEEDS-TRANSFORM-RANGE-LI-77E29B37` を先に解き、drop traversal range certificate と同じ水準の typed transform range lifecycle certificate を Resource IR に追加してから stdlib `filter` / prefix / `map` / `partition` の public non-Copy overload を開く。

## 進捗

2026-05-24 checkpoint:

- `filter<T: Drop>` の Owned/Owned success path は `(&T)->bool` borrowed predicate と private `vec_filter_drain_to_output` boundary で Resource IR TransformRange certificate へ接続した。
- `vec_filter_drain_to_output` は source initialized prefix の証明境界として `src_initialized_len` を loop bound / TransformRange marker count に使う。`src_len` と storage initialized range を混同しない。
- source-level regression として `Vec<DropPayload>.filter` が source drain、selected output initialization、discard actual drop、borrow scoped predicate、return output cleanup を通すことを固定した。
- source policy は public `filter` に raw lifecycle marker を置かず、private TransformRange drain helper と public delegated filter surface を構造的に検査する。
- まだ `filter` だけの first connection であり、prefix / map / partition は未接続。特に `partition` は matched/rest の 2 output owner と rollback cleanup を別途設計する必要があるため、この issue は open のまま維持する。
- map は borrowed observation ではなく transformer の出力 owner と input owner の関係を扱うため、TransformRange をそのまま流用できる部分と `U` output construction proof を分けて検討する。

## 検証

Add Resource IR compile-pass/fail regressions for Drop payload filter/prefix/map/partition lifecycle, source policies preventing Copy-bound removal without borrowed/move/drop proof, and focused doctests.
