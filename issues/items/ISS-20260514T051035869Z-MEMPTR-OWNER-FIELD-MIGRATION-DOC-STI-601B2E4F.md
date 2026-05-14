---
id: ISS-20260514T051035869Z-MEMPTR-OWNER-FIELD-MIGRATION-DOC-STI-601B2E4F
title: "MemPtr owner field migration doc still lists resolved StreamWriter.buf"
area: docs
status: fixed
resolved: true
priority: P2
type: bug
created: 2026-05-14
updated: 2026-05-14
target: doc/neplg2/stdlib_collection_mem_string_static_safety_design.md
---

# ISS-20260514T051035869Z-MEMPTR-OWNER-FIELD-MIGRATION-DOC-STI-601B2E4F: MemPtr owner field migration doc still lists resolved StreamWriter.buf

## 概要

The stdlib collection/string memory safety design still lists StreamWriter.buf as a remaining MemPtr owner-like field even after the StreamWriter state was migrated to ByteBuilder.

## 対象

- `doc/neplg2/stdlib_collection_mem_string_static_safety_design.md`

## 根拠

- `doc/neplg2/stdlib_collection_mem_string_static_safety_design.md` の Stage A 追記が、`StreamWriter.buf` を残存 `MemPtr` owner-like field として列挙したままだった。
- 実際の source policy `nodesrc/test_stdlib_memptr_owner_field_policy.js` は `StreamWriter.buf` の transitional exception を削除済みで、現在の baseline は 7 field である。

## 問題

The stdlib collection/string memory safety design still lists StreamWriter.buf as a remaining MemPtr owner-like field even after the StreamWriter state was migrated to ByteBuilder.

## 影響

Stage 6 progress tracking can overstate remaining direct MemPtr owner fields and mislead future work toward a field that no longer exists.

## 修正方針

Refresh the remaining MemPtr owner-field list to remove StreamWriter.buf and state the current seven-field baseline.

## 検証

Run the issue checker and the stdlib MemPtr owner-field migration policy.

## 対応内容

- `stdlib_collection_mem_string_static_safety_design.md` の残存 `MemPtr` owner-like field 一覧から `StreamWriter.buf` を削除した。
- 2026-05-14 追記として、`StreamWriter` が `ByteBuilder` owner boundary へ移行済みであり、残件 baseline が 7 field であることを明記した。

## 検証結果

- `node nodesrc/test_stdlib_memptr_owner_field_policy.js` で current baseline が 7 transitional field であることを確認する。
- `node nodesrc/issues.js check --dir issues` で issue metadata の整合を確認する。
