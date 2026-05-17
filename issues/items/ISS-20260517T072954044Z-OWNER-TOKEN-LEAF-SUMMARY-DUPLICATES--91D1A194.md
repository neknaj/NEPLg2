---
id: ISS-20260517T072954044Z-OWNER-TOKEN-LEAF-SUMMARY-DUPLICATES--91D1A194
title: "Owner token leaf summary duplicates compiler memory field contract"
area: compiler/resource-ir
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-05-17
updated: 2026-05-17
target: "nepl-core/src/resource/owner_summary_owner_token_leaf.rs; nepl-core/src/resource_primitives.rs; nodesrc/test_resource_checker_responsibility.js"
---

# ISS-20260517T072954044Z-OWNER-TOKEN-LEAF-SUMMARY-DUPLICATES--91D1A194: Owner token leaf summary duplicates compiler memory field contract

## 概要

RegionToken の raw leaf 抽出が owner_summary_owner_token_leaf.rs 内で field_name == raw を直接判定しており、CompilerMemoryFieldSpec に集約した内部メモリ形状契約と二重管理になっている。

## 対象

- `nepl-core/src/resource/owner_summary_owner_token_leaf.rs; nepl-core/src/resource_primitives.rs; nodesrc/test_resource_checker_responsibility.js`

## 根拠

- `doc/neplg2/static_check_complexity_reduction_plan.md` の Stage 6 では、`MemPtr = non-owning pointer` / `RegionToken = free obligation owner` の内部メモリ形状を source proof、typecheck registration、Resource IR semantic proof へ接続する方針を取っている。
- 既存の `CompilerMemoryFieldSpec` 集約後も、`owner_summary_owner_token_leaf.rs` だけが `RegionToken.raw` の leaf を local な field name 判定から導出していた。

## 問題

RegionToken の raw leaf 抽出が owner_summary_owner_token_leaf.rs 内で field_name == raw を直接判定しており、CompilerMemoryFieldSpec に集約した内部メモリ形状契約と二重管理になっている。

## 影響

静的検査の Resource IR がソース型から証明した内部メモリ形状ではなく個別実装の文字列判定に依存し、RegionToken 形状変更時に owner summary 側だけが古い契約を使い続ける危険がある。

## 修正方針

owner token raw leaf の抽出を CompilerMemoryType::OwnerToken と CompilerMemoryFieldSpec::RawI32 から導出し、型が owner token と証明できない場合は leaf を生成しない。責務検査も個別文字列判定を禁止する。

## 対応内容

- `CompilerMemoryFieldSpec` に対する field index query を追加し、Resource IR owner summary が shared field spec から raw leaf index を導出するようにした。
- `owner_summary_owner_token_leaf.rs` は `type_is_owner_token` で TypeCtx の証明済み owner-token identity を確認してから leaf を生成する。未証明の同形 struct は free obligation owner leaf にならない。
- Resource checker responsibility policy を更新し、owner-token raw leaf が `CompilerMemoryType::OwnerToken` / `CompilerMemoryFieldSpec::RawI32` / `type_is_owner_token` を使うことと、local な `field_name == "raw"` 判定を戻さないことを検査する。
- 関連計画: [doc/neplg2/static_check_complexity_reduction_plan.md](../../doc/neplg2/static_check_complexity_reduction_plan.md)

## 検証

- `cargo fmt -p nepl-core --check`
- `cargo check -p nepl-core`
- `cargo test -p nepl-core owner_token_leaf --lib -- --nocapture`
- `cargo test -p nepl-core compiler_memory_type_field_specs_are_kind_owned --lib -- --nocapture`
- `node nodesrc/test_resource_checker_responsibility.js`
- `node nodesrc/issues.js check --dir issues`
- `git diff --check`
