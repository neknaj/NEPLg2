---
id: ISS-20260430T012045983Z-RESOURCE-IR-CANNOT-SUMMARIZE-RETURNE-2FDA4B38
title: "Resource IR cannot summarize returned raw headers with length-guarded dynamic ranges"
area: core
status: open
resolved: false
priority: P2
type: architecture
created: 2026-04-30
updated: 2026-05-07
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

## 2026-05-06 dynamic raw address view origin 部分対応

`ISS-20260506T172012873Z-RESOURCE-IR-DYNAMIC-RAW-ADDRESS-VIEW-77E94B53` として、dynamic range summary 以前に発生していた stable local origin の欠落を分離して修正した。

`fill_i32 pref pref_len 0` は `pref[+?].deref` の Copy cell を initialized にするが、後続の `add pref prev_off` は別の `pref` read から `tmp[+?]` を作る。既存 `ValueOrigin` は exact `tmp -> %pref` だけを解決し、projection suffix を origin 側へ戻せなかったため、`tmp[+?].deref` が `Uninit` になっていた。

今回の修正で `tmp[+?]` は `%pref[+?]` へ正規化される。これは通常 i32 copy を raw alias group として seed しないため、deep-prefix compile-time regression を再発させずに dynamic initialized Copy range の false positive を解消する。

確認結果:

- `cargo test -p nepl-core --test resource_ir resource_ir_cell_check_preserves_dynamic_fill_origin_across_local_reads -- --nocapture`: passed
- `cargo test -p nepl-core --test kp kpread_to_kpwrite_prefixsum_i32 -- --nocapture`: `pref` dynamic range の `resource.cell.uninit` は解消。次の別件として fs/stdio scratch dealloc の `resource.owner.no_free_obligation` が発火したため、`ISS-20260506T172100644Z-FS-AND-STDIO-SCRATCH-RAW-DEALLOC-LOS-57895CB4` に分離した。

この親 issue は引き続き open とする。今回の修正は projected raw address view の origin propagation であり、length field / guard condition / initialized range を一体で表す dependent summary model はまだ未完了である。

## 2026-05-07 relation guard fact 部分対応

`ISS-20260506T201433509Z-RESOURCE-CONDITION-FACTS-DROP-NONZER-5EE6B7A6` として、range summary の前提になる nonzero relational guard を Resource IR に残す対応を分離して修正した。

これまで `ResourceConditionFact` は `lt 0 x` / `le x 0` のような zero/one comparison だけを単項 fact として保持し、`lt i len` は fact なしになっていた。そのため、将来の returned raw header summary が `i < header.len` を要求しても、compiler は typed guard を参照できなかった。

今回の対応で `ResourceConditionFact::I32Relation` と `ResourceI32RelationOp` を追加し、`lt i len` が `I32Relation { left: i, op: Lt, right: len }` として lowering / dump される。これはまだ dynamic initialized range を証明する本体ではないが、length field / guard condition / initialized range をつなぐための typed precondition である。

この親 issue は引き続き open とする。残件は、relation fact と symbolic raw offset を結び、`i < len` が証明された場合だけ `base + i` の initialized range を許可する model を実装することである。

## 2026-05-07 symbolic raw offset 部分対応

`ISS-20260506T202600181Z-RESOURCE-RAW-OFFSETS-ERASE-SYMBOLIC--E5DDB5A0` として、dynamic raw address offset が `ResourceOffset { bytes: None }` に潰れていた問題を分離して修正した。

`ResourceOffset` は `Known(usize)` / `Symbolic { place }` / `Unknown` の enum になり、`RawAddressOffset` も simple dynamic offset place を `Symbolic` として保持する。`mem_ptr_add ptr idx` のような raw view は `base[+symbolic]` として Resource IR に残るため、relation fact と offset identity を後続 summary が参照できる。

一方で、この変更は dynamic offset を安全とみなす checker 緩和ではない。general overlap 判定では `Symbolic` / `Unknown` を may-overlap として扱うため、memory safety は保守的なまま維持する。

この親 issue は引き続き open とする。残件は、`I32Relation` と `ResourceOffset::Symbolic` を照合し、loop / branch summary 上で initialized byte range を typed fact として伝播する本体実装である。

## 2026-05-07 i32 relation fact store 部分対応

`ISS-20260506T203942617Z-RESOURCE-BRANCH-PATHS-DO-NOT-RETAIN--4242E13D` として、`ResourceConditionFact::I32Relation` が branch path の fact store に保存されていない問題を分離して修正した。

`I32AliasFacts` は value / unary condition 専用のまま維持し、二項関係は別の `I32RelationFacts` に分離した。truthy branch は relation をそのまま保存し、false branch は negated op として保存する。copy / merge / clear でも relation fact が追従するため、後続の initialized range summary は HIR 条件式を再走査せず Resource IR state へ問い合わせられる。

この親 issue は引き続き open とする。残件は、保存済み relation fact と symbolic raw offset を実際に照合し、guarded initialized range を cell availability 判定へ接続することである。

## 2026-05-07 initialized branch fact 部分対応

`ISS-20260506T210407334Z-INITIALIZED-RESOURCE-BRANCH-PATHS-DO-F88296F7` として、owner checker では保存される typed condition fact が initialized checker の branch path へ反映されていない問題を分離して修正した。

`initialized_control.rs` は then / else path を clone した直後に `record_condition_fact_value_constraints` を実行し、その後に既存の realloc condition handling を適用する。これにより `ResourceConditionFact::I32Relation` は initialized cell availability 側の `RawCellAddressAliases` からも query 可能になる。

この親 issue は引き続き open とする。残件は、initialized checker が保持できるようになった relation proof と `ResourceOffset::Symbolic` を、raw memory load の availability 判定へ安全に接続することである。

## 2026-05-07 string char slice source reservation 追記

`ISS-20260506T155757405Z-STRING-FLOAT-AND-CHAR-BUILDER-OWNER--37EDC044` の focused run で、`tests/stdlib/string_char.n.md::doctest#1` が `str_slice_chars_result s 1 3` の成功後に同じ source `s` を読む箇所で `resource.owner.reserved` になった。

確認内容:

- `str_slice_chars_result` は char index を byte offset へ変換し、最終的に `str_slice_result` / `string_from_mem_unchecked_result` で新しい `str` 領域へ copy する。
- caller から見ると source `str` は `Copy` view であり、slice 結果は新規確保された `str` なので、source `s` が reserved のまま残るのは API 利用側の所有権違反ではない。
- したがってこの残件は `string_char.n.md` の順序だけを変えて隠すのではなく、returned raw header / copied string source view の summary が source `str` を予約したままにしないことを Resource IR 側で表現する必要がある。

## 2026-05-07 symbolic Copy store 部分対応

`ISS-20260506T211740745Z-SYMBOLIC-COPY-STORES-ERASE-UNKNOWN-O-0BD91F6C` として、symbolic offset store が unknown-offset initialized Copy fact を過剰に消す問題を分離して修正した。

`RawMemoryOp::Store` は store 専用の typed clearing を使う。overlap する raw cell fact でも、既存 fact が initialized Copy で stored value と同じ Copy 型として扱える場合は保持し、non-Copy / moved / uninit state は従来どおり保守的に消す。

この親 issue は引き続き open とする。`kpread_to_kpwrite_prefixsum_i32` はなお `pref[+symbolic].deref` の `Cell(Uninit)` で失敗するため、残件は loop condition fact と guarded initialized range summary の接続である。

## 2026-05-07 loop condition fact 部分対応

`ISS-20260506T212446487Z-RESOURCE-LOOPS-DO-NOT-CARRY-TYPED-CO-FD0086F2` として、`ResourceOp::Loop` が typed condition fact を持たない問題を分離して修正した。

`while lt i len` は now `ResourceConditionFact::I32Relation { left: i, op: Lt, right: len }` として Loop op に保存される。initialized checker と owner checker は condition evaluation 後、loop body path に truthy fact、exit path に false/negated fact を適用してから各 path を検査する。

この親 issue は引き続き open とする。今回の修正で loop body から `i < len` を Resource IR state として参照できるようになったが、まだ symbolic raw offset と initialized range fact を照合して raw load availability へ接続する本体 model は未完了である。

## 2026-05-07 RawAddressView authority 部分対応

`ISS-20260506T215615927Z-RESOURCE-RAWADDRESSVIEW-TREATS-ORDIN-B3C620DA` として、`RawAddressView` が通常の `i32` arithmetic を無条件に raw pointer として昇格する問題を分離して修正した。

`RawAddressView` は lowering 上 `add` / `sub` から広めに生成されるため、checker 側では既存の raw-address proof が必要である。今回の対応で、alias table の exact/prefix proof、initialized checker の raw cell / raw storage proof、owner checker の owner / storage-origin proof がある場合だけ view を伝播し、scalar `ValueOrigin` だけでは raw alias を seed しない。

また、storage-offset view を local に束縛しただけで `pref[+?].deref` の broad initialized fact を view local 側へ rekey しないようにした。view は non-owning pointer expression であり、base storage fact の所有 canonical ではない。

確認として `resource_ir_cell_check_preserves_dynamic_fill_across_impure_i32_reads` を追加し、`fill_i32 pref pref_len 0` の後に unrelated impure `i32` arithmetic を挟んでも `load_i32 add pref off` が通ることを確認した。`kpread_to_kpwrite_prefixsum_i32` も pass しており、この経路の `pref` dynamic range blocker は解消した。

この親 issue は引き続き open とする。残件は、明示 guard / returned header / length field をまたぐ dependent initialized range summary を Resource IR の typed model として表現することである。

## 2026-05-07 KP unique/count fixture 初期化 contract 整理

`ISS-20260507T010031891Z-KP-UNIQUE-COUNT-FIXTURE-LACKS-EXPLIC-85D146AF` として、`tests/stdlib/kp.n.md::doctest#7` が direct post-unique loop で `resource.cell.uninit` になる fixture 側 blocker を分離して修正した。

この doctest は fixed-offset stores で 6 要素を初期化した後、`unique_sorted_i32` の戻り値 `new_len` を上限に dynamic offset load を行っていた。現行 Resource IR は exact offset store と `new_len <= len` の関係を dependent range summary として結び付けないため、後続 loop の `load_i32 ptr` は `RawMemoryLoadCell Uninit` になる。

fixture では配列全体を `fill_i32 data len 0` で initialized Copy range にしてから exact value を上書きする形へ更新した。これは runtime semantics を変えず、`RawMemoryLoadCell` を緩めず、現在の Resource IR が要求する source-level range contract を明示する修正である。

確認結果:

- `node nodesrc/tests.js -i tests/stdlib/kp.n.md --no-tree --dist web/dist -o tmp/kp_agent1_after_unique_range_init_default_timeout.json -j 1 --assert-io`: total=7, passed=7, failed=0, errored=0

この親 issue は引き続き open とする。残件は、明示的な fill に依存しない returned header / length field / guard relation をまたぐ dependent initialized range summary を typed model として実装することである。
