---
id: ISS-20260517T075612588Z-RAW-ADDRESS-RETURN-PROOF-DUPLICATES--E5A6AE6F
title: "Raw address return proof duplicates field accessor and raw field spelling"
area: compiler/resource-ir
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-05-17
updated: 2026-05-17
target: "nepl-core/src/resource/lower_raw_address_return.rs; nepl-core/src/intrinsic_kinds.rs; nepl-core/src/resource_primitives.rs"
---

# ISS-20260517T075612588Z-RAW-ADDRESS-RETURN-PROOF-DUPLICATES--E5A6AE6F: Raw address return proof duplicates field accessor and raw field spelling

## 概要

transparent raw-address return proof が get/get_field と raw field name を local string branch で分類し、FieldAccessorKind と CompilerMemoryFieldSpec に集約した証明 domain を消費していない。

## 対象

- `nepl-core/src/resource/lower_raw_address_return.rs; nepl-core/src/intrinsic_kinds.rs; nepl-core/src/resource_primitives.rs`

## 根拠

- `FieldAccessorKind` は field accessor intrinsic と `core/field` member spelling の共有分類器であり、`CompilerMemoryFieldSpec` は `MemPtr` / `RegionToken` の内部 field contract の共有 source of truth である。
- `lower_raw_address_return.rs` は transparent raw-address return proof の中で `get` / `get_field` / `raw` を直接比較しており、共有 enum domain を消費していなかった。

## 問題

transparent raw-address return proof が get/get_field と raw field name を local string branch で分類し、FieldAccessorKind と CompilerMemoryFieldSpec に集約した証明 domain を消費していない。

## 影響

Resource IR の raw address return propagation が field accessor / compiler memory field contract の変更に追従できず、個別文字列分岐が古い proof rule として残る。

## 修正方針

FieldAccessorKind に source member と intrinsic spelling をまとめて classifier する query を追加し、lower_raw_address_return.rs の get/get_field/raw 判定を FieldAccessorKind::Get と CompilerMemoryFieldSpec::RawI32 に接続する。責務検査で direct string branch の再導入を禁止する。

## 対応内容

- `FieldAccessorKind::from_call_base_name` を追加し、source member (`get` など) と intrinsic (`get_field` など) の call-base spelling を同じ enum で分類できるようにした。
- `lower_raw_address_return.rs` の raw-address return proof は `FieldAccessorKind::Get` と `CompilerMemoryFieldSpec::RawI32.name()` を使うようにし、`matches!(base_name, "get" | "get_field")` と literal `"raw"` 判定を削除した。
- Resource checker responsibility policy に、transparent return proof が shared classifier / field spec を使うことと、direct string branch を戻さないことを追加した。
- 関連計画: [doc/neplg2/static_check_complexity_reduction_plan.md](../../doc/neplg2/static_check_complexity_reduction_plan.md)

## 検証

- `cargo fmt -p nepl-core --check`
- `cargo check -p nepl-core`
- `cargo test -p nepl-core field_accessor_call_base_name --lib -- --nocapture`
- `node nodesrc/test_resource_checker_responsibility.js`
- `node nodesrc/issues.js check --dir issues`
- `git diff --check`
