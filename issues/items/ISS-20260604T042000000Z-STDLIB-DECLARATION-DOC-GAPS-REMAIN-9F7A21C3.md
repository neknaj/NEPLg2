---
id: ISS-20260604T042000000Z-STDLIB-DECLARATION-DOC-GAPS-REMAIN-9F7A21C3
title: "stdlib declaration documentation gaps remain high after baseline refresh"
area: stdlib
status: open
resolved: false
priority: P1
type: doc
created: 2026-06-04
updated: 2026-06-06
target: "stdlib/core, stdlib/alloc, stdlib/std"
---

# ISS-20260604T042000000Z-STDLIB-DECLARATION-DOC-GAPS-REMAIN-9F7A21C3: stdlib declaration documentation gaps remain high after baseline refresh

## 概要

`nodesrc/test_stdlib_documentation_contract.js` の current baseline を再集計した時点で、stdlib は `declarationNoDoc=361`、`declarationNoDoctest=1690`、`publicDeclarationNoDoctest=1531` を持つ。これは Zenn 記事の「契約、現状実装、enum の場合分け、計算量、simple/typical example、doc test」を doc comment に書く方針に対して未達である。

2026-06-05 の BitSet slice で `stdlib/alloc/collections/bitset` の facade / type / layout / storage / mutation / diagnostic helper docs と report doctest を追加し、baseline は `moduleNoDoctest=303`、`declarationNoDoc=350`、`declarationNoDoctest=1686`、`publicDeclarationNoDoctest=1527` まで改善した。

同日の AdjacencyMatrix slice で `stdlib/alloc/collections/adjacency_matrix` の facade / type / storage / mutation / diagnostic / observer / update / bulk / cleanup docs と report doctest を追加し、baseline は `moduleNoDoctest=301`、`declarationNoDoc=343`、`declarationNoDoctest=1679`、`publicDeclarationNoDoctest=1520` まで改善した。ただし binary_heap / bloom_filter / btree などに declaration doc gap が残るため、この issue は open のまま継続する。

同日の BinaryHeap slice で `stdlib/alloc/collections/binary_heap` の facade / type invariant / pop result / observer / pop API / storage helper / order helper docs と report doctest を追加し、baseline は `moduleNoDoctest=299`、`declarationNoDoc=332`、`declarationNoDoctest=1671`、`publicDeclarationNoDoctest=1512` まで改善した。ただし bloom_filter / btree などに declaration doc gap が残るため、この issue は open のまま継続する。

同日の BloomFilter slice で `stdlib/alloc/collections/bloom_filter` の facade / type invariant / hash helper / layout helper / storage helper / mutation helper / public API docs と report doctest を追加し、baseline は `moduleNoDoctest=297`、`declarationNoDoc=318`、`declarationNoDoctest=1670`、`publicDeclarationNoDoctest=1511` まで改善した。ただし btreemap / btreeset / counting_bloom_filter などに declaration doc gap が残るため、この issue は open のまま継続する。

同日の CountingBloomFilter slice で `stdlib/alloc/collections/counting_bloom_filter` の facade / type invariant / hash helper / storage helper / mutation helper / public API docs と report doctest を追加し、baseline は `moduleNoDoctest=295`、`declarationNoDoc=306`、`declarationNoDoctest=1668`、`publicDeclarationNoDoctest=1509` まで改善した。ただし btreemap / btreeset / disjoint_set などに declaration doc gap が残るため、この issue は open のまま継続する。

同日の BTreeMap slice で `stdlib/alloc/collections/btreemap/search.nepl` と `storage.nepl` の search / owner-backed storage helper docs と report doctest を追加し、baseline は `moduleNoDoctest=295`、`declarationNoDoc=287`、`declarationNoDoctest=1668`、`publicDeclarationNoDoctest=1509` まで改善した。ただし btreeset / disjoint_set / fenwick などに declaration doc gap が残るため、この issue は open のまま継続する。

同日の BTreeSet slice で `stdlib/alloc/collections/btreeset/search.nepl` と `storage.nepl` の search / key-only owner-backed storage helper docs と report doctest を追加し、baseline は `moduleNoDoctest=295`、`declarationNoDoc=272`、`declarationNoDoctest=1668`、`publicDeclarationNoDoctest=1509` まで改善した。ただし disjoint_set / fenwick / segment_tree などに declaration doc gap が残るため、この issue は open のまま継続する。

同日の DisjointSet slice で `stdlib/alloc/collections/disjoint_set/api/diagnostic.nepl`、`storage.nepl`、`query.nepl` の diagnostic / typed storage / borrowed query helper docs と report doctest を追加し、baseline は `moduleNoDoctest=295`、`declarationNoDoc=266`、`declarationNoDoctest=1668`、`publicDeclarationNoDoctest=1509` まで改善した。ただし fenwick / segment_tree / sparse_set などに declaration doc gap が残るため、この issue は open のまま継続する。

同日の Fenwick slice で `stdlib/alloc/collections/fenwick/api/diagnostic.nepl`、`storage.nepl`、`query.nepl`、`mutation.nepl` の diagnostic / typed storage / borrowed prefix query / point update helper docs と report doctest を追加し、baseline は `moduleNoDoctest=295`、`declarationNoDoc=257`、`declarationNoDoctest=1668`、`publicDeclarationNoDoctest=1509` まで改善した。ただし segment_tree / sparse_set / vec などに declaration doc gap が残るため、この issue は open のまま継続する。

同日の SegmentTree slice で `stdlib/alloc/collections/segment_tree/api/diagnostic.nepl`、`layout.nepl`、`storage.nepl`、`range.nepl`、`mutation.nepl` の diagnostic / base layout / typed storage / range traversal / point update helper docs と report doctest を追加し、baseline は `moduleNoDoctest=295`、`declarationNoDoc=244`、`declarationNoDoctest=1668`、`publicDeclarationNoDoctest=1509` まで改善した。ただし sparse_set / vec / diag などに declaration doc gap が残るため、この issue は open のまま継続する。

同日の SparseSet slice で `stdlib/alloc/collections/sparse_set/api/diagnostic.nepl`、`storage.nepl`、`membership.nepl`、`mutation.nepl` の diagnostic / typed dense-sparse storage / membership / insert-remove helper docs と report doctest を追加し、baseline は `moduleNoDoctest=295`、`declarationNoDoc=234`、`declarationNoDoctest=1668`、`publicDeclarationNoDoctest=1509` まで改善した。ただし vec / diag / io などに declaration doc gap が残るため、この issue は open のまま継続する。

2026-06-06 の Vec slice で `stdlib/alloc/collections/vec/invariant.nepl`、`mutation/push.nepl`、`storage/fill.nepl`、`transform/filter/select.nepl` の invariant adapter / push overload / filled constructor / Copy filter docs と report doctest を追加し、baseline は `moduleNoDoctest=295`、`declarationNoDoc=229`、`declarationNoDoctest=1668`、`publicDeclarationNoDoctest=1509` まで改善した。ただし diag / io / string builder などに declaration doc gap が残るため、この issue は open のまま継続する。

同日の Diag slice で `stdlib/alloc/diag/diag.nepl`、`error/diag.nepl`、`error/diags.nepl` の renderer by-value overload / stdio print helper / typed error accessor / `Diags` by-value observer docs と report doctest を追加し、baseline は `moduleNoDoctest=295`、`declarationNoDoc=215`、`declarationNoDoctest=1668`、`publicDeclarationNoDoctest=1509` まで改善した。ただし hash32 / io / string builder などに declaration doc gap が残るため、この issue は open のまま継続する。

同日の Alloc IO slice で `stdlib/alloc/io/bytebuf.nepl`、`bytebuilder/types.nepl`、`traits.nepl` の `ByteBuf` observer / pointer projection / cleanup、`ByteBuilder` pointer projection、stream trait / forwarding helper docs と report doctest を追加した。trait body には現行 NEPLg2 構文上 doc comment を置けないため、`nodesrc/test_stdlib_documentation_contract.js` は trait body method を個別 declaration として数えず、trait declaration doc に contract を集約する形へ修正した。baseline は `declarations=2488`、`declarationNoDoc=162`、`declarationNoDoctest=1662`、`publicDeclarationNoDoctest=1509`、`privateDeclarationNoDoctest=153` まで改善した。ただし hash32 / string builder / string integer などに declaration doc gap が残るため、この issue は open のまま継続する。

同日の Hash32 slice で `stdlib/alloc/hash/hash32.nepl` と `fnv1a32.nepl` の module doc、`mix`、`hash_bytes_loop`、`hash32` primitive / `str` overload、`Fnv1a32` state / constructor / update / finalize docs と report doctest を追加した。`hash_bytes_loop` は `Option::Some` / `Option::None` の byte read boundary を明記し、`hash32 str` は UTF-8 byte 列を hash 対象にする契約を固定した。さらに `sha256_free` が内部 buffer owner を `free` するのに `%fn` だった純粋性不整合を `%impure fn` へ修正し、owner close doctest を追加した。baseline は `moduleNoDoctest=293`、`declarationNoDoc=161`、`declarationNoDoctest=1651`、`publicDeclarationNoDoctest=1498`、`privateDeclarationNoDoctest=153` まで改善した。ただし string builder / string integer / string UTF-8 helper などに declaration doc gap が残るため、この issue は open のまま継続する。

## 対象

- `stdlib/core`
- `stdlib/alloc`
- `stdlib/std`

## 根拠

- `node nodesrc/test_stdlib_documentation_contract.js` の再集計で、current baseline は `files=456`、`declarationNoDoc=361`、`declarationNoDoctest=1690` だった。
- `stdlib/alloc/collections/adjacency_matrix/layout.nepl` の layout helper 5件には doc comment と doctest を追加済みだったが、その後の BitSet / AdjacencyMatrix / BinaryHeap / BloomFilter / CountingBloomFilter / BTreeMap / BTreeSet / DisjointSet / Fenwick slice により sample gaps は segment_tree / sparse_set / vec 系へ進んでいる。
- baseline refresh はこれ以上の悪化を止める regression guard であり、既存 gap を解消したことを意味しない。
- `stdlib/alloc/collections/bitset` では、owner-backed `BitSetUpdateError` を直接構築する doctest を避け、public `insert` / `remove` の Err 経路から error を取得して `bitset_update_error_diag` と `bitset_update_error_owner` の契約を確認する形にした。

## 問題

現状の stdlib は module doc の欠落は 0 だが、declaration 単位では doc comment と doctest が不足している。public API の contract と current implementation が宣言近傍にないため、型だけでは分からない所有権、計算量、error enum の条件、境界条件を利用者や reviewer が確認しにくい。

2026-06-05 時点で、baseline は現在値まで締め直した。これにより既存 gap の悪化は検査で止まるが、`declarationNoDoc=361` と `declarationNoDoctest=1690` はまだ未解決の負債であるため、この issue は open のままとする。宣言検出そのものが減って gap が隠れることを防ぐため、`declarations=2525` も下限として検査する。

同日 BitSet slice 後に baseline を再度締め直した。新しい悪化防止ラインは `moduleNoDoctest=303`、`declarationNoDoc=350`、`declarationNoDoctest=1686`、`publicDeclarationNoDoctest=1527` である。`nodesrc/test_stdlib_bitset_doc_report_contract.js` により、BitSet の report doctest と owner recovery doc contract は total count だけでなく module 固有にも固定する。

同日 AdjacencyMatrix slice 後に baseline を再度締め直した。新しい悪化防止ラインは `moduleNoDoctest=301`、`declarationNoDoc=343`、`declarationNoDoctest=1679`、`publicDeclarationNoDoctest=1520` である。`nodesrc/test_stdlib_adjacency_matrix_doc_report_contract.js` により、AdjacencyMatrix の facade lifecycle、type invariant、typed byte storage、mutation、diagnostic kind、borrowed observer、owner recovery doc contract を module 固有にも固定する。

同日 BinaryHeap slice 後に baseline を再度締め直した。新しい悪化防止ラインは `moduleNoDoctest=299`、`declarationNoDoc=332`、`declarationNoDoctest=1671`、`publicDeclarationNoDoctest=1512` である。`nodesrc/test_stdlib_binary_heap_doc_report_contract.js` により、BinaryHeap の facade lifecycle、type invariant、observer / pop API、`Vec Option .T` storage、index math、swap、sift-up / sift-down、`BinaryHeapPop` owner accessor doc contract を module 固有にも固定する。

同日 BloomFilter slice 後に baseline を再度締め直した。新しい悪化防止ラインは `moduleNoDoctest=297`、`declarationNoDoc=318`、`declarationNoDoctest=1670`、`publicDeclarationNoDoctest=1511` である。`nodesrc/test_stdlib_bloom_filter_doc_report_contract.js` により、BloomFilter の facade lifecycle、type invariant、invalid length error kind、borrowed observer、false positive / false negative contract、hash / layout / storage / mutation helper doc contract を module 固有にも固定する。

同日 CountingBloomFilter slice 後に baseline を再度締め直した。新しい悪化防止ラインは `moduleNoDoctest=295`、`declarationNoDoc=306`、`declarationNoDoctest=1668`、`publicDeclarationNoDoctest=1509` である。`nodesrc/test_stdlib_counting_bloom_filter_doc_report_contract.js` により、CountingBloomFilter の facade lifecycle、type invariant、invalid length error kind、borrowed observer、false positive / false negative、counter saturation / lower-bound remove、typed counter storage、hash / storage / mutation helper doc contract を module 固有にも固定する。

同日 BTreeMap slice 後に baseline を再度締め直した。新しい悪化防止ラインは `moduleNoDoctest=295`、`declarationNoDoc=287`、`declarationNoDoctest=1668`、`publicDeclarationNoDoctest=1509` である。`nodesrc/test_stdlib_btree_search_doc_report_contract.js` と `nodesrc/test_stdlib_btreemap_storage_doc_report_contract.js` により、BTreeMap の lower_bound / is_at、`Vec Option .K` / `Vec Option .V` storage、partial allocation cleanup、owner recovery、grow failure、storage invariant failure、Copy boundary、O(cap) / O(len0) contract を module 固有にも固定する。

同日 BTreeSet slice 後に baseline を再度締め直した。新しい悪化防止ラインは `moduleNoDoctest=295`、`declarationNoDoc=272`、`declarationNoDoctest=1668`、`publicDeclarationNoDoctest=1509` である。`nodesrc/test_stdlib_btree_search_doc_report_contract.js` と `nodesrc/test_stdlib_btreeset_storage_doc_report_contract.js` により、BTreeSet の lower_bound / is_at、`Vec Option .T` key-only storage、`Option::Some key` / `Option::None` slot state、`diag_out_of_memory`、`BTreeSetInsertError` owner recovery、旧 storage free、old last slot clear、storage invariant failure、Copy boundary、O(cap) / O(len0) contract を module 固有にも固定する。

同日 DisjointSet slice 後に baseline を再度締め直した。新しい悪化防止ラインは `moduleNoDoctest=295`、`declarationNoDoc=266`、`declarationNoDoctest=1668`、`publicDeclarationNoDoctest=1509` である。`nodesrc/test_stdlib_disjoint_set_doc_report_contract.js` により、DisjointSet の storage invariant、diagnostic enum kind、typed `Vec i32` storage boundary、`Option::Some` / `Option::None` query contract、path compression なしの borrowed observer、`DisjointSetUpdateError` による owner recovery、union-by-size、O(log n) / O(n) / O(1) contract を module 固有にも固定する。

同日 Fenwick slice 後に baseline を再度締め直した。新しい悪化防止ラインは `moduleNoDoctest=295`、`declarationNoDoc=257`、`declarationNoDoctest=1668`、`publicDeclarationNoDoctest=1509` である。`nodesrc/test_stdlib_fenwick_doc_report_contract.js` により、Fenwick の `n + 1` 1-indexed storage invariant、sentinel cell、diagnostic enum kind、typed `Vec i32` storage boundary、`Option::Some` / `Option::None` prefix query contract、owner-preserving `FenwickAddError`、storage invariant failure で rollback を契約しないこと、O(log n) / O(1) / O(bit_len) contract を module 固有にも固定する。

同日 SegmentTree slice 後に baseline を再度締め直した。新しい悪化防止ラインは `moduleNoDoctest=295`、`declarationNoDoc=244`、`declarationNoDoctest=1668`、`publicDeclarationNoDoctest=1509` である。`nodesrc/test_stdlib_segment_tree_doc_report_contract.js` により、SegmentTree の `base` / `2 * base` storage invariant、`n == 0` でも base は 1、typed `Vec i32` storage boundary、diagnostic enum kind、`Option::Some` / `Option::None` range query contract、owner-preserving `SegmentTreeUpdateError`、storage invariant failure で rollback を契約しないこと、O(log n) / O(1) contract を module 固有にも固定する。

同日 SparseSet slice 後に baseline を再度締め直した。新しい悪化防止ラインは `moduleNoDoctest=295`、`declarationNoDoc=234`、`declarationNoDoctest=1668`、`publicDeclarationNoDoctest=1509` である。`nodesrc/test_stdlib_sparse_set_doc_report_contract.js` により、SparseSet の `[0, n)` domain、`new 0` valid empty set、typed `Vec i32` dense/sparse storage boundary、diagnostic enum kind、`Option::Some` / `Option::None` mutation contract、borrowed membership fail-closed contract、owner-preserving `SparseSetUpdateError`、storage invariant failure で rollback を契約しないこと、O(1) / O(n) contract を module 固有にも固定する。

2026-06-06 Vec slice 後に baseline を再度締め直した。新しい悪化防止ラインは `moduleNoDoctest=295`、`declarationNoDoc=229`、`declarationNoDoctest=1668`、`publicDeclarationNoDoctest=1509` である。`nodesrc/test_stdlib_vec_doc_report_contract.js` により、Vec の storage invariant adapter が enum proof を bool / message へ畳まないこと、Copy / Drop `push` overload の `VecPushRejected .T` owner recovery、`filled` の initialized storage contract、Copy `filter` の input owner recovery と Drop payload との差分、storage invariant failure で rollback を契約しないこと、O(1) / O(n) contract を module 固有にも固定する。

同日 Diag slice 後に baseline を再度締め直した。新しい悪化防止ラインは `moduleNoDoctest=295`、`declarationNoDoc=215`、`declarationNoDoctest=1668`、`publicDeclarationNoDoctest=1509` である。`nodesrc/test_stdlib_diag_doc_report_contract.js` により、Diag の enum authority と表示文字列の分離、by-value `diags_to_string` の `impure fn` owner cleanup、`Diags` by-value observer の borrowed observation + `diags_free`、`diags_has_errors_loop` の `Vec.get` + exhaustive `DiagLevel` match、stdio print helper の IO boundary を module 固有にも固定する。

同日 Alloc IO slice 後に baseline を再度締め直した。新しい悪化防止ラインは `declarations=2488`、`moduleNoDoctest=295`、`declarationNoDoc=162`、`declarationNoDoctest=1662`、`publicDeclarationNoDoctest=1509`、`privateDeclarationNoDoctest=153` である。`nodesrc/test_stdlib_alloc_io_doc_report_contract.js` により、`ByteBufStorage::Empty` / `Owned RegionToken` owner state、非所有 `MemPtr` view、`Option::Some` / `Option::None` pointer and byte access、`ByteBuf` cleanup、`ByteBuilder` pointer projection、stream trait の `StdErrorKind` / `Result` / `impure` boundary、trait body method を個別 doc 対象にしない scanner contract を module 固有にも固定する。

同日 Hash32 slice 後に baseline を再度締め直した。新しい悪化防止ラインは `declarations=2488`、`moduleNoDoctest=293`、`declarationNoDoc=161`、`declarationNoDoctest=1651`、`publicDeclarationNoDoctest=1498`、`privateDeclarationNoDoctest=153` である。`nodesrc/test_stdlib_hash32_doc_report_contract.js` により、Hash32 / FNV-1a の report doctest、signed `i32` bit pattern、非暗号 hash boundary、UTF-8 byte hashing、`Option::Some` / `Option::None` byte read boundary、FNV offset basis / prime / byte range、O(1) / O(n) complexity、`sha256_free` の owner-closing `impure fn` boundary を module 固有にも固定する。

同日 StringBuilder fallback wrapper slice 後に baseline を再度締め直した。新しい悪化防止ラインは `declarations=2488`、`moduleNoDoctest=293`、`declarationNoDoc=154`、`declarationNoDoctest=1651`、`publicDeclarationNoDoctest=1498`、`privateDeclarationNoDoctest=153` である。`nodesrc/test_alloc_string_doc_report_contract.js` により、`string_builder_new`、`sb_append`、`sb_append_char`、`sb_append_ascii`、`sb_append_byte`、`sb_build`、`sb_append_i32` の report doctest、入力 builder owner consumption、`Result::Err` からの空 builder / 空文字列 fallback、ASCII / byte / UTF-8 boundary、O(1) / O(n) / O(total_bytes) complexity を module 固有にも固定する。

同日の UTF-8 / core char helper slice 後に baseline を再度締め直した。新しい悪化防止ラインは `declarations=2488`、`moduleNoDoctest=293`、`declarationNoDoc=145`、`declarationNoDoctest=1651`、`publicDeclarationNoDoctest=1498`、`privateDeclarationNoDoctest=153` である。`nodesrc/test_stdlib_utf8_validation_doc_report_contract.js` と `nodesrc/test_core_char_doc_report_contract.js` により、`string_utf8_in_range`、`string_utf8_is_continuation`、`string_utf8_lead_kind`、`string_utf8_byte_at_checked`、2/3/4 byte sequence validator、`char_utf8_step_new`、`char_utf8_cont_byte` の report doctest、closed byte range、continuation byte range、overlong / surrogate / U+10FFFF boundary、`Option::None` から `Result::Err` への変換、non-owning `MemPtr` span、`CharUtf8Step` field contract、O(1) / O(byte_len) complexity を module 固有にも固定する。ただし `char_offsets`、string integer / float parse-format、scanner、slice、core/gui render command などに declaration doc gap が残るため、この issue は open のまま継続する。

同日の CharOffsets slice 後に baseline を再度締め直した。新しい悪化防止ラインは `declarations=2488`、`moduleNoDoctest=292`、`declarationNoDoc=143`、`declarationNoDoctest=1648`、`publicDeclarationNoDoctest=1495`、`privateDeclarationNoDoctest=153` である。`nodesrc/test_alloc_string_doc_report_contract.js` により、char index から byte offset への mixed UTF-8 mapping、internal `-1` sentinel と `Result::Err "string.char invalid slice range"` の境界、step width の continuation rejection、unchecked O(1) constructor、empty slice、`end_char == str_char_count(s)`、reversed range rejection を module 固有にも固定する。`nodesrc/test_stdlib_string_utf8_boundary.js` と `nodesrc/test_stdlib_text_boundary.js` により、UTF-8 lead kind classifier が negative sentinel を ASCII として扱わないことも固定する。ただし string integer / float parse-format、scanner、slice、core/gui render command などに declaration doc gap が残るため、この issue は open のまま継続する。

同日の Concat slice 後に baseline を再度締め直した。新しい悪化防止ラインは `declarations=2488`、`moduleNoDoctest=291`、`declarationNoDoc=142`、`declarationNoDoctest=1646`、`publicDeclarationNoDoctest=1493`、`privateDeclarationNoDoctest=153` である。`nodesrc/test_alloc_string_doc_report_contract.js` により、`concat_result` の `Result::Ok` / `Result::Err` 境界、`string_alloc_region` / `mem_copy` raw copy 境界、途中まで構築された `str` owner を公開しないこと、`concat` が error reason を捨てて `""` へ落とす互換 fallback であること、`concat3` が `a + b` の中間確保と再コピーを行う現状実装を module 固有にも固定する。ただし string integer / float parse-format、scanner、slice、core/gui render command などに declaration doc gap が残るため、この issue は open のまま継続する。

同日の FloatFormat slice 後に baseline を再度締め直した。新しい悪化防止ラインは `declarations=2488`、`moduleNoDoctest=290`、`declarationNoDoc=141`、`declarationNoDoctest=1642`、`publicDeclarationNoDoctest=1489`、`privateDeclarationNoDoctest=153` である。`nodesrc/test_alloc_string_doc_report_contract.js` により、`from_f64_fraction_trim_len` の末尾 0 trim、raw pointer writer helper の invalid trim fail-closed branch、`from_f64_build_fixed_result` の fixed-size allocation / `RegionToken` owner boundary、StringBuilder 非使用、`from_f64_result` の NaN / 6 桁上限 / 非丸め digit 展開、`from_f64` の `"0"` fallback、`from_f32` の f64 共通経路を module 固有にも固定する。Infinity の public contract 未整備は `ISS-20260605T194600610Z-STRING-FLOAT-INFINITY-FORMAT-UNSPECIFIED-A5C2D91E` として分離した。ただし string float parse、integer parse-format、scanner、slice、core/gui render command などに declaration doc gap が残るため、この issue は open のまま継続する。

同日の FloatParse slice 後に baseline を再度締め直した。新しい悪化防止ラインは `declarations=2488`、`moduleNoDoctest=289`、`declarationNoDoc=140`、`declarationNoDoctest=1640`、`publicDeclarationNoDoctest=1487`、`privateDeclarationNoDoctest=153` である。`nodesrc/test_alloc_string_doc_report_contract.js` と `nodesrc/test_stdlib_string_float_boundary.js` により、`float_parse_byte_or_invalid` の `Option` -> `-1` internal sentinel 変換、sentinel を public contract にしないこと、`to_f64` の digit 必須、clean end-of-input、exponent digit 必須、`nan` / `inf` symbolic value rejection、`Result::Err 1` 互換境界、`to_f32` の `to_f64` 結果伝播を module 固有にも固定する。parse error typed enum 未整備は `ISS-20260606T052400000Z-STRING-FLOAT-PARSE-ERROR-KIND-COLLAPSED-I32-4B21D9A7`、parse / format special value policy 未整備は `ISS-20260606T053000000Z-STRING-FLOAT-PARSE-SPECIAL-VALUE-POLICY-MISSING-7C18B2D4` として分離した。ただし string integer parse-format、scanner、slice、core/gui render command などに declaration doc gap が残るため、この issue は open のまま継続する。

同日の IntegerFormat slice 後に baseline を再度締め直した。新しい悪化防止ラインは `declarations=2488`、`moduleNoDoctest=288`、`declarationNoDoc=134`、`declarationNoDoctest=1640`、`publicDeclarationNoDoctest=1487`、`privateDeclarationNoDoctest=153` である。`nodesrc/test_alloc_string_doc_report_contract.js` により、`from_i32_radix` / `from_i64_radix` / `from_u128` / `from_u128_radix` / `from_i128` / `from_i128_radix` の report doctest、2 / 8 / 10 / 16 のみの radix contract、lowercase digit、negative sign、`Result::Err 1` invalid radix、`Result::Err 12` allocation / builder failure、`string_alloc_region` / `string_finish` raw storage boundary、scratch raw buffer 非使用、fallback API と Result API の境界を module 固有にも固定する。format error typed enum 未整備は `ISS-20260605T203044028Z-STRING-INTEGER-FORMAT-ERROR-KIND-COLLAPSED-I32-8C63F92A` として分離した。ただし string integer parse、scanner、slice、core/gui render command などに declaration doc gap が残るため、この issue は open のまま継続する。

## 影響

stdlib の修正時に、契約ではなく実装断片や既存挙動の記憶へ依存しやすくなる。特に collection / IO / GUI のように owner、Result、capability、platform boundary が絡む module では、doc gap が静的検査の活用不足やテスト観点漏れにつながる。

## 修正方針

module family ごとに分割して、declaration doc と declaration doctest を減らす。単純な baseline 下げではなく、各 public API について contract、現在の実装、計算量、Result / Option / enum の分岐条件、simple example と typical example を追加する。helper-only private declaration は、module doc または近傍の public doctestで検証される場合に限り、その根拠を記す。

## 検証

- `node nodesrc/test_stdlib_documentation_contract.js`
- module family ごとの focused doctest
- 追加される cfg-test-style regular tests
- `node nodesrc/test_stdlib_diag_doc_report_contract.js`
- `node nodesrc/test_stdlib_alloc_io_doc_report_contract.js`
- `node nodesrc/test_stdlib_hash32_doc_report_contract.js`
- `node nodesrc/test_alloc_string_doc_report_contract.js`
- `node nodesrc/tests.js -i stdlib/alloc/string/concat.nepl --no-tree -o tmp/agent2-string-concat-doc-slice.json -j 1 --dist web/dist --assert-io`
- `node nodesrc/tests.js -i stdlib/alloc/string/float/format.nepl --no-tree -o tmp/agent2-string-float-format-doc-slice.json -j 1 --dist web/dist --assert-io`
- `node nodesrc/tests.js -i stdlib/alloc/string/float/parse.nepl --no-tree -o tmp/agent2-string-float-parse-doc-slice-module.json -j 1 --dist web/dist --assert-io`
- `node nodesrc/tests.js -i stdlib/alloc/string/integer/format.nepl --no-tree -o tmp/agent2-string-integer-format-doc-slice.json -j 1 --dist web/dist --assert-io`
- `node nodesrc/tests.js -i stdlib/alloc/hash/sha256/api.nepl -i stdlib/alloc/hash/hash32.nepl -i stdlib/alloc/hash/fnv1a32.nepl -i stdlib/tests/hash.n.md --no-tree -o tmp/agent2-hash32-doc-smoke-5.json -j 1 --dist web/dist --assert-io`
- `node nodesrc/test_stdlib_hash_string_access_boundary.js`
- `node nodesrc/test_stdlib_hash_nmd_report_contract.js`
- `node nodesrc/test_stdlib_string_slice_boundary.js`
- `node nodesrc/test_stdlib_text_boundary.js`
- `node nodesrc/test_stdlib_utf8_validation_doc_report_contract.js`
- `node nodesrc/test_core_char_doc_report_contract.js`
- `node nodesrc/test_stdlib_char_utf8_byte_contract.js`
- `node nodesrc/test_stdlib_string_utf8_boundary.js`
- `node nodesrc/test_stdlib_string_storage_boundary.js`
- `node nodesrc/test_stdlib_bytebuf_utf8_boundary.js`
- `node nodesrc/tests.js -i stdlib/alloc/string/utf8.nepl -i stdlib/core/char.nepl -i tests/stdlib/char_utf8_byte_at.n.md -i tests/stdlib/string_char.n.md -i tests/stdlib/text_utf8.n.md --no-tree -o tmp/agent2-utf8-char-doc-slice.json -j 1 --dist web/dist --assert-io`
- `node nodesrc/tests.js -i stdlib/alloc/io/bytebuf.nepl -i stdlib/alloc/io/traits.nepl -i stdlib/alloc/io/bytebuilder/types.nepl --no-tree -o tmp/agent2-alloc-io-doc-smoke-4.json -j 1 --dist web/dist --assert-io`
- `node nodesrc/test_stdlib_io_bytebuf_owner_boundary.js`
- `node nodesrc/test_stdlib_builder_owner_boundary.js`
- `node nodesrc/test_stdlib_bytebuf_utf8_boundary.js`
- `node nodesrc/tests.js -i stdlib/alloc/diag/diag.nepl -i stdlib/alloc/diag/error/diag.nepl -i stdlib/alloc/diag/error/diags.nepl --no-tree -o tmp/agent2-diag-doc-smoke-4.json -j 1 --dist web/dist --assert-io`
- `node nodesrc/tests.js -i stdlib/tests/diag.n.md --no-tree -o tmp/agent2-diag-nmd-after-impure.json -j 1 --dist web/dist --assert-io`
- `node nodesrc/tests.js -i tests/stdlib/collections_diag.n.md --no-tree -o tmp/agent2-collections-diag-after-impure.json -j 1 --dist web/dist --assert-io`
- `node nodesrc/test_stdlib_bitset_doc_report_contract.js`
- `node nodesrc/tests.js -i stdlib/alloc/collections/bitset.nepl -i stdlib/alloc/collections/bitset/types.nepl -i stdlib/alloc/collections/bitset/layout.nepl -i stdlib/alloc/collections/bitset/storage.nepl -i stdlib/alloc/collections/bitset/mutation.nepl -i stdlib/alloc/collections/bitset/api.nepl -i stdlib/alloc/collections/bitset/api/diagnostic.nepl -i stdlib/alloc/collections/bitset/api/create.nepl -i stdlib/alloc/collections/bitset/api/observer.nepl -i stdlib/alloc/collections/bitset/api/update.nepl -i stdlib/alloc/collections/bitset/api/bulk.nepl -i stdlib/alloc/collections/bitset/api/cleanup.nepl -i tests/stdlib/bitset_collections.n.md --no-tree -o tmp/agent2-bitset-doc-slice-2.json -j 1 --dist web/dist --assert-io`
- `node nodesrc/test_stdlib_adjacency_matrix_doc_report_contract.js`
- `node nodesrc/tests.js -i stdlib/alloc/collections/adjacency_matrix.nepl -i stdlib/alloc/collections/adjacency_matrix/types.nepl -i stdlib/alloc/collections/adjacency_matrix/layout.nepl -i stdlib/alloc/collections/adjacency_matrix/storage.nepl -i stdlib/alloc/collections/adjacency_matrix/mutation.nepl -i stdlib/alloc/collections/adjacency_matrix/api.nepl -i stdlib/alloc/collections/adjacency_matrix/api/diagnostic.nepl -i stdlib/alloc/collections/adjacency_matrix/api/create.nepl -i stdlib/alloc/collections/adjacency_matrix/api/observer.nepl -i stdlib/alloc/collections/adjacency_matrix/api/update.nepl -i stdlib/alloc/collections/adjacency_matrix/api/bulk.nepl -i stdlib/alloc/collections/adjacency_matrix/api/cleanup.nepl -i tests/stdlib/adjacency_matrix_collections.n.md -i stdlib/tests/adjacency_matrix.n.md --no-tree -o tmp/agent2-adjacency-matrix-doc-slice.json -j 1 --dist web/dist --assert-io`
- `node nodesrc/test_stdlib_binary_heap_doc_report_contract.js`
- `node nodesrc/tests.js -i stdlib/alloc/collections/binary_heap.nepl -i stdlib/alloc/collections/binary_heap/types.nepl -i stdlib/alloc/collections/binary_heap/storage.nepl -i stdlib/alloc/collections/binary_heap/order.nepl -i stdlib/alloc/collections/binary_heap/api.nepl -i stdlib/alloc/collections/binary_heap/api/create.nepl -i stdlib/alloc/collections/binary_heap/api/observer.nepl -i stdlib/alloc/collections/binary_heap/api/push.nepl -i stdlib/alloc/collections/binary_heap/api/pop.nepl -i stdlib/alloc/collections/binary_heap/api/cleanup.nepl -i tests/stdlib/binary_heap_collections.n.md -i stdlib/tests/binary_heap.n.md --no-tree -o tmp/agent2-binary-heap-doc-slice.json -j 1 --dist web/dist --assert-io`
- `node nodesrc/test_stdlib_bloom_filter_doc_report_contract.js`
- `node nodesrc/tests.js -i stdlib/alloc/collections/bloom_filter.nepl -i stdlib/alloc/collections/bloom_filter/types.nepl -i stdlib/alloc/collections/bloom_filter/hash.nepl -i stdlib/alloc/collections/bloom_filter/layout.nepl -i stdlib/alloc/collections/bloom_filter/storage.nepl -i stdlib/alloc/collections/bloom_filter/mutation.nepl -i stdlib/alloc/collections/bloom_filter/api.nepl -i stdlib/tests/bloom_filter.n.md -i tests/stdlib/bloom_filter_collections.n.md --no-tree -o tmp/agent2-bloom-filter-doc-slice-fourth.json -j 1 --dist web/dist --assert-io`
- `node nodesrc/test_stdlib_counting_bloom_filter_doc_report_contract.js`
- `node nodesrc/tests.js -i stdlib/alloc/collections/counting_bloom_filter.nepl -i stdlib/alloc/collections/counting_bloom_filter/types.nepl -i stdlib/alloc/collections/counting_bloom_filter/hash.nepl -i stdlib/alloc/collections/counting_bloom_filter/storage.nepl -i stdlib/alloc/collections/counting_bloom_filter/mutation.nepl -i stdlib/alloc/collections/counting_bloom_filter/api.nepl --no-tree -o tmp/agent2-counting-bloom-filter-doc-modules.json -j 1 --dist web/dist --assert-io`
- `node nodesrc/tests.js -i stdlib/tests/counting_bloom_filter.n.md -i tests/stdlib/counting_bloom_filter_collections.n.md --no-tree -o tmp/agent2-counting-bloom-filter-existing-tests.json -j 1 --dist web/dist --assert-io`
- `node nodesrc/test_stdlib_btree_search_doc_report_contract.js`
- `node nodesrc/test_stdlib_btreemap_storage_doc_report_contract.js`
- `node nodesrc/tests.js -i stdlib/alloc/collections/btreemap/search.nepl -i stdlib/alloc/collections/btreemap/storage.nepl -i stdlib/tests/btreemap.n.md --no-tree -o tmp/agent2-btreemap-doc-slice.json -j 1 --dist web/dist --assert-io`
- `node nodesrc/test_stdlib_btreeset_storage_doc_report_contract.js`
- `node nodesrc/tests.js -i stdlib/alloc/collections/btreeset/search.nepl -i stdlib/alloc/collections/btreeset/storage.nepl -i stdlib/tests/btreeset.n.md --no-tree -o tmp/agent2-btreeset-doc-slice.json -j 1 --dist web/dist --assert-io`
- `node nodesrc/test_stdlib_disjoint_set_doc_report_contract.js`
- `node nodesrc/tests.js -i stdlib/alloc/collections/disjoint_set.nepl -i stdlib/alloc/collections/disjoint_set/types.nepl -i stdlib/alloc/collections/disjoint_set/storage.nepl -i stdlib/alloc/collections/disjoint_set/query.nepl -i stdlib/alloc/collections/disjoint_set/api.nepl -i stdlib/alloc/collections/disjoint_set/api/diagnostic.nepl -i stdlib/alloc/collections/disjoint_set/api/create.nepl -i stdlib/alloc/collections/disjoint_set/api/observer.nepl -i stdlib/alloc/collections/disjoint_set/api/mutation.nepl -i stdlib/alloc/collections/disjoint_set/api/cleanup.nepl -i stdlib/tests/disjoint_set.n.md -i tests/stdlib/disjoint_set_collections.n.md --no-tree -o tmp/agent2-disjoint-set-doc-slice.json -j 1 --dist web/dist --assert-io`
- `node nodesrc/test_stdlib_fenwick_doc_report_contract.js`
- `node nodesrc/tests.js -i stdlib/alloc/collections/fenwick.nepl -i stdlib/alloc/collections/fenwick/types.nepl -i stdlib/alloc/collections/fenwick/storage.nepl -i stdlib/alloc/collections/fenwick/query.nepl -i stdlib/alloc/collections/fenwick/mutation.nepl -i stdlib/alloc/collections/fenwick/api.nepl -i stdlib/alloc/collections/fenwick/api/diagnostic.nepl -i stdlib/alloc/collections/fenwick/api/create.nepl -i stdlib/alloc/collections/fenwick/api/observer.nepl -i stdlib/alloc/collections/fenwick/api/query.nepl -i stdlib/alloc/collections/fenwick/api/update.nepl -i stdlib/alloc/collections/fenwick/api/cleanup.nepl -i stdlib/tests/fenwick.n.md -i tests/stdlib/fenwick_collections.n.md --no-tree -o tmp/agent2-fenwick-doc-slice.json -j 1 --dist web/dist --assert-io`
- `node nodesrc/test_stdlib_segment_tree_doc_report_contract.js`
- `node nodesrc/tests.js -i stdlib/alloc/collections/segment_tree.nepl -i stdlib/alloc/collections/segment_tree/types.nepl -i stdlib/alloc/collections/segment_tree/layout.nepl -i stdlib/alloc/collections/segment_tree/storage.nepl -i stdlib/alloc/collections/segment_tree/range.nepl -i stdlib/alloc/collections/segment_tree/mutation.nepl -i stdlib/alloc/collections/segment_tree/api.nepl -i stdlib/alloc/collections/segment_tree/api/diagnostic.nepl -i stdlib/alloc/collections/segment_tree/api/create.nepl -i stdlib/alloc/collections/segment_tree/api/observer.nepl -i stdlib/alloc/collections/segment_tree/api/query.nepl -i stdlib/alloc/collections/segment_tree/api/update.nepl -i stdlib/alloc/collections/segment_tree/api/cleanup.nepl -i stdlib/tests/segment_tree.n.md -i tests/stdlib/segment_tree_collections.n.md --no-tree -o tmp/agent2-segment-tree-doc-slice.json -j 1 --dist web/dist --assert-io`
- `node nodesrc/test_stdlib_sparse_set_doc_report_contract.js`
- `node nodesrc/tests.js -i stdlib/alloc/collections/sparse_set.nepl -i stdlib/alloc/collections/sparse_set/types.nepl -i stdlib/alloc/collections/sparse_set/storage.nepl -i stdlib/alloc/collections/sparse_set/membership.nepl -i stdlib/alloc/collections/sparse_set/mutation.nepl -i stdlib/alloc/collections/sparse_set/api.nepl -i stdlib/alloc/collections/sparse_set/api/diagnostic.nepl -i stdlib/alloc/collections/sparse_set/api/create.nepl -i stdlib/alloc/collections/sparse_set/api/observer.nepl -i stdlib/alloc/collections/sparse_set/api/update.nepl -i stdlib/alloc/collections/sparse_set/api/bulk.nepl -i stdlib/alloc/collections/sparse_set/api/cleanup.nepl -i stdlib/tests/sparse_set.n.md -i tests/stdlib/sparse_set_collections.n.md --no-tree -o tmp/agent2-sparse-set-doc-slice.json -j 1 --dist web/dist --assert-io`
