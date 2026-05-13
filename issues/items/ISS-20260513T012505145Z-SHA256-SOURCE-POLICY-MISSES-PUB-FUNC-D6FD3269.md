---
id: ISS-20260513T012505145Z-SHA256-SOURCE-POLICY-MISSES-PUB-FUNC-D6FD3269
title: "source policy regressions miss pub declarations and typed cleanup invariants"
area: nodesrc
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-13
updated: 2026-05-13
target: nodesrc/run_source_policy_regressions.js
---

# ISS-20260513T012505145Z-SHA256-SOURCE-POLICY-MISSES-PUB-FUNC-D6FD3269: source policy regressions miss pub declarations and typed cleanup invariants

## 概要

visibility migration 後も `nodesrc/run_source_policy_regressions.js` が全 source policy を監視できる必要があるが、複数の検査が旧式の `fn` / `enum` / `struct` 宣言だけを抽出していた。そのため `pub fn` / `pub enum` / `pub struct` に移行した現在の stdlib / self-host 実装を見つけられず、sha256 以降の静的安全 policy が途中で隠れていた。

## 対象

- `nodesrc/test_stdlib_sha256_no_unsafe_unwraps.js`
- `nodesrc/test_stdlib_sort_merge_no_unsafe_unwraps.js`
- `nodesrc/test_stdlib_btree_insert_no_unsafe_grow_unwraps.js`
- `nodesrc/test_stdlib_no_unsafe_helpers.js`
- `nodesrc/test_stdlib_io_bytebuf_owner_boundary.js`
- `nodesrc/test_stdlib_stdio_ansi_boundary.js`
- `nodesrc/test_stdlib_stdio_print_i32_boundary.js`
- `nodesrc/test_stdlib_streamio_scanner_boundary.js`
- `nodesrc/test_stdlib_streamio_writer_boundary.js`
- `nodesrc/test_stdlib_string_no_unsafe_unwraps.js`
- `nodesrc/test_selfhost_*.js` の top-level 宣言抽出検査
- `stdlib/std/fs/raw/fd_io.nepl`
- `stdlib/std/stdio/read/buffer.nepl`
- `nepl-core/src/resource/owner_entry.rs`
- `nepl-core/src/parser.rs`

## 根拠

- `node nodesrc/run_source_policy_regressions.js` が `sha256_rounds_loop`、self-host HIR / mono / lexer、stdlib string / stdio / streamio の順に `pub` 宣言を見つけられず停止した。
- typed owner cleanup に移行した fs / stdio scratch helper は `dealloc_ptr` の `Err` 分岐が internal invariant だが、旧 source policy はすべての `unreachable` を一律に禁止しており、型付き cleanup invariant と unsafe unwrap を区別していなかった。
- `Resource IR` の checker 分割後に `owner_entry.rs` が responsibility policy の対象外になり、parser の line limit も visibility parsing の変更後に境界を越えていた。

## 問題

source policy は静的検査・所有権境界・self-host model の退行を検出する最後の網として扱っている。検査自体が旧構文に依存すると、実装が安全でも検査が停止するだけでなく、後続の policy 失敗が見えなくなる。

## 影響

- `pub` 宣言移行後の stdlib / self-host safety policy が安定して実行できない。
- unsafe unwrap 禁止 policy が typed cleanup invariant を誤検出し、逆に本当に危険な raw cleanup との区別ができない。
- Resource IR checker の責務分割が policy 上で監視されず、静的検査の大規模修正で再集中が起きても検出できない。

## 修正方針

- top-level 宣言抽出を `pub` の有無を明示的に扱う正規表現へ統一する。
- `enum` / `struct` payload 検査も `pub` 宣言を同じ構文として扱い、variant/payload の網羅性監視を維持する。
- `dealloc_ptr` の `Result::Err` arm に限定して `unreachable` を typed cleanup invariant として許可し、unsafe unwrap helper の一般利用は禁止し続ける。
- fs / stdio の discard helper に doctest を追加し、typed cleanup invariant を文書と実行テストで固定する。
- Resource owner checker の entrypoint を `owner_entry.rs` へ分離し、responsibility policy の対象 module として line limit を明示する。
- parser の import visibility parsing を同じ責務の範囲で整理し、parser/backend responsibility policy を回復する。

## 検証

- `node nodesrc/run_source_policy_regressions.js`
- `node nodesrc/test_selfhost_hir_range_payload.js`
- `node nodesrc/test_selfhost_mono_instance_absence.js`
- `node nodesrc/test_selfhost_lexer_raw_mode_directive_enum.js`
- `node nodesrc/test_stdlib_string_no_unsafe_unwraps.js`
- `node nodesrc/test_stdlib_stdio_ansi_boundary.js`
- `node nodesrc/test_stdlib_streamio_scanner_boundary.js`
- `node nodesrc/test_resource_checker_responsibility.js`
- `node nodesrc/test_parser_backend_responsibility_policy.js`
- `node nodesrc/tests.js -i stdlib/std/fs/raw/fd_io.nepl --no-tree -o tmp/agent1-fs-discard-doc.json -j 1 --dist web/dist`
- `node nodesrc/tests.js -i stdlib/std/stdio/read/buffer.nepl --no-tree -o tmp/agent1-stdio-discard-doc.json -j 1 --dist web/dist`
