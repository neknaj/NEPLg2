---
id: ISS-20260430T012045983Z-RESOURCE-IR-CANNOT-SUMMARIZE-RETURNE-2FDA4B38
title: "Resource IR cannot summarize returned raw headers with length-guarded dynamic ranges"
area: core
status: open
resolved: false
priority: P2
type: architecture
created: 2026-04-30
updated: 2026-05-06
target: "nepl-core/src/resource/initialized_summary*.rs, nepl-core/src/resource/initialized_external_io*.rs, nepl-core/src/resource/initialized_raw_memory.rs, nepl-core/tests/kp.rs"
source: doc/neplg2/static_check_complexity_reduction_plan.md#stage-4-resource-check-への移行
---

# ISS-20260430T012045983Z-RESOURCE-IR-CANNOT-SUMMARIZE-RETURNE-2FDA4B38: Resource IR cannot summarize returned raw headers with length-guarded dynamic ranges

## 概要

Resource IR initialized-cell summaries can propagate returned raw header fields and unknown-offset initialized Copy cells, but they still cannot express a dependent invariant such as header.buf plus offsets below header.len are initialized after a loop that repeatedly fd_read's into buf + len.

## 対象

- `nepl-core/src/resource/initialized_summary*.rs, nepl-core/src/resource/initialized_external_io*.rs, nepl-core/src/resource/initialized_raw_memory.rs, nepl-core/tests/kp.rs, tests/stdlib/kp.n.md`

## 根拠

- `ISS-20260429T234223901Z-KP-RAW-MEMORY-TESTS-FAIL-RESOURCE-IR-9B5964DA` では direct `fd_read`、単発 read の returned header、`fill_i32` 済み prefix buffer の dynamic-offset read は通るようになった。
- 一方で full scanner style の loop は `write_ptr = add buf len` に対して `fd_read` を繰り返し、最後に `sc` header に `buf` / `len` / `cap` を詰めて返す。
- 現在の `initialized_return` summary は「raw cell が initialized」「raw cell の値が raw address」という事実を返せるが、「`header.len` 未満の offset は `header.buf` から initialized」という dependent fact を表現しない。
- そのため full scanner loop を source-level regression として戻すには、単なる alias rekey ではなく range owner、長さ field、loop write の関係を Resource IR の型付き summary として持つ必要がある。

## 問題

Resource IR initialized-cell summaries can propagate returned raw header fields and unknown-offset initialized Copy cells, but they still cannot express a dependent invariant such as header.buf plus offsets below header.len are initialized after a loop that repeatedly fd_read's into buf + len.

## 影響

A full scanner-style grow/read loop must be reduced or tracked outside Resource IR instead of being proven by the compiler. Leaving this implicit would hide a static-checking completeness gap for self-host input scanners.

## 修正方針

Design a typed range-summary model for returned raw headers: connect the pointer field, the length/capacity fields, and loop writes to dynamic offsets as a single initialized range fact without weakening RawMemoryLoadCell strictness.

具体的には、`initialized_return` の raw cell list だけではなく、次の関係を表す summary を追加する。

- pointer field: returned header のどの raw cell が buffer pointer か。
- length field: どの raw cell が initialized upper bound を表すか。
- capacity field: storage boundary と realloc 後の有効領域を表すか。
- write source: `fd_read` / copy / fill がどの dynamic offset range を initialized にしたか。

この summary は `load_u8 add buf i` のような caller 側の raw load を無条件に通すものではない。guard condition または Resource IR の condition fact により `i < len` が証明できる場合だけ initialized range として扱う。

## 検証

Add a source-level scanner regression that returns a header after a loop of fd_read/realloc and then reads bytes guarded by len. Keep direct fd_read and single-read returned-header regressions passing.

## 2026-05-06 現状確認

現在の実装では、古い `initialized_return.rs` は `initialized_summary*.rs` へ分割済みである。単発の returned raw header regression は `resource_ir_cell_check_returned_raw_header_preserves_initialized_pointee` で通過するが、full scanner style の source-level regression は range summary の診断へ到達する前に Stage 5 effect gate で停止する。

確認した再現:

- `cargo test -p nepl-core --test resource_ir resource_ir_cell_check_returned_raw_header_preserves_initialized_pointee -- --nocapture`: passed
- `cargo test -p nepl-core --test kp local_scanner_new_logic_debug -- --nocapture`: `concat_result` / `from_u128_radix` / `len__str` / `string_finish_base` など raw-memory backed pure stdlib helper の `UnsafeMemoryInPureFunction` / `PureCallsImpure` で compile failure

この issue は引き続き open とする。理由は、既存の単発 returned-header summary は十分ではなく、header pointer field / len field / initialized byte range の dependent relation を型付き summary として表す必要が残るためである。ただし full scanner regression を authoritative に戻すには、先に `ISS-20260427T132406497Z-CORE-MEM-RAW-MEMORY-OPERATIONS-BYPAS-DC67DF04` 側で raw-memory-backed stdlib helper の effect boundary を整理する必要がある。

## 2026-05-06 wasm doctest 追加確認

`trunk build` 後に `node nodesrc/tests.js -i tests/stdlib/kp.n.md -o output/kp_alloc_string_raw_boundary.json --runner wasm --no-tree -j 1 --assert-io` を実行したところ、7 件すべて compile failure になった。

先頭の blocker は `ISS-20260506T123740149Z-STDLIB-RAW-MEMORY-BACKED-SCANNER-AND-338A3B52` として切り出した `alloc/io.nepl` / `alloc/string/utf8.nepl` / `std/text.nepl` / `std/streamio/scanner/state.nepl` の Stage 5 raw-memory boundary 未整理である。

ただし doctest#3 では effect blocker の奥に、`pref` の dynamic-offset prefix buffer read が `resource.cell.possibly_moved` / `resource.cell.uninit` として残っていることも確認した。これはこの issue の本体である returned / dynamic range initialized summary 不足に該当するため、Stage 5 の追加 blocker を取り除いた後に `tests/stdlib/kp.n.md` を authoritative source-level regression として再実行する。

## 2026-05-06 Stage 5 blocker 解消後の再確認

`ISS-20260506T123740149Z-STDLIB-RAW-MEMORY-BACKED-SCANNER-AND-338A3B52` で byte/scanner helper の `effect.pure.calls_impure` blocker を解消した後、`tests/stdlib/kp.n.md::doctest#3` は引き続き `pref` の dynamic-offset prefix buffer read で `resource.cell.possibly_moved` / `resource.cell.uninit` になる。

この結果により、Stage 5 の raw-memory boundary ではなく、この issue が追跡する dynamic range initialized summary が次の compile blocker として残っていることを確認した。owner leak と float timeout は別 issue に分離し、この issue は `pref` の `store_i32 add pref mul i 4` で初期化した range を `load_i32 add pref left_off/right_off` の guard と結び付ける Resource IR summary を対象に継続する。

## 2026-05-06 fs/stdio owner 修正後の再確認

`ISS-20260506T130126516Z-RESOURCE-OWNER-SUMMARIES-REJECT-FS-A-7E58243F` の修正後、`tests/stdlib/kp.n.md` の fs/stdio read scratch owner leak は消えたが、doctest#3 は引き続き `pref` の dynamic range read で停止している。

確認結果:

- doctest#1/#2/#4 は passed。
- doctest#3 は `pref` の `resource.cell.possibly_moved` / `resource.cell.uninit`。
- doctest#5/#6 は stdout を出して passed したが、約 56-59 秒で performance residual が残る。
- doctest#7 は `unwrap_ok dealloc` 経由の raw owner consumption が見えない別 issue として `ISS-20260506T134653279Z-RESOURCE-OWNER-SUMMARY-MISSES-RAW-DE-007EB7EA` に分離した。

この issue の範囲は引き続き、guarded dynamic offset と initialized range fact を Resource IR summary に型付きで表現することである。

## 2026-05-06 string boundary 修正後の再確認

`ISS-20260506T135746003Z-STRING-ACCESS-SPLIT-LOSES-RAW-MEMORY-8C64A912` の修正後、`tests/stdlib/kp.n.md::doctest#3` は `len__str` の effect blocker ではなく、再び `pref` の `resource.cell.possibly_moved` / `resource.cell.uninit` で停止した。

これにより、この issue が tracking している dynamic initialized range summary が KP doctest#3 の本体 blocker として残っていることを再確認した。

## 2026-05-06 unwrap_ok dealloc 修正後の再確認

`ISS-20260506T134653279Z-RESOURCE-OWNER-SUMMARY-MISSES-RAW-DE-007EB7EA` の修正後に `node nodesrc/tests.js -i tests/stdlib/kp.n.md -o output/kp_after_unwrap_ok_dealloc_summary.json --runner wasm --no-tree -j 1 --assert-io` を再実行した。

結果は total=7, passed=4, failed=1, errored=2 で、doctest#7 の owner leak は消えた。一方 doctest#3 は引き続き `pref` の dynamic-offset prefix buffer read で `resource.cell.possibly_moved` を出しているため、この issue の dynamic initialized range summary 残件は継続する。

## 2026-05-06 KP doctest#3 source discipline の切り分け

`tests/stdlib/kp.n.md::doctest#3` は `store_i32 pref 0` のみで prefix buffer 全体の初期化を loop induction と入力制約へ暗黙依存していた。source 上に `l/r` の範囲 guard や typed range contract がないため、これを compiler 側で通すと dynamic offset を過剰に initialized 扱いする危険がある。

この doctest の書き方問題は `ISS-20260506T145720311Z-KP-PREFIX-SUM-DOCTEST-RELIES-ON-IMPL-5F1F3821` に分離し、Rust KP regression と同じ `fill_i32 pref pref_len 0` へ揃えた。したがって、この issue は doctest#3 そのものではなく、明示 guard / typed range fact を持つ source に対する将来の Resource IR dynamic range summary として継続する。

`node nodesrc/run_doctest.js -i tests/stdlib/kp.n.md -n 3 --dist web/dist` では doctest#3 が passed になった。full KP run では remote main の `alloc/string/integer.nepl` split に伴う `from_u128_radix` boundary miss が新たに出たため、これは `ISS-20260506T150445017Z-STRING-INTEGER-SPLIT-LOSES-RAW-MEMOR-36A59D71` として分離した。
