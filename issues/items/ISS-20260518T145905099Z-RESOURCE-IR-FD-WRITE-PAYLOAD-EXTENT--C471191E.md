---
id: ISS-20260518T145905099Z-RESOURCE-IR-FD-WRITE-PAYLOAD-EXTENT--C471191E
title: "Resource IR fd_write host memory span proof must cross raw aliases and function summaries"
area: core
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-18
updated: 2026-05-18
target: "nepl-core/src/resource/owner_external_io.rs; nepl-core/src/resource/owner_host_memory_summary.rs; nepl-core/src/resource/host_memory_address.rs; nepl-core/src/resource/external_io_iov_layout.rs; nepl-core/src/resource/initialized_alias_origin.rs; nepl-core/tests/resource_ir.rs"
---

# ISS-20260518T145905099Z-RESOURCE-IR-FD-WRITE-PAYLOAD-EXTENT--C471191E: Resource IR fd_write host memory span proof must cross raw aliases and function summaries

## 概要

`fd_write` の readable payload span 証明が、`MemPtr<T>` の raw field 正規化、iovec descriptor cell の alias、関数境界の host memory span summary、scalar length/count origin の区別を跨げず、正しい readable span を `ExternalIoPayloadExtent` として拒否していた。

修正では stdlib/std/stdio の個別 helper を列挙して許可せず、Resource IR の HostMemorySpan contract を汎用 summary requirement として持ち越し、caller 側の raw alias / scalar alias / owner extent facts から再証明する経路を追加した。

## 対象

- `nepl-core/src/resource/host_memory_address.rs`
- `nepl-core/src/resource/owner_host_memory_summary.rs`
- `nepl-core/src/resource/owner_external_io.rs`
- `nepl-core/src/resource/external_io_iov_layout.rs`
- `nepl-core/src/resource/owner_host_memory_span.rs`
- `nepl-core/src/resource/owner_return_apply.rs`
- `nepl-core/src/resource/summary.rs`
- `nepl-core/src/resource/initialized_alias_origin.rs`
- `nepl-core/src/resource/owner_summary_parameters.rs`
- `nepl-core/src/resource/owner_summary_raw_use_walk.rs`
- `nepl-core/tests/resource_ir.rs`

## 根拠

- `cargo test -p nepl-core --test resource_ir resource_ir_owner_check_accepts_stdio_fd_write_scratch_cleanup -- --nocapture` は pass した。stdio scratch cleanup regression で `fd_write` payload readable span を証明できる。
- 新規 regression `resource_ir_owner_check_applies_fd_write_wrapper_iovec_payload_span_summary` は pass した。`fd_write` を wrapper 関数へ移しても、callee summary が caller 側の iovec payload store から再証明される。
- 新規 regression `resource_ir_owner_check_reports_fd_write_wrapper_missing_iovec_payload_store` は pass した。caller が iovec payload pointer を書き込んでいない場合は、関数境界 summary があっても `ExternalIoPayloadExtent` を拒否する。
- 既存 regression `resource_ir_owner_check_fd_write_rejects_iovec_payload_extent_mismatch` は pass した。payload extent mismatch の拒否は維持している。
- initialized 側の iovec input/output regression も pass した。host memory address normalization が owner checker だけでなく initialized checker 側にも一貫して適用される。

## 問題

根本原因は複合的だった。

- Host memory address の引数が `MemPtr<T>` のまま扱われ、raw field への正規化と raw alias canonicalize が checker 間で統一されていなかった。
- iovec descriptor scan が canonical iov address 配下だけを見ており、同じ raw address に対する alias cell に格納された buffer pointer を見落としていた。
- 関数境界 summary が owner transfer / consumed extent 中心で、host memory span requirement を deferred proof として伝播できなかった。
- iov count や length のような scalar argument と address argument の summary が同じ扱いで、descriptor 内部 cell の projection が caller parameter の scalar origin として誤って採用される可能性があった。
- raw memory store による non-owning raw address view のコピーが、free obligation owner の消費と同じ扱いになる経路が残っていた。
- Raw pointer parameter の raw-address alias seed が不足し、callee 内部で `MemPtr<T>` raw field と caller の raw address fact が接続されない場合があった。

## 影響

この問題により、string や slice 由来の non-owning readable view を `fd_write` へ渡す正しいコードが、free obligation owner を持たないという理由で拒否される可能性があった。逆に修正が粗い場合は、iovec payload pointer が未設定の caller まで通してしまう危険があったため、accept / reject の両方向の regression を追加した。

## 修正方針

完了済み。

- `host_memory_address_place` を追加し、host memory address 引数は `MemPtr<T>` raw field へ正規化してから raw alias canonicalize する。
- `OwnerHostMemorySpanRequirement` を summary に追加し、callee で直接証明できない HostMemorySpan は関数境界を越えて caller facts で再証明する。
- Host memory summary の argument は `Address` と `Scalar` に分離し、length/count の scalar summary では raw address projection を採用しない。
- iovec payload scan は iov address の全 raw alias 配下を走査し、alias cell に書かれた descriptor を見落とさない。
- `RawMemoryOp::Store` は non-owning raw address view のコピーを free obligation owner 消費として扱わない。
- Raw pointer parameter の raw-address alias を owner summary / owner check の入口で seed する。
- `RawValueOrigins::copy_stable_origin` は source 自体が temporary でも、source に紐づく stable origin を raw cell へ伝搬する。これにより `data_len -> tmp -> iov[4]` のような store 経路を scalar parameter requirement として要約できる。
- stdlib/std/stdio の個別 whitelist は追加していない。

## 検証

- `cargo check -p nepl-core`: pass
- `cargo test -p nepl-core resource::initialized_alias_origin::tests::copy_stable_origin_follows_temporary_source_origin -- --nocapture`: pass
- `cargo test -p nepl-core --test resource_ir resource_ir_owner_check_accepts_stdio_fd_write_scratch_cleanup -- --nocapture`: pass
- `cargo test -p nepl-core --test resource_ir resource_ir_owner_check_applies_fd_write_wrapper_iovec_payload_span_summary -- --nocapture`: pass
- `cargo test -p nepl-core --test resource_ir resource_ir_owner_check_reports_fd_write_wrapper_missing_iovec_payload_store -- --nocapture`: pass
- `cargo test -p nepl-core --test resource_ir resource_ir_owner_check_fd_write_rejects_iovec_payload_extent_mismatch -- --nocapture`: pass
- `cargo test -p nepl-core --test resource_ir resource_ir_cell_check_fd_write_accepts_initialized_iovec_buffer -- --nocapture`: pass
- `cargo test -p nepl-core --test resource_ir resource_ir_cell_check_fd_write_reports_uninitialized_iovec_buffer -- --nocapture`: pass
- `cargo test -p nepl-core --test resource_ir resource_ir_cell_check_fd_read_reports_uninitialized_iovec_descriptor -- --nocapture`: pass
- `cargo fmt --all`: pass
- `trunk build`: pass
- `node nodesrc/tests.js -i stdlib/neplg2/core/check/module.nepl -i stdlib/neplg2/core/check/checker.nepl -i stdlib/neplg2/core/pipeline.nepl -i tests/stdlib/neplg2_checker.n.md --no-tree -o tmp/agent1-fd-write-readable-extent.json -j 1 --dist web/dist --assert-io`: total=5, passed=5
- `node nodesrc/issues.js check`: pass
