---
id: ISS-20260514T054314434Z-COPY-IMPL-CAN-MARK-COMPILER-OWNER-TO-D6C08048
title: "Copy impl can mark compiler owner tokens as copyable"
area: core
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-14
updated: 2026-05-14
target: "nepl-core/src/typecheck/copy_capability.rs, nepl-core/src/typecheck/driver.rs, tests/compiler/move_effect.n.md, nepl-core/tests/neplg2.rs"
---

# ISS-20260514T054314434Z-COPY-IMPL-CAN-MARK-COMPILER-OWNER-TO-D6C08048: Copy impl can mark compiler owner tokens as copyable

## 概要

The Copy capability validator checks structural copy eligibility but does not reject compiler memory owner-token definitions such as RegionToken. Since RegionToken is structurally MemPtr plus i32, a source impl can mark it Copy even though the Resource IR owner model treats it as the free-obligation owner token.

## 対象

- `nepl-core/src/typecheck/copy_capability.rs, nepl-core/src/typecheck/driver.rs, tests/compiler/move_effect.n.md, nepl-core/tests/neplg2.rs`

## 根拠

- `RegionToken<T>` は compiler memory boundary が発行する free-obligation owner token であり、`MemPtr<T>` のような non-owning pointer projection ではない。
- direct constructor restriction は `StructConstructorPolicy::RawMemoryBoundaryOnly(OwnerToken)` で既に区別していたが、Copy capability impl の target validation はこの policy を見ていなかった。
- `RegionToken<T>` は構造だけを見ると `MemPtr<T>` と `i32` の field で構成されるため、構造的 copy eligibility だけでは owner token の線形性を証明できない。

## 問題

The Copy capability validator checks structural copy eligibility but does not reject compiler memory owner-token definitions such as RegionToken. Since RegionToken is structurally MemPtr plus i32, a source impl can mark it Copy even though the Resource IR owner model treats it as the free-obligation owner token.

## 影響

A free-obligation owner token can be duplicated at the trait/copy layer, weakening the static-check contract that owner tokens are linear and that MemPtr remains the copyable non-owning projection.

## 修正方針

Reject Copy-capability impl targets whose resolved struct definition has the compiler owner-token constructor policy. Keep raw pointer MemPtr copyable and keep ordinary user structs named RegionToken unaffected unless their source definition was recognized as the compiler memory owner token.

## 対応

- Copy capability impl target の検証を `typecheck/copy_capability.rs` に分離し、resolved struct definition が `StructConstructorPolicy::RawMemoryBoundaryOnly(RestrictedStructConstructor::OwnerToken)` を持つ場合は `type.copy_impl.target_not_copy` として拒否するようにした。
- 判定は struct 名の文字列だけではなく、`TypeCtx::same_type` と struct table 上の constructor policy に基づけた。これにより compiler-owned stdlib `RegionToken<T>` だけを拒否し、`#no_prelude` の通常 user struct が偶然 `RegionToken` という名前を持つ場合は従来どおり構造的 Copy を許可する。
- `tests/compiler/move_effect.n.md` と Rust integration test に、owner token への Copy impl 拒否と、同名 user struct への Copy impl 許可の両方を追加した。

## 検証

- `cargo test -p nepl-core --test neplg2 copy_impl -- --nocapture`
- `node nodesrc/test_static_check_boundary_responsibility.js`
- `trunk build`
- `node nodesrc/run_doctest.js -i tests/compiler/move_effect.n.md -n 99 --dist web/dist`
- `node nodesrc/run_doctest.js -i tests/compiler/move_effect.n.md -n 100 --dist web/dist`
