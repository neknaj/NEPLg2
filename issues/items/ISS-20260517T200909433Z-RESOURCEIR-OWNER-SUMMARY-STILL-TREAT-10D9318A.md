---
id: ISS-20260517T200909433Z-RESOURCEIR-OWNER-SUMMARY-STILL-TREAT-10D9318A
title: "ResourceIR owner summary still treats copied str views as moved across stdlib_map resolution"
area: CORE
status: fixed
resolved: true
priority: P0
type: architecture
created: 2026-05-17
updated: 2026-05-18
target: "nepl-core/src/resource, stdlib/neplg2/core/module/stdlib_map.nepl, stdlib/std/test/report.nepl"
---

# ISS-20260517T200909433Z-RESOURCEIR-OWNER-SUMMARY-STILL-TREAT-10D9318A: ResourceIR owner summary still treats copied str views as moved across stdlib_map resolution

## 概要

Running tests/stdlib/neplg2_stdlib_map.n.md after converting it to stdout/exit_code reports failed during ResourceIR owner checking. The first blocker was `resource.owner.use_after_move` for `str` view values such as `base_dir` and a temporary path value in `selfhost_module_path_resolve_relative*`; after that was fixed, the same proof gap exposed `resource.owner.maybe_leak` on `std/test` report `str` fields returned through branch/call summaries. Both cases came from ResourceIR owner summary treating Copy `str` views and owner-backed `str` storage with the same rule.

## 対象

- `nepl-core/src/resource, stdlib/neplg2/core/module/stdlib_map.nepl, stdlib/std/test/report.nepl`

## 根拠

- `trunk build` 後に `node nodesrc/run_doctest.js -i tests\stdlib\neplg2_stdlib_map.n.md -n 1 --assert-io --dist web\dist` を実行すると、exit code comparison ではなく compile phase で失敗していた。
- raw `runSingle` で3件を確認すると、いずれも `resource.owner.use_after_move` を報告した。対象は `selfhost_module_path_resolve_relative` の `base_dir: str` と、`selfhost_module_path_resolve_relative_loop` の temporary `str` path value である。
- その後の focused regression で、`push_err -> checks_print_report -> checks_exit_code` の branch report flow が `TestReport.lines` / `legacy_summary` / `legacy_human` を `resource.owner.maybe_leak` として残すことも確認した。型として Copy である `str` を一律 view 扱いすると、owner-backed report string まで summary から消えるため、所有状態も判定に含める必要がある。
- Copy payload source reservation の一部は `ISS-20260517T201648863Z-RESOURCEIR-VARIANT-OWNER-RESERVATION-F28E5200` で解消したが、owner summary apply / call argument handling 側には non-owning Copy `str` view を linear owner moved state と混同する問題が残る。

## 問題

Running tests/stdlib/neplg2_stdlib_map.n.md after converting it to stdout/exit_code reports failed during ResourceIR owner checking. The remaining diagnostics showed that owner summary application did not distinguish non-owning Copy `str` views from true owner-backed `str` storage.

## 影響

Self-host stdlib doctests that should validate module path mapping could not execute under the static checker, and future self-host code that copies `str` views through helper calls could be rejected or hidden behind stale test metadata. This is memory-safety critical because the checker must distinguish non-owning views from true free-obligation storage without per-module allowlists.

## 修正方針

ResourceIR owner summary application now treats Copy owner leaves as view facts only when the source has no transferable owner obligation. If the source has `Live` / `MaybeFreed` ownership, the normal owner transfer path remains active. Return storage-origin owners are moved out when a returned value carries copied owner origins from outside the return place. The proof is derived from `TypeCtx::is_copy`, owner leaf structure, owner table state, raw aliases, and storage origins; it does not use stdlib module-name allowlists.

## 検証

Focused cargo tests for ResourceIR owner summary/variant behavior, tests/stdlib/neplg2_stdlib_map.n.md with --assert-io, and source policy regressions.

## 2026-05-17 切り分け

stdlib_map doctest の report metadata を `stdout` / `exit_code` に移行した後、runner は stale `ret:` mismatch ではなく compiler diagnostic まで進むようになった。これは test manifest の問題ではなく、ResourceIR owner summary が non-owning `str` view と free-obligation owner を十分に分離できていない core 側問題である。

この issue では stdlib module 名や特定 helper 名を許可する形の対症療法は禁止する。修正は TypeCtx の Copy capability、storage origin、ResourceIR owner token、aggregate projection の構造から一般的に証明し、`str` view の call argument / return flow と true owner-bearing storage flow を同じ証明器で区別する。

## 2026-05-18 修正

- `consume_owner_summary_parameters` と owner return summary application で、Copy `str` leaf を一律に move / consume しない代わりに、`OwnerTable` / raw alias 解決後に transferable owner がない場合だけ non-owning view facts として copy するようにした。
- owner-backed `str` storage は `Live` / `MaybeFreed` のまま通常の owner transfer に流すため、`TestReport` の `name` / `lines` / `legacy_summary` / `legacy_human` のような本物の free obligation は summary から落ちない。
- variant pending owner effect も同じ判定に合わせ、Copy view payload の inactive reservation は消費せず、owner-backed payload は既存の transfer / unavailable diagnostic を維持する。
- returned aggregate が外側 storage origin を保持する場合、return terminator で origin source owner も return-value operation として move out するようにした。これにより、copied view を返す summary と owner escape の整合を保つ。
- 回帰テストとして `resource_ir_owner_summary_keeps_copy_str_views_after_selfhost_path_resolution` と `resource_ir_owner_summary_returns_branch_report_with_copy_str_payloads` を追加した。
- 検証:
  - `cargo fmt -p nepl-core --check`
  - `cargo test -p nepl-core resource_ir_owner_summary_keeps_copy_str_views_after_selfhost_path_resolution --test resource_ir -- --nocapture`
  - `cargo test -p nepl-core resource_ir_owner_summary_returns_branch_report_with_copy_str_payloads --test resource_ir -- --nocapture`
  - ResourceIR owner summary / variant / raw owner focused regressions 8件
  - `trunk build`
  - `node nodesrc/run_doctest.js -i tests\stdlib\neplg2_stdlib_map.n.md -n 1 --assert-io --dist web\dist`
