---
id: ISS-20260514T153830277Z-STRING-APPEND-BOUNDARY-POLICY-STILL--F431B18E
title: "StringBuilder wrapper boundary policy still requires raw evidence for safe wrappers"
area: stdlib
status: fixed
resolved: true
priority: P1
type: test
created: 2026-05-14
updated: 2026-05-15
target: "nodesrc/test_stdlib_string_facade_boundary.js, stdlib/alloc/string/builder/{append,build,reserve,types}.nepl"
---

# ISS-20260514T153830277Z-STRING-APPEND-BOUNDARY-POLICY-STILL--F431B18E: StringBuilder wrapper boundary policy still requires raw evidence for safe wrappers

## 概要

The alloc/string facade boundary policy still requires StringBuilder wrapper modules such as stdlib/alloc/string/builder/append.nepl, build.nepl, reserve.nepl, and types.nepl to carry source-level raw memory boundary evidence, but those modules now delegate storage mutation to ByteBuilder and only handle the text API wrapper.

## 対象

- `nodesrc/test_stdlib_string_facade_boundary.js, stdlib/alloc/string/builder/{append,build,reserve,types}.nepl`

## 根拠

- `node nodesrc/test_stdlib_string_facade_boundary.js` が `stdlib/alloc/string/builder/append.nepl must carry source-level raw memory boundary evidence` で失敗し、同じ list には direct raw operation を持たない `build.nepl` / `reserve.nepl` / `types.nepl` も残っていた。
- `stdlib/alloc/string/builder/append.nepl` は `byte_builder_push_bytes_ref` / `byte_builder_push_char_utf8` / `byte_builder_push_u8` へ委譲する text API wrapper であり、direct `mem_copy` / `alloc_region` / `RegionToken` owner manipulation を持っていない。
- `build.nepl` は `byte_builder_finish` / `io_bytebuf_to_str_result`、`reserve.nepl` は `byte_builder_*` grow API、`types.nepl` は `StringBuilder` wrapper type と borrowed view API へ責務を限定している。
- Stage 6 の方針は raw authority を source property proof に寄せることであり、safe wrapper に raw evidence を要求して raw-memory-boundary を広げることではない。

## 問題

The alloc/string facade boundary policy still requires StringBuilder wrapper modules such as stdlib/alloc/string/builder/append.nepl, build.nepl, reserve.nepl, and types.nepl to carry source-level raw memory boundary evidence, but those modules now delegate storage mutation to ByteBuilder and only handle the text API wrapper.

## 影響

The source policy pressures a safe wrapper module to regain raw-memory-boundary evidence, widening Stage 6 authority instead of proving raw operations in the lower storage modules that actually own them.

## 修正方針

Move the StringBuilder wrapper modules out of the raw-evidence-required list and add negative assertions that they stay free of direct raw memory evidence, while keeping raw evidence requirements for concat/builder_ext/format modules that still own raw storage operations.

## 検証

Run node nodesrc/test_stdlib_string_facade_boundary.js, source policy runner in warn-only mode, issue check, and diff whitespace check.

## 2026-05-15 Agent 1 修正

`nodesrc/test_stdlib_string_facade_boundary.js` の raw-evidence-required list から `stdlib/alloc/string/builder/append.nepl` / `build.nepl` / `reserve.nepl` / `types.nepl` を外し、代わりにこれらの file が direct raw memory evidence を持たないことを監視する negative assertion へ移した。

これらは `StringBuilder` の text API wrapper であり、storage mutation は `ByteBuilder` 境界へ委譲している。ここへ raw evidence を要求すると、source capability proof が必要最小境界ではなく wrapper layer へ広がるため、Stage 6 の raw boundary shrink と逆方向になる。

検証:

- `node nodesrc/test_stdlib_string_facade_boundary.js`: passed
- `node nodesrc/run_source_policy_regressions.js --warn-only`: string facade warning は解消。documentation contract warning は継続。
- `node nodesrc/issues.js check --dir issues`: passed
- `git diff --check`: passed
