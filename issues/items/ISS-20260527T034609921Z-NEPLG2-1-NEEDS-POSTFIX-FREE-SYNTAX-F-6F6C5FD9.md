---
id: ISS-20260527T034609921Z-NEPLG2-1-NEEDS-POSTFIX-FREE-SYNTAX-F-6F6C5FD9
title: "NEPLg2.1 needs postfix-free syntax for type-only layout generic calls"
area: neplg21
status: verified
resolved: true
priority: P1
type: architecture
created: 2026-05-27
updated: 2026-05-28
target: "nepl-core/src/parser/**; nepl-core/src/typecheck/**; stdlib/core/mem/{layout,types}.nepl; tests/compiler/{intrinsic,sizeof}.n.md"
---

# ISS-20260527T034609921Z-NEPLG2-1-NEEDS-POSTFIX-FREE-SYNTAX-F-6F6C5FD9: NEPLg2.1 needs postfix-free syntax for type-only layout generic calls

## 概要

NEPLg2.1 removes postfix generic calls, but size_of<T> and align_of<T> carry their type argument only in type space and return plain i32. Current inference has no value/result evidence to recover T after a purely mechanical cleanup.

## 対象

- `nepl-core/src/parser/**; nepl-core/src/typecheck/**; stdlib/core/mem/{layout,types}.nepl`

## 根拠

- `size_of<T>` / `align_of<T>` は値引数を持たず、戻り値も `i32` だけであるため、`alloc_region<T>` のように戻り値の `Result RegionToken T str` から `T` を復元する経路がない。
- core/mem の positive doctest cleanup を進める過程で、`alloc_region<T>` / `dealloc_region<T>` / `region_ptr_at<T,U>` などは typed local の `%Result ...` 注釈へ移せる一方、type-only layout query だけは同じ方法では移せないことを確認した。
- NEPLg2.1 の後置ジェネリクス撤廃を完了させるには、型だけを問い合わせる組込み/stdlib helper に対する公式の前置記法、または compiler 側の明示的な推論・例外設計が必要である。

## 問題

NEPLg2.1 removes postfix generic calls, but size_of<T> and align_of<T> carry their type argument only in type space and return plain i32. Current inference has no value/result evidence to recover T after a purely mechanical cleanup.

## 影響

Positive core/mem doctests for size_of<i32>, align_of<i32>, and size_of<MemPtr<i32>> cannot be migrated to postfix-free source without either a dedicated surface form or compiler inference support. Leaving the gap undocumented would hide a real NEPLg2.1 migration boundary.

## 修正方針

Define an official postfix-free source form or compiler-owned exception for type-only layout queries, then migrate stdlib/core/mem/layout.nepl and stdlib/core/mem/types.nepl positive doctests away from postfix generic calls.

## 検証

Add parser/typechecker coverage for the selected postfix-free type-only generic form and extend the NEPLg2.1 core/mem doccomment source-policy so size_of and align_of no longer need an exclusion.

## 2026-05-28 解決内容

- `size_of %T` / `align_of %T` を NEPLg2.1 の公式 postfix-free layout query 形として採用した。
- parser は `size_of` / `align_of` の直後に限って `%` type expression を explicit type args として `Symbol::Ident` に保持する。これは値注釈でも部分適用でもなく、layout query 専用の compiler-owned type metadata である。
- 通常関数の `callee %i32 value` は従来どおり value-level type ascription として扱う回帰テストを追加した。
- `stdlib/core/mem/layout.nepl`、`stdlib/core/mem/types.nepl`、`tests/compiler/intrinsic.n.md`、`tests/compiler/sizeof.n.md` の layout query 呼び出しを `size_of %...` / `align_of %...` へ移行した。
- source policy regression は executable code / doctest code fence だけを対象にし、コメントや doccomment の増加そのものを制限しない形で旧 `size_of<...>` / `align_of<...>` を検出する。

## 残件の分離

- `size_of_t<i32>` のような値引数なし・戻り値 `i32` の user generic wrapper は、型根拠が call site に残らないため本 issue とは別問題として扱う。
- 追加 issue: `ISS-20260528T085628387Z-NEPLG2-1-NEEDS-POSTFIX-FREE-TYPE-EVI-43646AFF`

## 2026-05-28 検証

- pass: `cargo fmt -p nepl-core --check`
- pass: `cargo check -p nepl-core`
- pass: `cargo check --manifest-path nepl-web\Cargo.toml`
- pass: `cargo test -p nepl-core --test intrinsic intrinsic_size_and_align_neplg21_type_marker -- --nocapture`
- pass: `cargo test -p nepl-core --test resource_ir resource_ir_layout_intrinsics_use_shared_core_intrinsic_kind -- --nocapture`
- pass: `cargo test -p nepl-core --test typeannot test_neplg21_percent_after_normal_callee_remains_value_annotation -- --nocapture`
- pass: `trunk build`
- pass: `node nodesrc/tests.js -i tests/compiler/intrinsic.n.md --no-tree -o tmp/neplg21-core-mem-type-query-intrinsic-20260528.json -j 1 --dist web/dist --assert-io`
- pass: `node nodesrc/tests.js -i stdlib/core/mem/layout.nepl -i stdlib/core/mem/types.nepl --no-tree -o tmp/neplg21-core-mem-layout-type-query-20260528.json -j 1 --dist web/dist --assert-io`
- pass: `node nodesrc/tests.js -i tests/compiler/sizeof.n.md --no-tree -o tmp/neplg21-core-mem-sizeof-type-query-20260528.json -j 1 --dist web/dist --assert-io`
- pass: `node nodesrc/test_neplg21_core_mem_layout_type_query_cleanup.js`
- pass: `node nodesrc/test_neplg21_core_mem_positive_doc_postfix_cleanup.js`
- pass with existing warn-only findings: `node nodesrc/run_source_policy_regressions.js --warn-only`
