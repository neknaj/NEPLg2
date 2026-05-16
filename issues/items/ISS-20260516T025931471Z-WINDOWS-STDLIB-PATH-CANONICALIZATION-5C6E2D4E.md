---
id: ISS-20260516T025931471Z-WINDOWS-STDLIB-PATH-CANONICALIZATION-5C6E2D4E
title: "Windows stdlib path canonicalization can drop source capabilities for virtual files"
area: core
status: open
resolved: false
priority: P1
type: bug
created: 2026-05-16
updated: 2026-05-16
target: nepl-core/src/loader.rs
---

# ISS-20260516T025931471Z-WINDOWS-STDLIB-PATH-CANONICALIZATION-5C6E2D4E: Windows stdlib path canonicalization can drop source capabilities for virtual files

## 概要

On Windows, canonicalizing an existing stdlib root can produce a verbatim \\?\C:\\ path while a virtual or non-existing stdlib child falls back to a normal C:\\ path. Loader::configured_stdlib_source_path then compares paths with different prefix forms and can fail starts_with, so compiler-owned source capabilities are not attached even though the path is under the configured stdlib root.

## 対象

- `nepl-core/src/loader.rs`

## 根拠

- `owner_aggregate_boundary_accepts_field_alias_import_call_head` の追加中、実在しない `stdlib/alloc/collections/owner_box/field_alias.nepl` を inline source path として使うと、`Loader::configured_stdlib_source_path` が false になり root source の `SourceCapabilities` が付与されなかった。
- 一時調査では `canon="C:\\neknaj\\neplg2_1\\stdlib\\..."`、`root="\\\\?\\C:\\neknaj\\neplg2_1\\stdlib"` となり、同じ stdlib root 配下であるにもかかわらず Windows verbatim prefix の有無で `PathBuf::starts_with` が一致しなかった。
- 今回の field evidence regression は実在する stdlib file path を使うようにして切り分けたが、loader の canonical representation は別途統一する必要がある。

## 問題

On Windows, canonicalizing an existing stdlib root can produce a verbatim \\?\C:\\ path while a virtual or non-existing stdlib child falls back to a normal C:\\ path. Loader::configured_stdlib_source_path then compares paths with different prefix forms and can fail starts_with, so compiler-owned source capabilities are not attached even though the path is under the configured stdlib root.

## 影響

Source capability tests using virtual stdlib files can silently exercise SourceCapabilities::none, and provider/virtual stdlib sources may lose raw memory or owner aggregate authority. This is an under-grant rather than a direct safety hole, but it weakens regression coverage and can hide real source-capability bugs.

## 修正方針

Normalize Windows verbatim prefixes before lexical path comparison, or store stdlib root and loaded source paths in a common canonical representation. Add a focused loader regression for an existing stdlib root with a non-existing stdlib child path.

## 検証

cargo test -p nepl-core loader::tests::configured_stdlib_source_path_accepts_virtual_child_under_existing_windows_root -- --nocapture
