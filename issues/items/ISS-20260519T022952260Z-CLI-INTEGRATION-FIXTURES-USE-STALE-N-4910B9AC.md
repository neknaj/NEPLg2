---
id: ISS-20260519T022952260Z-CLI-INTEGRATION-FIXTURES-USE-STALE-N-4910B9AC
title: "CLI integration fixtures and Resource IR host-memory proof drift from current std APIs"
area: core
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-19
updated: 2026-05-19
target: "nepl-cli/tests/cli_output.rs; nepl-core/src/resource/host_memory_contract.rs; nepl-core/src/resource/initialized*.rs; nepl-core/src/resource/owner*.rs; nepl-core/src/resource/lower_raw_address*.rs; nepl-core/tests/resource_ir.rs; stdlib/std/fs/dir/read_fd.nepl"
---

# ISS-20260519T022952260Z-CLI-INTEGRATION-FIXTURES-USE-STALE-N-4910B9AC: CLI integration fixtures and Resource IR host-memory proof drift from current std APIs

## 概要

Rust CLI integration tests still generated NEPL source with old nullary function references and direct core/mem raw load/store calls. Updating them to current public stdlib APIs exposed real Resource IR host-memory proof gaps around WASI path and directory operations: direct host input/output spans, counted host output spans, non-owning string/region views, and transparent raw-address wrapper lowering were not all proved from source/IR facts.

## 対象

- `nepl-cli/tests/cli_output.rs`
- `nepl-core/src/resource/host_memory_contract.rs`
- `nepl-core/src/resource/initialized*.rs`
- `nepl-core/src/resource/owner*.rs`
- `nepl-core/src/resource/lower_raw_address*.rs`
- `nepl-core/src/resource/coverage_hir*.rs`
- `nepl-core/tests/resource_ir.rs`
- `stdlib/std/fs/dir/read_fd.nepl`

## 根拠

- GitHub Actions `26070180460` の `rust-test` job で `nepl-cli/tests/cli_output.rs` が失敗していた。
- `check_uses_explicit_stdlib_root` / `check_uses_env_stdlib_root` は、custom stdlib の `answer <()->i32>` を `answer` と値参照しており、現行 resolver では `answer ()` という明示 call が必要である。
- WASI filesystem / stdio fixture は ordinary source から `core/mem` の raw `store_u8` / `store_i32` / `load_*` を直接使っていた。現行 `core/mem` facade は raw helper を再公開しないため `type.overload.no_match` になり、`core/mem/raw` を直接 import して通すのも raw-memory boundary を弱める方向なので不適切である。
- stdlib を通る CLI integration test では `resolve.shadow.outer_definition` warning が出ることがある。warning は表示されるべきだが、CLI の run/check/test が成功している場合に integration test 自体を止める根拠にはしない。
- `std/fs` API へ移すと、`path_open` の host-memory contract が WASI ABI 引数を 1 つずれて解釈していたため、`path_ptr` ではなく `dirflags` を入力 span として検査していた。
- `fd_readdir` は host が実際に初期化した byte 数を `used_ptr` へ返す counted output だが、initialized proof が capacity 全体と used prefix を区別できず、`buf + off` のような affine raw address alias も source-derived scalar fact から十分に復元できていなかった。
- `path_open(mem_ptr_addr(string_data_ptr path), len path, ...)` は `str` 由来の non-owning input view であり、initialized checker が byte range を証明すればよい。一方 owner checker は free obligation owner を要求しており、host input の読み取り権限と free obligation ownership を混同していた。
- `io_bytebuf_region_ptr(region)` のような薄い wrapper が `region_ptr` を返す場合、Resource IR lowering は `RegionPtr` の transparent return を raw-address view として落としていなかった。また HIR coverage は `&local` のように deref 不要な実引数まで deref projection として過剰計上していた。

## 問題

古い CLI fixture を raw memory 直書きのまま残すと、ordinary source が public stdlib API を通る現在の設計を検証できない。さらに、fixture を stdlib API へ移した時に現れた Resource IR の誤拒否は、stdlib 関数名を許可するだけでは直せない。HostMemorySpan、raw alias、scalar flow、initialized range、owner extent、transparent return lowering を、source code と typed Resource IR fact から汎用的に証明する必要がある。

## 影響

Main branch CI can fail before CLI behaviorを確認できないだけでなく、正しい `std/fs` / `std/stdio` 経由のプログラムが `resource.cell.uninit` や `resource.owner.*` で誤拒否される。逆に雑に許可すると、host-visible raw memory span の実際の初期化範囲や owner extent を検査しないまま通す危険がある。

## 修正方針

完了済み。

- CLI integration fixture は explicit nullary call と public `std/stdio` / `std/fs` API を使う。ordinary test source へ raw memory operation authority を戻さない。
- `HostMemorySpan` contract を WASI ABI の typed enum match として修正し、`path_open` は `path_ptr=arg2` / `path_len=arg3` を入力 span として検査する。
- `fd_readdir` は `buf` capacity と `used_ptr` output count を分け、host が返した byte count prefix だけを initialized range として扱う。
- i32 scalar summary / affine raw offset fact は source-derived call graph から構築し、`add off 4` のような式で source value が既知でも symbolic offset fact を落とさない。
- owner checker は input-only non-owning raw view を free obligation owner と混同しない。ただし tracked owner state がある alias は従来通り owner extent proof を要求する。
- `region_ptr` transparent wrapper return を Resource IR の non-owning raw address view として lowered proof に含め、HIR coverage は実際に deref projection が必要な参照実引数だけを数える。

## 検証

- `cargo check -p nepl-core`: pass
- `cargo test -p nepl-core same_call_pointer_length_external_io_spans_match_wasi_abi --lib -- --nocapture`: pass
- `cargo test -p nepl-core records_i32_offset_for_symbolic_add_even_when_source_value_is_known --lib -- --nocapture`: pass
- `cargo test -p nepl-core fd_readdir --test resource_ir -- --nocapture`: pass
- `cargo test -p nepl-core path_open --test resource_ir -- --nocapture`: pass
- `cargo test -p nepl-core resource_ir_lowering_preserves_transparent_region_ptr_wrapper --test resource_ir -- --nocapture`: pass
- `cargo test -p nepl-core resource_ir_owner_check_path_open_accepts_non_owning_string_data_ptr --test resource_ir -- --nocapture`: pass
- `cargo test -p nepl-cli path_open --test cli_output -- --nocapture`: pass
- `cargo test -p nepl-cli run_wasi_fd_readdir_returns_stable_directory_entries --test cli_output -- --nocapture`: pass
