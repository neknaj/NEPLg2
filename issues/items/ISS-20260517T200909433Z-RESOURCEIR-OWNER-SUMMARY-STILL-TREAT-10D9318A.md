---
id: ISS-20260517T200909433Z-RESOURCEIR-OWNER-SUMMARY-STILL-TREAT-10D9318A
title: "ResourceIR owner summary still treats copied str views as moved across stdlib_map resolution"
area: CORE
status: open
resolved: false
priority: P0
type: architecture
created: 2026-05-17
updated: 2026-05-17
target: "nepl-core/src/resource, stdlib/neplg2/core/module/stdlib_map.nepl, stdlib/std/test/report.nepl"
---

# ISS-20260517T200909433Z-RESOURCEIR-OWNER-SUMMARY-STILL-TREAT-10D9318A: ResourceIR owner summary still treats copied str views as moved across stdlib_map resolution

## 概要

Running tests/stdlib/neplg2_stdlib_map.n.md after converting it to stdout/exit_code reports still fails during ResourceIR owner checking. The remaining current diagnostics are `resource.owner.use_after_move` for `str` view values such as `base_dir` and a temporary path value in `selfhost_module_path_resolve_relative*`. This shows the current owner summary still treats non-owning Copy string views as if they were moved linear owners in this path-resolution flow.

## 対象

- `nepl-core/src/resource, stdlib/neplg2/core/module/stdlib_map.nepl, stdlib/std/test/report.nepl`

## 根拠

- `trunk build` 後に `node nodesrc/run_doctest.js -i tests\stdlib\neplg2_stdlib_map.n.md -n 1 --assert-io --dist web\dist` を実行すると、exit code comparison ではなく compile phase で失敗する。
- raw `runSingle` で3件を確認すると、いずれも `resource.owner.use_after_move` を報告する。対象は `selfhost_module_path_resolve_relative` の `base_dir: str` と、`selfhost_module_path_resolve_relative_loop` の temporary `str` path value である。
- Copy payload source reservation の一部は `ISS-20260517T201648863Z-RESOURCEIR-VARIANT-OWNER-RESERVATION-F28E5200` で解消したが、owner summary apply / call argument handling 側には non-owning Copy `str` view を linear owner moved state と混同する問題が残る。

## 問題

Running tests/stdlib/neplg2_stdlib_map.n.md after converting it to stdout/exit_code reports still fails during ResourceIR owner checking. The remaining current diagnostics are `resource.owner.use_after_move` for `str` view values such as `base_dir` and a temporary path value in `selfhost_module_path_resolve_relative*`. This shows the current owner summary still treats non-owning Copy string views as if they were moved linear owners in this path-resolution flow.

## 影響

Self-host stdlib doctests that should validate module path mapping cannot execute under the static checker, and future self-host code that copies `str` views through helper calls can be rejected or hidden behind stale test metadata. This is memory-safety critical because the checker must distinguish non-owning views from true free-obligation storage without per-module allowlists.

## 修正方針

Redesign ResourceIR owner summary application so Copy/non-owning `str` view places are not consumed or marked moved by call argument/return summaries, while raw owner leaves backed by storage_origin remain tracked. The proof must be derived from type Copy capability, storage origin, and ResourceIR owner-token structure, not from stdlib module names. Add focused regressions for stdlib_map relative path resolution and generic Copy view call arguments.

## 検証

Focused cargo tests for ResourceIR owner summary/variant behavior, tests/stdlib/neplg2_stdlib_map.n.md with --assert-io, and source policy regressions.

## 2026-05-17 切り分け

stdlib_map doctest の report metadata を `stdout` / `exit_code` に移行した後、runner は stale `ret:` mismatch ではなく compiler diagnostic まで進むようになった。これは test manifest の問題ではなく、ResourceIR owner summary が non-owning `str` view と free-obligation owner を十分に分離できていない core 側問題である。

この issue では stdlib module 名や特定 helper 名を許可する形の対症療法は禁止する。修正は TypeCtx の Copy capability、storage origin、ResourceIR owner token、aggregate projection の構造から一般的に証明し、`str` view の call argument / return flow と true owner-bearing storage flow を同じ証明器で区別する。
