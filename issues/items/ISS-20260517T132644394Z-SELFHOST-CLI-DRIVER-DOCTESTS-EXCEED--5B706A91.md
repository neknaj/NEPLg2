---
id: ISS-20260517T132644394Z-SELFHOST-CLI-DRIVER-DOCTESTS-EXCEED--5B706A91
title: "selfhost_cli_driver doctests exceed extended compile timeout"
area: TEST
status: fixed
resolved: true
priority: P1
type: performance
created: 2026-05-17
updated: 2026-05-17
target: "nepl-core/src/types.rs, nepl-core/src/resource/effect_summary_identity.rs, nepl-core/src/resource/effect_summary_pointer.rs, nepl-core/src/resource/effect_raw_provenance.rs, nepl-core/src/resource/raw_pointer_type.rs, nodesrc/test_typekind_doc_free_policy.js, tests/stdlib/selfhost_cli_driver.n.md"
---

# ISS-20260517T132644394Z-SELFHOST-CLI-DRIVER-DOCTESTS-EXCEED--5B706A91: selfhost_cli_driver doctests exceed extended compile timeout

## 概要

`tests/stdlib/selfhost_cli_driver.n.md` が compile phase で timeout していた。原因は selfhost fixture だけではなく、compiler 側で doc 文字列と raw provenance summary を hot path に乗せ、Resource Effect summary が型で不要と証明できる関数まで重い固定点計算を行っていたこと。

## 対象

- `nepl-core/src/types.rs`
- `nepl-core/src/resource/effect_summary_identity.rs`
- `nepl-core/src/resource/effect_summary_pointer.rs`
- `nepl-core/src/resource/effect_raw_provenance.rs`
- `nepl-core/src/resource/raw_pointer_type.rs`
- `nodesrc/test_typekind_doc_free_policy.js`
- `tests/stdlib/selfhost_cli_driver.n.md`

## 根拠

- `TypeKind::Struct` / `TypeKind::Enum` が `doc: Option<String>` を保持しており、stdlib の大きな doc comment が型同一性、unification、debug 出力、Resource IR summary の hot path に混入していた。
- raw identity / raw pointer return summary は、return 型が `str` など raw provenance を運べない場合や、`StringBuilder` / `RegionToken` の owned storage を non-owning pointer alias summary として扱う場合でも、callee 固定点を実行していた。
- untyped projection suffix の伝播により、recursive selfhost lexer/parser 経路で `Ok.Ok...` のような型構造上あり得ない return projection chain を summary 候補にできた。
- 修正前は `NEPL_TEST_CASE_TIMEOUT_MS=300000` でも対象 3 doctest が compile timeout した。修正後は通常設定で `tests/stdlib/selfhost_cli_driver.n.md` 全 3 件が pass した。

## 問題

`tests/stdlib/selfhost_cli_driver.n.md` currently timed out in compile phase for all three doctests even with `NEPL_TEST_CASE_TIMEOUT_MS=300000`. This blocked runtime verification of stdout fixture changes and exposed Resource Effect summary work that was not bounded by type-level necessity.

## 影響

Selfhost CLI driver behavior could not be verified by focused local doctest runs. The same root cause also made Stage 6 Resource IR / effect boundary checks scale with documentation payload and impossible raw provenance summary paths, so larger selfhost modules would continue to regress.

## 修正方針

Fix compiler complexity at the proof boundary without weakening static checks:

- Keep source/HIR documentation, but remove doc payload from `TypeKind` so type identity and Resource IR proof structures are semantic only.
- Validate return summary projections through `TypeCtx` before publishing them.
- Compute raw pointer summaries only for types that can carry non-owning raw pointer aliases.
- Compute raw identity summaries only where return type can require raw identity provenance.
- Keep pure internal allocation escape diagnostics by consuming summary-origin spans, not by re-running full raw provenance tracking on every function.
- Add source-policy regression coverage for TypeKind doc-free semantics and Resource checker module responsibility.

## 検証

Run focused `selfhost_cli_driver` doctests under normal timeout, source-policy regressions, core compile checks, issue checks, and diff checks.

## 関連計画

- `doc/neplg2/static_check_complexity_reduction_plan.md` Stage 6: compiler-core / Resource IR / raw memory provenance。今回の修正は `MemPtr = non-owning pointer` と `OwnedRegion/RegionToken = free obligation owner` の分離を summary 計算にも反映し、個別 stdlib 名の allowlist ではなく型構造と Resource IR proof から不要な計算を除外する。

## 対応内容

- `TypeKind::Struct` / `TypeKind::Enum` から `doc` field を削除した。AST/HIR 側の doc は保持し、型 identity / unification / Resource IR summary からだけ doc payload を除外した。
- raw identity return summary は `TypeCtx` で projection suffix を検証し、return 型が raw identity summary を必要としない場合は空 summary として早期終了するようにした。
- raw pointer return summary は `raw_pointer_type.rs` の typed predicate で non-owning raw pointer alias を運べる型だけを対象にし、owned storage token や `str` を pointer alias summary の seed / return にしないようにした。
- summary 由来の pure internal alloc escape diagnostic を `effect_raw_provenance.rs` に分離し、checked `MemPtr` access がない関数では full raw provenance tracking を避けるようにした。
- non-Copy move では raw identity / raw pointer alias / raw memory identity を transfer し、move 後の alias fact が source 側に残らないようにした。
- Resource checker responsibility policy に追加 module を登録し、今回の分割が監視から漏れないようにした。

## 検証結果

- `cargo check -p nepl-core`: pass
- `cargo test -p nepl-core typed_resource_ir_effect_check_keeps_i32_raw_identity_parameter_summary --test resource_ir -- --nocapture`: pass
- `node nodesrc/test_resource_checker_responsibility.js`: pass
- `node nodesrc/test_typekind_doc_free_policy.js`: pass
- `node nodesrc/run_source_policy_regressions.js`: pass
- `trunk build`: pass
- `node nodesrc/tests.js -i tests/stdlib/selfhost_cli_driver.n.md --no-tree -o tmp/agent1-selfhost-cli-driver-after-typed-seed.json -j 1 --dist web/dist --assert-io`: pass, 3/3

Focused doctest timing after the fix:

- `doctest#1`: compile 59037ms, run 19ms, total 59056ms
- `doctest#2`: compile 45921ms, run 13ms, total 45934ms
- `doctest#3`: compile 53748ms, run 20ms, total 53768ms
