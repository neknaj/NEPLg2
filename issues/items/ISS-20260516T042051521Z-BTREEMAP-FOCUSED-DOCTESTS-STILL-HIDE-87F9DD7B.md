---
id: ISS-20260516T042051521Z-BTREEMAP-FOCUSED-DOCTESTS-STILL-HIDE-87F9DD7B
title: "BTreeMap focused doctests still hide assertion details behind checks_exit_code"
area: TEST
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-05-16
updated: 2026-05-16
target: "stdlib/tests/btreemap.n.md, nodesrc/test_stdlib_btreemap_report_contract.js, nepl-core/src/resource/owner_summary*.rs, nepl-core/src/source_capability/**"
---

# ISS-20260516T042051521Z-BTREEMAP-FOCUSED-DOCTESTS-STILL-HIDE-87F9DD7B: BTreeMap focused doctests still hide assertion details behind checks_exit_code

## 概要

`stdlib/tests/btreemap.n.md` uses `std/test` `Checks` plus `checks_exit_code`, so successful runs do not pin assertion labels, expected values, or actual values in stdout.

The stdout migration exposed a deeper Resource IR false positive: owner summary treated every `i32` leaf inside an owner-token-backed aggregate as a possible free obligation. Returning `BTreeMap` through helper calls therefore made collection metadata and `RegionToken.size` look like leaked raw owners, even though only `RegionToken.raw` is the actual free-obligation identity.

## 対象

- `stdlib/tests/btreemap.n.md`
- `nodesrc/test_stdlib_btreemap_report_contract.js`
- `nepl-core/src/resource/owner_summary*.rs`
- `nepl-core/src/source_capability/**`

## 根拠

- `stdlib/tests/btreemap.n.md` の 5 件は `checks_exit_code` だけで成功を表し、stdout に assertion label / expected / actual を固定していなかった。
- 5 件を canonical `TestReport` stdout fixture へ移行すると、`btreemap_insert_error_rolls_back_owner` が `resource.owner.maybe_leak` で止まった。
- 原因は BTreeMap 固有の不足ではなく、Resource IR owner summary が owner-token を含む aggregate の `i32` leaf を全て raw free obligation candidate として列挙していたことだった。
- Stage 6 の方針では `MemPtr<T>` は non-owning pointer、`RegionToken<T>.raw` が free obligation identity、`RegionToken<T>.size` や collection `len` / `cap` は条件・metadata scalar である。metadata scalar を owner 候補に混ぜると型安全・メモリ安全の証明が偽陽性になる。

## 問題

- BTreeMap focused doctest が exit-code-only で、collection behavior の regression を stdout diff として確認できなかった。
- stdout 化で露出した Resource IR owner summary は、`i32` condition/value leaf と raw free-obligation leaf の責務を分けていなかった。
- `BTreeMap` / `Vec` / `RegionToken` のような owner-token-backed aggregate では、`RegionToken.raw` だけが free obligation owner であり、`RegionToken.size` と collection metadata を owner として扱ってはいけない。

## 影響

- BTreeMap regressions can pass or fail only through an exit code, making Rust and selfhost runner output compatibility and collection behavior diffs harder to diagnose.
- Resource IR owner summary が metadata scalar を owner と誤分類すると、owner-preserving helper や collection rollback path が false leak になり、Stage 6 の `RegionToken` owner model を実用テストに接続できない。
- false positive を stdlib module allowlist で避けると、静的検査が source/type/IR proof ではなく個別許可へ戻るため、今回の修正対象にはしない。

## 修正方針

- BTreeMap focused doctests を canonical `TestReport` stdout reports with `exit_code:` metadata へ移行し、`ret:` / `checks_exit_code` 退行を source policy contract で拒否する。
- SourceCapability unified proof に explicit generic constructor evidence を追加し、`BTreeMap<i32, DropPayload>` のような型適用付き constructor evidence を同じ proof collector で収集する。
- Resource IR owner summary の `i32` leaf 列挙を、condition/value summary 用と raw owner candidate 用へ分離する。
- 型構造から owner token を含む aggregate を判定し、owner-token-backed aggregate の free obligation candidate は `RegionToken.raw` に限定する。stdlib module 名や BTreeMap 固有名の allowlist は使わない。
- `MemPtr<T>` parameter 由来の typed raw load は、外部から渡された non-owning raw cell として Resource IR initialized check に seed する。

## 検証

- `cargo fmt -p nepl-core --check`
- `cargo test -p nepl-core resource_ir_owner_check_transfers_nested_btree_insert_error_owner_through_helper -- --exact --nocapture`
- `cargo test -p nepl-core resource_ir_owner_summary_does_not_treat_plain_i32_struct_fields_as_owners -- --exact --nocapture`
- `cargo test -p nepl-core resource_ir_owner_check_reinitializes_self_update_aggregate_return -- --exact --nocapture`
- `cargo test -p nepl-core resource_ir_owner_check_moves_result_payload_field_owner_to_match_bind -- --exact --nocapture`
- `cargo test -p nepl-core resource_ir_owner_summary_consumes_owned_err_payload_from_unreachable_arm -- --exact --nocapture`
- `cargo test -p nepl-core resource_ir_owner_check_rejects_region_ptr_raw_owner_return -- --exact --nocapture`
- `node nodesrc/test_resource_checker_responsibility.js`
- `node nodesrc/test_static_check_boundary_responsibility.js`
- `node nodesrc/test_stdlib_btreemap_report_contract.js`
- `node nodesrc/test_stdlib_btree_borrowed_observers.js`
- `trunk build`
- `node nodesrc/tests.js -i stdlib/tests/btreemap.n.md --no-tree -o tmp/agent1-btreemap-report-tests.json -j 1 --dist web/dist --assert-io`

## 2026-05-16 Agent 1 修正

`stdlib/tests/btreemap.n.md` の focused doctest 5 件を、`neplg2:test[stdio, normalize_newlines]`、`exit_code: 0`、deterministic `stdout:` を持つ canonical `TestReport` fixture へ移行した。`nodesrc/test_stdlib_btreemap_report_contract.js` は、同 fixture が `ret:` / `checks_exit_code` に戻らず、`test_report_new` / `test_report_print_stdout` / `test_report_exit_code` を使うことを固定する。

stdout 化で露出した `resource.owner.maybe_leak` は、BTreeMap 固有の例外ではなく Resource IR owner summary の分類誤りだった。`owner_summary_i32_condition_leaf.rs` と `owner_summary_raw_i32_leaf.rs` へ分離し、条件・metadata scalar と raw free-obligation candidate を同じ列挙器で扱わないようにした。さらに `owner_summary_owner_token_type.rs` で型構造から owner token を含む aggregate を判定し、その場合の free obligation candidate は `RegionToken.raw` だけに限定した。`RegionToken.size`、`Vec.len`、`Vec.cap`、BTreeMap metadata は condition/value scalar としては使えるが、free obligation owner にはしない。

同時に、SourceCapability unified proof へ explicit generic constructor evidence を追加し、型適用付き constructor boundary を domain 別 walker ではなく単一 proof collector で収集するようにした。`MemPtr<T>` parameter 由来の raw load についても、Resource IR initialized check が外部から渡された typed pointer の raw cell を追跡できるように seed を追加した。

この修正は stdlib module 名や BTreeMap 名の allowlist ではない。source AST evidence、型構造、Resource IR summary の三者から owner/token 性質を導き、metadata scalar を owner と誤診断する根本原因を閉じた。
