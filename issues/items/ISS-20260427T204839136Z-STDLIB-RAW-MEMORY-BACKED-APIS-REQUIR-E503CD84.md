---
id: ISS-20260427T204839136Z-STDLIB-RAW-MEMORY-BACKED-APIS-REQUIR-E503CD84
title: "stdlib raw-memory-backed APIs require staged effect migration"
area: stdlib
status: open
resolved: false
priority: P1
type: architecture
created: 2026-04-27
updated: 2026-05-19
target: "stdlib/core/mem.nepl, stdlib/alloc/collections/vec.nepl, stdlib/alloc/string.nepl, stdlib/alloc/io.nepl, stdlib/std/fs.nepl, stdlib/std/stdio.nepl, stdlib/std/streamio.nepl, nepl-core/src/typecheck.rs"
---

# ISS-20260427T204839136Z-STDLIB-RAW-MEMORY-BACKED-APIS-REQUIR-E503CD84: stdlib raw-memory-backed APIs require staged effect migration

## 概要

core/mem raw primitives are still exposed as Pure because large parts of stdlib call alloc_raw/dealloc_raw/load/store from functions whose public signatures are currently pure. Marking those primitives impure in the compiler immediately causes widespread D3025 failures in Vec, string, IO, fs, diag, and stdio helpers.

## 対象

- `stdlib/core/mem.nepl, stdlib/alloc/collections/vec.nepl, stdlib/alloc/string.nepl, stdlib/alloc/io.nepl, stdlib/std/fs.nepl, stdlib/std/stdio.nepl, stdlib/std/streamio.nepl, nepl-core/src/typecheck.rs`

## 根拠

- `tests/compiler/move_effect.n.md` の既存正常系「pure からメモリ操作を呼べる」は、pure `compute` から `alloc_raw` / `store_i32` / `load_i32` / `dealloc_raw` を呼び、`ret: 123` で通る。
- compiler 側で raw memory boundary 内の raw primitive を `Effect::Impure` として登録する試作を行うと、`tests/compiler/move_effect.n.md` の stdlib import ケースで `stdlib/alloc/collections/vec.nepl:278` の `store<.T>`、`vec.nepl:648` の `load<.T>`、`stdlib/alloc/string.nepl`、`stdlib/std/fs.nepl`、`stdlib/std/stdio.nepl`、`stdlib/std/streamio.nepl` などが一斉に D3025 になる。
- これは compiler の effect 判定だけの問題ではなく、stdlib が raw memory backed helper を pure API として公開・利用している設計移行の問題である。

## 問題

core/mem raw primitives are still exposed as Pure because large parts of stdlib call alloc_raw/dealloc_raw/load/store from functions whose public signatures are currently pure. Marking those primitives impure in the compiler immediately causes widespread D3025 failures in Vec, string, IO, fs, diag, and stdio helpers.

## 影響

The compiler cannot close the raw memory effect boundary issue without either breaking current stdlib APIs or introducing a richer internal memory effect. Pure source can still reach allocation or raw storage through stdlib wrappers, so the effect model remains unsound for self-host planning.

## 修正方針

Stage the migration: introduce a compiler-level internal/unsafe memory effect or explicit stdlib unsafe boundary, update raw-memory-backed stdlib APIs to either be impure or wrap an internal effect safely, and only then make core/mem raw primitives externally impure by default.

## 検証

Add compile_fail tests for user pure calls to raw primitives and stdlib migration tests that show intended safe wrappers either require impure context or are proven pure through the new internal effect boundary.

## 関連ドキュメント

- [NEPLg2 静的検査の複雑化解消計画](../../doc/neplg2/static_check_complexity_reduction_plan.md)

## 2026-04-28 issue 整理

この issue は Stage 6 の stdlib migration parent とする。raw-memory-backed 実装を禁止する issue ではなく、raw memory discipline を public API と利用者 code へ押し出さないための移行 issue である。

compiler 側の Resource IR / effect model が先行して整うまでは、既存 stdlib API を一括 impure 化しない。移行順は `core/mem` internal/public 境界、`Vec` / `StringBuilder` の owner token 移行、collection drop contract、self-host buffer API の順にする。stdlib 側で compiler 修正が必要になった場合は core issue と混ぜず、別 issue として分離する。

## 2026-04-28 memory model 方針レビュー追記

現在の stdlib は `Vec`、`string`、`io`、`fs`、`stdio`、`streamio` などの実装で raw memory backed helper を pure API の内部から呼んでいる。内部 mutation を使うこと自体は問題ではないが、raw address や storage identity が safe surface へ漏れる場合は referential transparency を compiler が証明できない。

したがって stdlib 側の方向は「raw memory を使わない」ではなく、「raw memory を public API discipline として利用者へ押し出さない」に修正する。具体的には、内部 builder / collection storage は `InternalAlloc` と owner token に閉じ、公開 API は Copy read、move-out、drop/free obligation を型と Resource IR で区別する。

self-host 実装では、S1/S2 の文字列走査・token 配列・diagnostic からこの方針を適用する。短期的には既存 `Vec` / `StringBuilder` を使って開始できるが、compiler 本体へ raw `MemPtr` 操作を直接持ち込む実装は避ける。

## 2026-04-28 stdlib full review 追記

最新 main (`0e6ffae`) で stdlib の source policy と doctest を再確認した。

- `nodesrc/test_stdlib*.js` の source policy は全件 pass。unsafe unwrap、match decision tree、NM raw aggregate detour、StringBuilder ownership comment、diag/error compact layout policy は現状の静的検査変更に追従できている。
- `node nodesrc/tests.js -i stdlib/tests --no-tree -o tmp/stdlib-tests-full-review-after-diag-policy.json -j 4`: `total=80`, `passed=80`
- `node nodesrc/tests.js -i tests/stdlib --no-tree -o tmp/tests-stdlib-full-review-after-diag-policy.json -j 4`: `total=311`, `passed=303`, `failed=7`, `errored=1`
- `node nodesrc/tests.js -i tests/stdlib/traits_hash.n.md --no-tree -o tmp/traits-hash-timeout-review-20260428.json -j 1`: `total=6`, `passed=6`

`tests/stdlib` の残件は、source policy の未追従ではなく、strict move checking が non-Copy owner を raw storage に置く古い fixture / API design を拒否しているものだった。

- `tests/stdlib/capacity_stack.n.md::doctest#6`: `Vec<Kind>` grow が `D3100 reallocating raw memory place containing non-Copy value: $memptr:v_data+?`
- `tests/stdlib/json_typed_values.n.md::doctest#2`: `Vec<JsonValue>` grow が同種の `D3100`
- `tests/stdlib/json_typed_values.n.md::doctest#3/#4`: structured JSON payload の raw `data` owner を再利用して `D3100 use of moved raw memory place: data`
- `tests/stdlib/fs.n.md::doctest#5/#6`: `std/test` result aggregation path で `D3100 use of moved raw memory place: popped`
- `tests/stdlib/neplg2_diag_outcome.n.md::doctest#3`: `SelfhostOutcome` result cell が `D3100 overwrite of raw memory place containing non-Copy value: $memptr:result_ptr`
- `tests/stdlib/traits_hash.n.md::doctest#5`: broad parallel run では 20s timeout したが、focused run では 6/6 pass したため、現時点では再現性ある issue として分離しない。

このため、self-host 実装開始の観点では、文字列・診断 text・NM direct serializer・source policy は前進しているが、typed JSON value、generic `Vec<T>` with enum/non-Copy payload、`std/test` aggregation、`SelfhostOutcome` cell owner は Resource IR / owned collection model の移行対象として残る。現時点で新規 issue 追加ではなく、この parent issue の Stage 6 入力として扱う。

## 2026-05-13 Stage 6 source capability proof 追記

raw-memory-boundary capability は、stdlib の特定 path に一致するだけでは付与しない設計へ移行する。`core/mem` facade や user source が同じ suffix を持つだけで raw boundary 扱いになると、静的検査の authority が file layout に依存し、compiler が source property を証明していない状態になるためである。

今回の compiler 側変更では、loader が source を SourceMap に登録した後、parse 済み AST を検査して raw-memory-boundary の証拠を確認してから capability を設定する。証拠は文字列 ad hoc ではなく `RawMemoryBoundaryEvidence` enum として分ける。

- `RawBodyInstruction`: `#wasm` / `#llvm` の raw memory operation
- `RawAddressBoundaryHelper`: `mem_ptr_addr` / `mem_ptr_wrap` / `region_ptr` など Resource IR が raw address identity として扱う helper
- `RawHelperCall`: raw helper 名への直接呼び出し
- `RawOwnerBoundaryHelper`: `alloc_ptr` / `realloc_ptr` / `dealloc_ptr` / `alloc_region` など free obligation owner helper
- `RawIntrinsic`: raw memory intrinsic
- `RestrictedConstructor`: `MemPtr` / `RegionToken` など raw boundary 専用 constructor

Stage 6 の現段階で `RAW_MEMORY_BOUNDARY_STDLIB_PATHS` の module allowlist は削除した。権限付与条件は「configured stdlib root 配下の compiler-owned source provenance かつ source evidence あり」であり、特定 stdlib module を列挙して許可しない。これにより path-only privilege と module-list 追従作業を廃止し、raw boundary の authority を source property proof に移した。

`source_capability.rs` は root module と `source_capability/raw_memory.rs` に分割し、loader / SourceMap から見える public surface と raw memory evidence scanner の責務を分けた。source scanner は enum + match で証拠種別を管理し、旧 diag id や path 文字列 allowlist は残していない。

検証は以下で行った。

- `cargo test -p nepl-core loader::tests::raw_memory_boundary -- --nocapture`: 9 passed
- `cargo test -p nepl-core --test effects raw_memory_boundary -- --nocapture`: 4 passed
- `cargo check -p nepl-core --tests`: passed
- `cargo fmt -p nepl-core --check`: passed
- `node nodesrc/test_static_check_boundary_responsibility.js`: passed
- `node nodesrc/test_resource_gate_order.js`: passed
- `node nodesrc/run_source_policy_regressions.js --warn-only`: passed
- `trunk build`: passed
- `node nodesrc/tests.js -i stdlib/core/mem -i stdlib/alloc/io/bytebuilder --no-tree -o tmp/agent1-source-capability-proof-mem-bytebuilder.json -j 1 --dist web/dist`: 32 passed
- `node nodesrc/tests.js -i stdlib/alloc/string -i stdlib/alloc/collections/vec --no-tree -o tmp/agent1-source-capability-proof-string-vec.json -j 1 --dist web/dist`: 46 total / 6 passed / 40 failed。失敗は `unwrap_ok` / `eq` / `gt` / `get` などの undefined identifier で、raw-memory-boundary capability 不足ではなく stale doctest import 問題として `ISS-20260513T082754555Z-VEC-AND-STRING-SUBMODULE-DOCTESTS-RE-E80ED44C` を追加した。

同日の main GitHub Actions run `25784435303` では、全体 doctest 側に以下の既知カテゴリの失敗が残っている。

- `tests/compiler/intrinsic.n.md`: pure function から raw `store` を呼ぶ古い fixture が `effect.pure.calls_impure` で拒否される。
- tutorials / examples: `Vec` / `push` / `vec_free_storage` 経由で `resource.owner.no_free_obligation` が出ている。
- stdlib dual backend: collection doctest に `resolve.identifier.undefined` と `resource.owner.no_free_obligation` が混在している。

これらは source capability 証明の回帰ではなく、Stage 6 の残件である raw-memory-backed collection owner model と古い raw intrinsic fixture の移行として扱う。

## 2026-05-13 Vec/string doctest import drift 整理追記

`ISS-20260513T082754555Z-VEC-AND-STRING-SUBMODULE-DOCTESTS-RE-E80ED44C` で Vec / string submodule doctest の stale implicit import を整理した。修正後の focused run は以下の状態である。

- `node nodesrc/tests.js -i stdlib/alloc/string --no-tree -o tmp/string-submodule-doctests-imports-after.json -j 1 --dist web/dist`: total=14, passed=14。
- `node nodesrc/tests.js -i stdlib/alloc/string -i stdlib/alloc/collections/vec --no-tree -o tmp/string-vec-submodule-doctests-imports-after.json -j 4 --dist web/dist`: total=46, passed=15, failed=31。失敗は全て `resource.owner.no_free_obligation` で、`resolve.identifier.undefined` は 0 件。

したがって、stdlib doctest の undefined identifier drift は解消済みであり、残る Vec 側の compile failure は `vec_free_storage` / `push` / merge sort buffer cleanup が `MemPtr` storage field を free obligation owner として扱う過渡設計に由来する。これは path / import の問題ではなく、Stage 6 の raw-memory-backed collection owner model 残件として継続する。

## 2026-05-14 CapacityStack fixture migration note

`tests/stdlib/capacity_stack.n.md` の stdout report 移行時に、2026-04-28 時点で記録していた `Vec<Kind>` grow failure は fixture 側を現行設計へ合わせた。`Kind` は payload を持たない enum なので `Clone` / `Copy` を明示し、Copy-only collection boundary を迂回せずに `Vec<Kind>` growth を検証する。

同じ移行で memory block case は `alloc_raw` / `store_i32` / `load_i32` の direct raw address 操作ではなく、`RegionToken` / `MemPtr` public API 経由の store/load に直した。これは raw-memory-backed API 移行 issue の解決ではなく、ordinary doctest が raw boundary privilege を要求しないようにする fixture 側の整理である。non-Copy collection owner model と raw-memory-backed storage の本体はこの issue の Stage 6 残件として継続する。

## 2026-05-13 Vec storage dealloc owner proof 追記

`ISS-20260513T090733651Z-VEC-STORAGE-CLEANUP-DEALLOCATES-THRO-4A132C97` で、Vec storage cleanup が `MemPtr` を raw address へ落として `dealloc_raw` へ渡していた問題を解消した。

- `vec_free_storage`、`push` の realloc failure cleanup、merge sort scratch buffer cleanup は `dealloc_ptr<T>` を通す。
- 確保直後の scratch buffer dealloc failure は通常の API error ではなく invariant violation として `unreachable` にし、Resource IR 上で owner leak branch を残さない。
- `node nodesrc/tests.js -i stdlib/alloc/collections/vec --no-tree -o tmp/vec-owner-dealloc-ptr-all-after-unreachable.json -j 4 --dist web/dist`: total=32, passed=32。

この修正は `MemPtr = non-owning pointer` の方針を弱めるものではなく、raw `i32` へ owner proof を落とさず typed owner-consuming API 境界で free obligation を閉じる Stage 6 の局所前進である。`OwnedBuffer<T>` / collection drop contract 自体は引き続き残る。

## 2026-05-13 Agent 1 source capability shadowing 追記

`ISS-20260513T095201685Z-RAW-MEMORY-SOURCE-CAPABILITY-TREATS--389248CD` で、raw-memory-boundary source capability scanner が raw helper と同名の parameter / local / same-module safe helper を evidence と誤認できる問題を修正した。

今回の修正により、capability scanner は `RawMemoryBoundaryEvidence` の enum 分類を維持したまま、`RawMemoryBoundaryScope` で lexical shadowing を管理する。関数/method parameter、block 内 `let` binding、match payload binding、同一 module の top-level 定義により shadow された `mem_ptr_addr` / `alloc_ptr` / `load_i32` などは raw evidence として数えない。

この親 issue は引き続き open とする。source capability proof の spelling false positive は閉じたが、raw-memory-backed stdlib public API の owner token / `OwnedBuffer<T>` / safe wrapper 移行は Stage 6 の残件である。

## 2026-05-13 Agent 1 compiler intrinsic fixture drift 追記

`ISS-20260513T121049229Z-COMPILER-INTRINSIC-FIXTURES-STILL-US-004161B4` で、`tests/compiler/intrinsic.n.md` の古い raw memory fixture を現在の Resource IR effect boundary に合わせた。

- raw `load` / `store` / `dealloc_raw` を直接検証する runtime tests は `fn main <()*>i32> ():` に変更し、pure function から unsafe memory operation を呼ぶ形を残さない。
- raw memory を使わない size/layout tests は pure のまま維持し、`size_of` / `align_of` は現在の public layout API である `core/mem` を直接 import する。
- `node nodesrc/tests.js -i tests/compiler/intrinsic.n.md --no-tree -o tmp/agent1-intrinsic-fixtures-after.json -j 1 --dist web/dist`: total=8, passed=8。

この修正は effect checker を緩めるものではなく、Stage 5/6 の unsafe memory boundary を正しく fixture に表現したものである。親 issue は collection owner model / safe public wrapper の残件があるため open のまま継続する。

## 2026-05-13 Agent 1 Vec raw data observer 境界追記

`ISS-20260513T115656872Z-VEC-DATA-OBSERVERS-EXPOSE-RAW-POINTE-674F1AFF` で、`Vec` の raw data observer が non-Copy payload に対して raw address / `MemPtr<T>` view を返せる入口を閉じた。

`data_mem_ptr` は `.T: Copy` に限定済みである。`data_ptr` は後続の `ISS-20260514T160255919Z-VEC-DATA-PTR-EXPOSES-RAW-I32-STORAGE-546EA2EB` で削除し、`vec_storage_mem_ptr` は `ISS-20260514T161819706Z-VEC-STORAGE-MEMPTR-HELPER-EXPOSES-LO-A9C5BC02` で削除した。raw `i32` address observer や lower-level storage-state helper の互換 API は残さない。これは raw-memory-backed public API migration の完了ではなく、`OwnedBuffer<T>` と borrow projection が入るまで unsafe な storage identity escape を Copy payload に限定しつつ、raw address への変換を raw-memory-boundary implementation point へ押し戻す局所前進である。

## 2026-05-14 Agent 1 VecDataLen raw view carrier 削除追記

`ISS-20260514T055830236Z-VECDATALEN-CARRIES-RAW-VEC-STORAGE-V-B662D7DF` で、`VecDataLen<T>` と `data_len<T>` を削除した。

`VecDataLen<T>` は `Vec.data: MemPtr<T>` と `len` を public struct field としてまとめるだけで、Copy-only にしても raw storage view を field projection 可能な形で再包装していた。これは `MemPtr = non-owning pointer` / `OwnedBuffer = free obligation owner` への Stage 6 移行方針と合わないため、互換 alias は残さず削除する。

呼び出し側は、必要な箇所で `len<T>(&Vec<T>)` と `data_mem_ptr<T>(&Vec<T>)` を明示的に別々に観測する。これにより `MemPtr` owner-like field policy の transitional baseline は `RegionToken.ptr`、`Vec.data`、`ByteBuf.ptr`、`ByteBuilder.ptr`、`StringBuilder.data` の 5 件になった。

## 2026-05-14 Agent 1 StringBuilder owner boundary 集約追記

`ISS-20260514T063755030Z-STRINGBUILDER-DUPLICATES-BYTEBUILDER-F90DFA2F` で、`StringBuilder` 固有の `Option<MemPtr<u8>>` / len / cap owner state を削除した。

`StringBuilder` は text API の境界であり、byte storage の free obligation は既に `ByteBuilder` が保持している。両者が独立に raw owner layout を持つと、Stage 6 の静的検査は同じ性質を 2 つの public struct で追跡する必要があり、`MemPtr = non-owning pointer` / owner wrapper 分離に反する。

修正後の `StringBuilder` は `bytes: ByteBuilder` だけを持つ typed wrapper である。capacity / append / free は `ByteBuilder` API へ委譲し、build は `ByteBuilder -> ByteBuf -> str` の typed owner boundary を通す。これにより `StringBuilder.data` は `MemPtr` owner-like field policy から外れ、transitional baseline は `RegionToken.ptr`、`Vec.data`、`ByteBuf.ptr`、`ByteBuilder.ptr` の 4 件になった。

## 2026-05-14 Agent 1 ByteBuf / ByteBuilder RegionToken owner 境界追記

`ISS-20260514T071955576Z-BYTEBUF-STORES-OWNED-BYTES-AS-OPTION-FA165159` で、`ByteBuf` / `ByteBuilder` の `Option<MemPtr<u8>>` owner field を削除した。

`MemPtr` は non-owning pointer / projection として固定する方針であり、owned byte storage を `Option<MemPtr<u8>>` field に置く設計は Stage 6 の過渡例外を stdlib public state に残していた。この時点の修正では `ByteBuf.region` / `ByteBuilder.region` が `RegionToken<u8>` owner を保持し、payload pointer は `io_bytebuf_data_ptr_ref` / `byte_builder_data_ptr_ref` で参照から non-owning view として得る形に移した。後続の 2026-05-18 追記で、現在の `ByteBuf` / `ByteBuilder` はそれぞれ storage enum によって empty state と owned state を分離している。

この移行中に ResourceIR function summary が `region_ptr` 由来の non-owning projection に `mem_ptr_add` を重ねた値を owner alias として扱い、append path を maybe leak と誤診断する問題も露出した。stdlib allowlist で回避せず、summary traversal に raw view state を持たせ、non-owning projection 由来の offset view が owner alias を伝播しないよう compiler 側を修正した。

これにより `MemPtr` owner-like field policy の transitional baseline は `RegionToken.ptr`、`Vec.data` の 2 件になった。残る Stage 6 の主対象は `Vec` / `OwnedBuffer<T>` と、forgeable `RegionToken` を compiler-issued owner token へ移す設計である。

## 2026-05-14 Agent 1 Vec RegionToken owner 境界追記

`ISS-20260514T085248522Z-VEC-STORES-BACKING-STORAGE-AS-MEMPTR-FFC9775A` で、`Vec<T>` の `data: MemPtr<T>` owner field を削除した。

`MemPtr<T>` は non-owning pointer / projection として固定する方針であり、collection の基礎 storage owner を `MemPtr<T>` field に置き続けると Stage 6 の Resource IR は owner と view の二重責務を追い続けることになる。修正後の `Vec<T>` は `region: RegionToken<T>` を free obligation owner として持ち、`data_mem_ptr<T>` / sort・transform・mutation 系は `RegionToken<T>` 参照から non-owning `MemPtr<T>` view を得る。

allocation は `alloc_region<T>`、storage-only cleanup は `dealloc_region<T>`、grow failure cleanup は `vec_realloc_region_or_free<T>` に集約した。これにより `nodesrc/test_stdlib_memptr_owner_field_policy.js` の transitional baseline は `RegionToken.ptr` の 1 件だけになった。

この親 issue は引き続き open とする。`RegionToken<T>` はまだ forgeable であり、`OwnedBuffer<T>` / initialized prefix / non-Copy payload drop traversal / owner-preserving fallible collection update は Stage 6 の残件である。

## 2026-05-15 Agent 1 alloc/string root facade raw helper 分離追記

`ISS-20260514T220733927Z-ALLOC-STRING-ROOT-RE-EXPORTS-RAW-STR-BF0F0254` で、ordinary `alloc/string` root facade から `alloc/string/storage` と `alloc/string/utf8` の public wildcard re-export を削除した。

`string_data_ptr`、`string_from_mem_unchecked_result`、`string_from_utf8_mem_result`、`string_utf8_validate_mem` などの raw `MemPtr`-based helper は、今後 root `alloc/string` import からは到達できない。OS boundary / storage conversion / scanner など本当に raw helper を使う stdlib 実装は、`alloc/string/storage` / `alloc/string/utf8` を明示 import する。

これは raw helper 自体を削除する変更ではなく、`MemPtr = non-owning pointer` と safe public facade の責務を分ける Stage 6 の public/raw boundary split である。`std/fs` / `std/stdio` / `std/env/cliarg` / `std/streamio` の focused doctest は pass している。`stdlib/tests/string.n.md` の broad run に残る stale import fixture は `ISS-20260514T221807506Z-STDLIB-STRING-DOCTESTS-RETAIN-STALE--CC9D6303` として別管理にした。

同じ監査で、`byte_builder_realloc_region_or_free` の realloc failure cleanup が `dealloc_ptr` 失敗時に `#intrinsic "unreachable"` へ落ちる問題を確認した。これは今回の Vec owner field 削除とは別件として、`ISS-20260514T093715629Z-BYTEBUILDER-GROW-CLEANUP-STILL-USES--DC675F3E` に分離して修正した。この時点の ByteBuilder grow failure cleanup は、`dealloc_region<u8> region` で owner token を直接消費する形まで進めていた。

## 2026-05-20 Agent 1 ByteBuilder fallible owner payload 追記

`ISS-20260519T181131422Z-BYTEBUILDER-FALLIBLE-OWNER-APIS-DISC-DBFDE7BB` で、`ByteBuilder` の owner-consuming fallible API が bare `StdErrorKind` だけを返す設計を修正した。

`byte_builder_reserve`、byte append 系、`byte_builder_finish` は失敗時に `ByteBuilderError` へ入力 builder owner を戻す。`byte_builder_push_bytebuf` は builder と入力 `ByteBuf` の 2 owner を消費するため、`ByteBuilderByteBufError` で両 owner を返す。realloc helper も `byte_builder_realloc_region_or_keep` に改め、失敗時に旧 `RegionToken<u8>` を `RegionReallocError<u8>` 経由で保持する。

これは `byte_builder_realloc_region_or_free` の cleanup discipline をさらに進め、失敗時 owner transfer を実装内部の free に隠さず API 型に出す Stage 6 の整理である。`StringBuilder` / `StreamWriter` は既存 public API を維持する箇所では内部 `ByteBuilderError` を明示的に free してから従来の error へ写像する。

## 2026-05-14 Agent 1 Vec merge sort fallible owner 追記

`ISS-20260514T102108865Z-VEC-SORT-MERGE-RET-ERR-PATH-LOSES-CO-98B83660` で、`sort_merge_ret<T>` の失敗時 owner contract を修正した。

`sort_merge_ret<T>` は `Vec<T>` owner を消費する API であるため、`Result<Vec<T>, StdErrorKind>` では allocation failure や cleanup failure で caller が `Vec<T>` を回収できなかった。修正後は `Result<Vec<T>, VecSortMergeError<T>>` を返し、`Err` payload に `Vec<T>` owner と `StdErrorKind` を保持する。

また、merge sort scratch buffer は `alloc_ptr` / `dealloc_ptr` の raw pointer owner ではなく、`alloc_region<T>` / `dealloc_region<T>` の `RegionToken<T>` owner として閉じる。`MemPtr<T>` は `region_ptr &buf_region` 由来の non-owning view であり、scratch cleanup も `unreachable` ではなく明示的な `Result` error として扱う。

## 2026-05-13 Agent 1 region_ptr_at alignment proof 追記

`ISS-20260513T100047236Z-REGION-PTR-AT-RETURNS-TYPED-MEMPTR-W-39BD1C91` で、`region_ptr_at` が byte bounds だけを検査して `MemPtr<U>` を返していた問題を修正した。

今回の修正により、`region_ptr_at` は `size_of<U>` による `off..off+size_of<U>` の範囲検査に加え、`align_of<U>` による実 address `base + off` の alignment 検査を同じ owner boundary で行う。typed pointer projection の前提を呼び出し側や後続 wrapper へ委譲せず、`RegionToken` から `MemPtr<U>` を得る時点で memory/type safety 条件を閉じる。

これは `MemPtr = non-owning pointer` / `RegionToken = free obligation owner` の分離を維持する修正であり、`MemPtr` に owner proof を追加するものではない。`OwnedBuffer<T>` / collection element drop traversal / raw-memory-backed public API migration は Stage 6 の残件として継続する。

## 2026-05-13 Agent 1 allocation payload size proof 追記

`ISS-20260513T101054155Z-CORE-MEM-ALLOCATION-BYTE-COUNTS-CAN--9B7BDEA4` で、allocator payload byte 数の overflow proof を追加した。

`alloc_raw` は `align8(size + header)` の前に `alloc_payload_fits` で `size + 8 + 7 <= i32::MAX` を満たすことを確認する。`alloc_region` は `count * size_of<T>` を実行する前に `max_alloc_payload_bytes / size_of<T>` から最大 count を求め、wrap した byte 数を allocator へ渡さない。

この作業中に、`dealloc` / `realloc` の size 引数を runtime check だけで強化すると Resource IR owner summary が正しく maybe leak を報告することを確認した。これは dealloc size と allocation extent の compiler-level proof が不足している別問題なので、`ISS-20260513T101719832Z-DEALLOC-AND-REALLOC-SIZE-ARGUMENTS-N-D7EADBBD` として分離した。

## 2026-05-13 Agent 1 self-host lexer Vec.data field read 追記

`ISS-20260513T215609976Z-SELF-HOST-LEXER-READS-VEC-RAW-DATA-F-8A56A6A1` で、self-host lexer の indent stack 操作が `Vec.data` raw storage field を直接読んでいた問題を分離して修正した。

`lex_stack_drop_top` は `Vec<i32>` の末尾を捨てるだけなので、`field::get stack "data"` による layout 再構成ではなく `drop_last<i32>` の public owner API で次の `Vec` owner を返す形へ変更した。これにより self-host compiler の syntax 層へ `Vec` の transitional `MemPtr` storage layout を持ち込まない。`drop_last<T>` 自体は `Vec` module 内に置き、現行 `Vec` の moved/uninitialized cell 制約に合わせて `.T: Copy` に限定した。

focused verification の過程で、Resource owner checker が non-Copy `Read` を owner move として扱わず、`push` の `Result::Ok` payload owner summary でも fresh / parameter-derived の複数候補が parameter 消費を隠す問題が露出した。これは stdlib allowlist で回避せず、compiler 側で owner transfer と variant projection return 正規化を修正した。Stage 6 の public API boundary 修正は、Resource IR の証明を弱めずに進める必要がある。

この親 issue は引き続き open とする。stdlib 全体にはまだ raw-memory-backed public API / owner token / `OwnedBuffer<T>` 移行が残るため、個別の安全境界を閉じながら Stage 6 を継続する。

## 2026-05-15 Agent 1 region_new doctest forged-token fixture 追記

`ISS-20260514T152732869Z-CORE-MEM-INTERNAL-REGION-NEW-DOCTEST-F1D709F2` で、`core/mem/internal.nepl` の `region_new` doctest が固定 raw address から `RegionToken<u8>` を構築する例を示していた問題を修正した。

`region_new` の正常例は `alloc_ptr<u8>` 由来の allocator-issued pointer から `RegionToken<u8>` を作り、最後に `dealloc_region<u8>` で free obligation を閉じる形へ変更した。あわせて source policy regression を追加し、canonical internal doctest が `region_new mem_ptr_wrap` や non-zero fixed raw address wrapping を正常例として再導入しないようにした。

これは `RegionToken<T>` の forgeability 本体を閉じる修正ではないため、この親 issue は open のまま継続する。Stage 6 の残件は引き続き compiler-issued owner token / `OwnedBuffer<T>` / initialized prefix / collection drop traversal である。

## 2026-05-18 Agent 1 Vec doctest raw address observer fixture 追記

`ISS-20260518T195527913Z-VEC-DOCTEST-RAW-ADDRESS-OBSERVATI-6CAB0F41` で、通常の `Vec` doctest と `data_mem_ptr` 使用例が `core/mem/internal` / `mem_ptr_addr` によって raw backing address を観測していた問題を修正した。

この変更は `data_mem_ptr<T>(&Vec<T>) -> MemPtr<T>` を final safe API として承認するものではない。現行 Stage 6 では `data_mem_ptr` は Copy payload に限定された typed non-owning view observer であり、通常の public behavior test は raw address positivity ではなく `with_capacity` / `is_empty` / `push` / `get` / `free` で検証する。

source policy は `stdlib/tests/vec.n.md` と `data_mem_ptr` の usage doctest に raw address observer が戻らないことを監視する。raw-memory-boundary の正当性は stdlib test fixture の internal import ではなく、compiler の source proof と typed Resource IR によって検査する。

## 2026-05-15 Agent 1 owner aggregate constructor capability 名称化追記

`ISS-20260515T020307026Z-OWNER-AGGREGATE-CONSTRUCTOR-CAPABILI-91ECE78D` で、owner aggregate constructor capability が file-wide bool として過大付与されていた問題を修正した。

修正前は compiler-owned stdlib source に unqualified constructor-like symbol が 1 つでもあると、その file 全体が owner-backed aggregate constructor boundary になった。これでは `Diag` など unrelated constructor の evidence だけで、同じ source 内の `Vec` / `HashMap` / owner wrapper direct constructor を許し得る。

修正後は `OwnerAggregateConstructorBoundary(String)` を `SourceCapabilities` に保持し、loader は constructor 名ごとの evidence を記録する。typecheck も実際に適用している constructor 名を照合するため、source property proof と許可される owner-backed constructor が一致する。この親 issue は引き続き open とする。raw-memory-backed public API / `OwnedBuffer<T>` / initialized prefix / collection drop traversal は Stage 6 残件として継続する。

## 2026-05-15 Agent 1 checked owner helper evidence 除外追記

`ISS-20260515T023829013Z-CHECKED-OWNER-HELPERS-GRANT-RAW-MEMO-EC25363F` で、`alloc_ptr` / `alloc_region` / `dealloc_region` などの checked owner wrapper 呼び出しを raw memory boundary evidence として扱っていた問題を修正した。

これらの wrapper は safe public API であり、stdlib implementation が allocation API を使うだけで raw intrinsic / raw body memory operation / restricted constructor authority を得るべきではない。修正後は actual raw operation、raw address identity helper、restricted compiler memory constructor、raw address intrinsic のみを raw boundary evidence とする。

この親 issue は引き続き open とする。今回閉じたのは source capability の過大付与であり、`OwnedBuffer<T>` / compiler-issued owner token / initialized prefix / collection drop traversal は Stage 6 残件として継続する。

## 2026-05-15 Agent 1 Vec empty cleanup storage-state 追記

`ISS-20260514T155034305Z-VEC-EMPTY-CLEANUP-TREATS-ZERO-CAPACI-EACF73AF` で、`Vec` の empty storage cleanup を storage state の match に戻した。

`VecStorageState::Empty` は allocation を持たない状態であり、zero-size `RegionToken<T>` sentinel を owner-consuming `dealloc_region` へ渡すべきではない。修正後の `vec_free_storage<T>` は `Empty` を no-op、`Owned` を `dealloc_region` として分岐するため、raw-memory-backed public API の過渡設計でも owner obligation の有無が `VecStorageState` の enum に現れる。

これは `RegionToken<T>` を compiler-issued token へ置き換える最終修正ではないため、この親 issue は open のまま継続する。次の主対象は `OwnedBuffer<T>` / initialized prefix / forged token API 廃止である。

## 2026-05-16 Agent 1 Vec empty cleanup final state 追記

`ISS-20260515T223330574Z-VEC-STORAGE-TAG-AND-REGIONTOKEN-OWNE-DDDAD134` の解決により、`VecStorageState` と split `RegionToken<T>` field は `VecStorage<T>::Empty | Owned(RegionToken<T>)` へ統合された。現在の `vec_free_storage<T>` は `VecStorage<T>` を消費し、`Empty` branch では owner payload が存在せず、`Owned region` branch だけが `dealloc_region` へ進む。

そのため `ISS-20260514T155034305Z-VEC-EMPTY-CLEANUP-TREATS-ZERO-CAPACI-EACF73AF` は fixed とする。この親 issue は open のまま継続する。残る対象は `RegionToken` の forgeability、compiler-issued `OwnedBuffer<T>`、initialized prefix、non-Copy payload drop traversal である。

## 2026-05-15 Agent 1 Vec empty sentinel helper private 化追記

`ISS-20260514T155620178Z-VEC-EMPTY-REGIONTOKEN-SENTINEL-HELPE-B3CF72E9` で、`vec_empty_region<T>` を public API から外した。

`vec_empty_region<T>` は `VecStorageState::Empty` を現行 struct layout に載せるための内部 helper であり、raw-memory-backed public API として公開すると caller が `RegionToken<T>` sentinel construction に依存できてしまう。修正後は `vec_empty<T>` だけを public typed constructor とし、zero-size sentinel helper は `storage/view.nepl` 内部に閉じる。

これは `RegionToken<T>` の forgeability 全体を閉じる修正ではないが、Stage 6 の public surface から不要な owner-token constructor を 1 つ減らす前進である。親 issue は `OwnedBuffer<T>` / compiler-issued token への移行が残るため open のまま継続する。

## 2026-05-15 Agent 1 Vec/KP raw i32 address public API 削除追記

`ISS-20260514T160255919Z-VEC-DATA-PTR-EXPOSES-RAW-I32-STORAGE-546EA2EB` で、`Vec.data_ptr<T>(&Vec<T>) -> i32` を削除した。互換 alias は残さず、呼び出し側は `data_mem_ptr<T>(&Vec<T>) -> MemPtr<T>` を使い、raw-memory-boundary 実装箇所でだけ明示的に `mem_ptr_addr` へ落とす。

同じ根として、`kpsearch` の raw `i32` pointer helper は public API から外し、公開面は `Vec<i32>` owner を消費する wrapper に揃えた。これにより ordinary source が KP の探索 API を使うために raw address を保持・組み立てる必要がなくなった。

## 2026-05-15 Agent 1 Vec storage MemPtr helper 削除追記

`ISS-20260514T161819706Z-VEC-STORAGE-MEMPTR-HELPER-EXPOSES-LO-A9C5BC02` で、`vec_storage_mem_ptr<T>(VecStorageState, &RegionToken<T>) -> MemPtr<T>` を削除した。公開 API は `data_mem_ptr<T>(&Vec<T>)` に集約し、storage state の `Empty` / `Owned` match はその observer boundary が直接所有する。

これにより caller が `VecStorageState` と `RegionToken` 参照を組み合わせて lower-level storage projection helper を呼ぶ経路を閉じた。`data_mem_ptr` 自体はまだ Copy-only raw storage view observer であり、`OwnedBuffer<T>` / borrow projection の残件は継続する。

## 2026-05-15 Agent 1 owner-backed aggregate boundary 追記

`ISS-20260514T164856024Z-OWNER-BACKED-AGGREGATE-CONSTRUCTORS--61400B84` で、`RegionToken<T>` を direct field に持つ aggregate を user source が直接構築・投影できる経路を compiler 側で閉じた。

`Vec<T>` は `region: RegionToken<T>` へ移行済みだが、struct constructor が public のままだと、caller は allocator-issued owner token ではない値を aggregate に詰めたり、`field::get_ref &v "region"` で free obligation owner を取り出したりできる。今回の修正では、特定 stdlib 名の allowlist ではなく、struct field 型が compiler owner token を含むかを typecheck が判定し、`OwnerBackedAggregateBoundaryOnly` policy を付与する。

許可境界は `OwnerAggregateConstructorBoundary` / `OwnerAggregateFieldBoundary` source capability として `RawMemoryBoundary` から分けた。configured stdlib root 配下でも無条件には付与せず、parsed source に aggregate constructor / field accessor の evidence がある場合だけ、対応する capability を付与する。stdlib 実装 source は owner aggregate の move/reconstruct/projection が必要だが、raw memory operation authority まで得るべきではないためである。これにより Stage 6 の「owner token は compiler が性質を証明した境界内でだけ扱える」という前提を強めつつ、raw-memory-backed public API migration はこの親 issue で継続する。

## 2026-05-15 Agent 1 empty RegionToken sentinel helper 追記

`ISS-20260514T171944501Z-BYTEBUF-AND-BYTEBUILDER-EXPOSE-EMPTY-6E06A830` で、`byte_builder_empty_region` / `io_bytebuf_empty_region` が public re-export されていた問題を分離して修正した。

今回の修正では、zero-size `RegionToken<u8>` sentinel helper を private にし、公開 API を `byte_builder_empty -> ByteBuilder` / `io_bytebuf_empty -> ByteBuf` に限定した。これは `Vec` の `vec_empty_region<T>` private 化と同じ方針であり、transitional owner-token sentinel を safe public surface に出さないための Stage 6 前進である。

## 2026-05-18 Agent 1 ByteBuf structural empty storage 追記

`ISS-20260429T131646897Z-BYTEBUF-EMPTY-NON-EMPTY-CONDITIONAL--34FBA0C2` で、`ByteBuf` の empty storage を private sentinel helper から完全に外し、`ByteBufStorage::Empty | Owned(RegionToken<u8>)` へ移した。

これにより `ByteBuf` は空状態で free obligation owner payload を持たず、`io_bytebuf_free` / `io_bytebuf_data_ptr_ref` は enum `match` の網羅性で empty branch と owned branch を分ける。当時 `ByteBuilder` 側には empty `RegionToken` sentinel が残っていたため、この親 issue の後続作業として `ByteBuilderStorage` または `OwnedBytes` / compiler-issued owner token への移行を継続することにした。

同じ focused verification で、`std/fs/dir/read_fd.nepl` が削除済みの `Vec.data` field と `Vec<str>` raw storage sort に依存していることを確認した。これは別 issue `ISS-20260514T172450328Z-FS-DIR-READER-STILL-DEPENDS-ON-RAW-V-05400C14` として記録し、この親 issue の raw-memory-backed stdlib API migration 残件として継続する。

## 2026-05-15 Agent 1 fs dir reader Vec boundary 追記

`ISS-20260514T172450328Z-FS-DIR-READER-STILL-DEPENDS-ON-RAW-V-05400C14` で、`std/fs/dir/read_fd.nepl` の旧 `Vec.data` layout 依存を削除した。

`fs_sort_strings` は raw `i32` storage pointer を受け取らず、`&Vec<str>` を受け取って `v::len` / `v::get` / `v::replace` 経由で stable insertion sort を行う。`str` は所有権を持たない Copy view として扱い、directory reader 側は sort error 時に `Vec<str>` owner を解放して `Err(e)` を返す。これにより `std/fs` import が削除済み Vec field で compile failure になる経路と、fs module が raw string storage sort に依存する経路を閉じた。

この修正は Stage 6 の public API migration の一部であり、raw-memory-backed stdlib 全体の `OwnedBuffer<T>` / compiler-issued owner token / initialized prefix migration は引き続きこの親 issue で継続する。

## 2026-05-15 Agent 1 std fs/stdio raw facade boundary 追記

`ISS-20260514T182018325Z-STD-FS-AND-STDIO-ROOT-FACADES-RE-EXP-9492D2E7` として、safe root facade の `std/fs` と `std/stdio` が raw ABI submodule を再公開していた問題を分離して修正した。

今回の修正では、`std/fs` root から `pub #import "./fs/raw" as *` を削除し、`std/stdio` root から `pub #import "./stdio/raw" as *` を削除した。fd/read/write の実装 module は `std/fs/raw` / `std/stdio/raw` を明示 import するため、ABI 境界は explicit raw submodule に閉じる。通常の `std/fs` / `std/stdio` import は filesystem / standard I/O の safe public API だけを公開する。

この親 issue は引き続き open とする。今回の修正は raw ABI helper の root re-export を閉じるものであり、raw-memory-backed stdlib 全体の `OwnedBuffer<T>` / compiler-issued owner token / initialized prefix migration は残件である。

## 2026-05-15 Agent 1 std env cliarg raw boundary 追記

`ISS-20260514T183506445Z-STD-ENV-CLIARG-ROOT-MIXES-RAW-ARGV-B-C76C9E1E` として、safe root facade の `std/env/cliarg` が raw argv scratch / out pointer 処理を直接持っていた問題を分離して修正した。

今回の修正では、root `std/env/cliarg` から `core/mem/raw` / `core/mem/internal` の直接 import を削除し、`cliarg_count` / `cliarg_get` を `std/env/cliarg/raw` の qualified helper へ委譲する thin facade にした。`args_sizes_get` / `args_get`、`mem_ptr_addr`、argv slot 初期化、raw slot load は `cliarg_count_result` / `cliarg_get_checked` に集約した。

また、C string conversion helper は root facade から暗黙に露出させず、必要な doctest は `std/env/cliarg/cstr` を明示 import する形に整理した。cstr doctest は `alloc_ptr` owner を直接扱う例から、`RegionToken<u8>` owner と `region_ptr` non-owning view を使う例へ更新し、Resource IR の owner obligation と整合させた。

この親 issue は引き続き open とする。今回の修正は env argv ABI helper の root direct raw 実装を閉じるものであり、raw-memory-backed stdlib 全体の `OwnedBuffer<T>` / compiler-issued owner token / initialized prefix migration は残件である。

## 2026-05-15 Agent 1 diag renderer Vec boundary 追記

`ISS-20260514T185052018Z-DIAG-RENDERER-READS-DIAGS-VEC-STORAG-D85114C9` として、`alloc/diag/diag.nepl` の public diagnostic renderer が `Diags.items` を raw `Vec` storage として直接走査していた問題を分離して修正した。

今回の修正では、renderer file から `core/mem` / `core/mem/internal` / `core/mem/raw` import を削除し、`diags_to_string` を borrowed `Vec<Diag>` + `v::len<Diag>` + `v::get<Diag>` の Copy-safe observer boundary へ移した。`Diag` は `Copy` として定義済みなので、renderer が `mem_ptr_addr data_mem_ptr<Diag>` と `load<Diag>` を直接使う必要はない。

これは `Diags` storage owner の内部実装を消す修正ではない。`alloc/diag/error/diags.nepl` はまだ `Diags` owner helper として raw storage scanner を持つが、ordinary formatting / print API から raw Vec layout 依存を外し、Stage 6 の public renderer / storage boundary 分離を進めた。親 issue は `OwnedBuffer<T>` / compiler-issued owner token / initialized prefix / collection drop traversal が残るため open のまま継続する。

## 2026-05-15 Agent 1 Diags error observer Vec boundary 追記

`ISS-20260514T190018299Z-DIAGS-ERROR-OBSERVER-SCANS-VEC-STORA-5ABF687A` として、`alloc/diag/error/diags.nepl` の `diags_has_errors` が `Vec<Diag>` storage を raw address と `load<Diag>` で直接走査していた問題を分離して修正した。

今回の修正では、`diags.nepl` から `core/mem` / `core/mem/internal` / `core/mem/raw` import を削除し、`diags_has_errors` を borrowed `Vec<Diag>` + `v::len<Diag>` + `v::get<Diag>` の Copy-safe observer traversal へ移した。by-value observer はこれまで通り観測後に `diags_free` で owner を閉じるため、owner cleanup contract は維持される。

これにより diagnostic module の read-only observer から raw Vec storage scan が消えた。`Vec` 本体の raw-memory-backed implementation と `OwnedBuffer<T>` / compiler-issued owner token / initialized prefix migration は引き続きこの親 issue の Stage 6 残件として継続する。

## 2026-05-15 Agent 1 kpprefix Vec owner boundary 追記

`ISS-20260514T195643983Z-KPPREFIX-EXPOSES-COPYABLE-RAW-PREFIX-416FBAB5` として、`kp/kpprefix` が raw prefix storage owner を public Copy handle として公開していた問題を分離して修正した。

`PrefixI32` は `ptr <i32>` / `len <i32>` を持つ `Copy` / `Clone` handle ではなく、`data <Vec<i32>>` を持つ owner handle に変更した。これに合わせて `prefix_build_i32` / `prefix_range_sum_i32` の public raw address API は削除し、公開面を `prefix_build_vec_i32(Vec<i32>) -> Result<PrefixI32, Diag>` と `prefix_sum_i32(&PrefixI32, i32, i32) -> Result<i32, Diag>` に揃えた。

構築処理は `vec::filled` で初期化済み prefix buffer を確保し、`vec::get` / `vec::replace` で累積和を埋める。query も `vec::get` を使い、範囲外は `Diag` を返す。これにより `kpprefix` 自体は `core/mem/raw` を import せず、raw storage identity は `Vec` の実装境界に閉じる。

この修正は `OwnedBuffer<T>` / compiler-issued owner token の最終移行ではないが、ordinary KP helper が raw address と copyable deallocation handle を public surface へ漏らす経路を閉じる Stage 6 の前進である。親 issue は raw-memory-backed stdlib API 全体の移行が残るため open のまま継続する。

## 2026-05-15 Agent 1 kpfenwick/kpdsu owner boundary 追記

`ISS-20260514T200755109Z-KPFENWICK-AND-KPDSU-EXPOSE-RAW-I32-O-953345F8` として、`kp/kpfenwick` と `kp/kpdsu` が allocation owner を public raw `i32` handle として公開していた問題を分離して修正した。

`kpfenwick` は raw `i32` handle を返す `fenwick_new`、raw handle を受ける `fenwick_free` / `fenwick_add` / query API を廃止した。公開面は `Fenwick` owner、`FenwickAddError` owner-preserving update error、`Diag` query error に揃えた。実装は raw memory helper を使わず、`alloc/collections/fenwick` の typed storage helper / mutation helper / query helper / diagnostic helper に委譲する。

`kpdsu` も raw parent/size storage handle を public `i32` として扱う構成を削除し、`DisjointSet` owner と `DisjointSetUpdateError` を使う `alloc/collections/disjoint_set` facade へ委譲した。query は `&DisjointSet` を読み取り、update は owner を消費して返すため、public API 上で owner/free obligation が型に残る。

この修正は `kpgraph` など残る KP raw-memory-backed module の最終移行ではない。親 issue は引き続き open とし、ordinary stdlib/KP API が raw storage identity を公開しない状態まで Stage 6 を継続する。

## 2026-05-15 Agent 1 kpgraph dense matrix owner boundary 追記

`ISS-20260514T202200590Z-KPGRAPH-EXPOSES-DENSE-MATRIX-RAW-I32-F15E55CB` として、`kp/kpgraph` が dense matrix allocation を public raw `i32` handle として公開していた問題を分離して修正した。

`DenseGraph` は `mat <i32>` ではなく `matrix <AdjacencyMatrix>` を保持する owner wrapper になった。構築は `Result<DenseGraph, Diag>`、更新は `Result<DenseGraph, DenseGraphUpdateError>`、BFS は `&DenseGraph` から `Result<Vec<i32>, Diag>` を返す。旧 `dense_graph_bfs_dist_raw(n, mat, start)` は互換 alias を残さず削除した。

BFS の距離配列と queue は `Vec<i32>` で初期化し、`v::get` / `v::replace` / `v::free` を使う。doctest も returned `Vec<i32>` の raw storage を `mem_ptr_addr` / `load_i32` で読む例をやめ、`v::get<i32>` で結果を表示する。

これにより KP graph helper の public surface から raw matrix pointer と raw Vec storage read の入口を閉じた。残る KP raw-memory-backed module では `kpsearch` の internal raw helper / public Vec wrapper 境界が次の確認対象である。

## 2026-05-15 Agent 1 kpgraph source policy 追記

`ISS-20260515T004634650Z-KPGRAPH-UNSAFE-UNWRAP-POLICY-STILL-E-1F2C94F7` として、`nodesrc/test_stdlib_kpgraph_no_unsafe_unwraps.js` が旧 raw BFS helper の存在を要求し続けていた問題を分離して修正した。

現在の `kpgraph` では `dense_graph_bfs_dist_raw` / `kp_push_i32` / `kp_i32_empty_vec` を戻さず、`dense_graph_bfs_dist(&DenseGraph, i32) -> Result<Vec<i32>, Diag>` を検査する。source policy は距離配列と queue の `v::filled` allocation failure、queue allocation failure 時の `dist` owner cleanup、`v::get` / `v::replace` による typed access、storage invariant failure 時の `dist` free を監視する。

## 2026-05-15 Agent 1 kpsearch Vec API boundary 追記

`ISS-20260514T203142216Z-KPSEARCH-STILL-IMPLEMENTS-PUBLIC-VEC-0D1AFA1D` として、`kp/kpsearch` が public Vec wrapper の内部実装で raw storage view に依存し続けていた問題を分離して修正した。

今回の修正で `kpsearch` から `core/mem` / `core/mem/internal` / `core/mem/allocator` / `core/mem/raw` import、`mem_ptr_addr data_mem_ptr`、raw `i32` helper を削除した。`lower_bound_vec_i32` / `upper_bound_vec_i32` / `contains_vec_i32` / `count_equal_range_vec_i32` は `&Vec<i32>` を受ける borrowed query API に変え、`Vec.len` / `Vec.get` だけで二分探索を行う。

`unique_sorted_vec_i32` は sorted Vec を in-place に圧縮するため owner-consuming API のまま残し、内部は `Vec.get` / `Vec.replace` で compaction する。caller は query 後に入力 owner を保持したまま free でき、unique 後は `UniqueSortedVecI32` が owner を保持する。

これにより KP module 群の public graph/search/prefix/DSU/Fenwick helper から raw storage identity を直接扱う入口を閉じた。今後は `Vec` / collection 本体の internal raw-memory-backed implementation と `OwnedBuffer<T>` / compiler-issued owner token migration を継続する。

## 2026-05-15 Agent 1 Vec sort facade raw boundary 追記

`ISS-20260514T204735670Z-VEC-SORT-FACADE-RE-EXPORTS-RAW-MEMPT-6646B4EF` として、canonical `alloc/collections/vec/sort` facade が raw `MemPtr` helper と raw slice sort adapter を再公開していた問題を分離して修正した。

今回の修正では、unchecked read/write/swap をいったん `sort/raw/access`、raw quick-sort traversal を `sort/raw/quick`、raw heap helper を `sort/raw/heap` に移し、safe root facade は `sort_quick` / `sort_heap` / `sort_merge` / simple sort / `sort_is_sorted` などの `Vec` API だけを公開する構成にした。`sort_i32` は raw address discipline を ordinary sort facade に固定する入口だったため、互換 alias を残さず削除した。

`sort_is_sorted` は `Vec.get` / `Option` による borrowed observer に変更し、`sort/merge` root facade も public API だけを再公開する。当時は raw traversal を必要とする implementation module が explicit raw submodule を import する構成にしたが、後続の 2026-05-19 追記で direct-importable raw submodule 自体も削除した。

## 2026-05-19 Agent 1 Vec sort direct import raw boundary 追記

`ISS-20260519T130927391Z-VEC-SORT-RAW-HELPERS-ARE-DIRECTLY-CA-BE6B177C` として、`alloc/collections/vec/sort/raw` を ordinary source から明示 import できること自体が bypass になる問題を追加で修正した。

今回の修正では、`sort/raw` facade と `sort/raw/access` / `sort/raw/quick` / `sort/raw/heap` を削除した。quick / heap / simple / merge range の raw `load` / `store` / swap traversal は、それぞれ範囲、`Vec` storage view、scratch owner を扱う検査済み implementation file 内の private helper に閉じ、public/direct-importable な unchecked sort helper 名を残さない。

これにより、ordinary caller は `data_mem_ptr(&v)` と `sort_set_unchecked_data` のような helper を組み合わせて `Vec.len` / initialized slot discipline を迂回できない。source policy は `sort/raw` 復活、`sort_get_unchecked*` / `sort_set_unchecked*` / `sort_swap*` / `sort_slice_quick` の shared helper 復活、private raw traversal helper の `pub fn` 化を拒否する。

この親 issue は引き続き open とする。今回閉じたのは Vec sort facade の raw re-export であり、`OwnedBuffer<T>` / compiler-issued owner token / initialized prefix / non-Copy payload drop traversal は Stage 6 の残件である。

## 2026-05-15 Agent 1 owner aggregate source capability 精度追記

`ISS-20260514T211956079Z-OWNER-AGGREGATE-BOUNDARY-TREATS-QUAL-8D858CD3` として、compiler の source capability 判定が `Result::Ok` などの qualified enum variant を owner aggregate constructor evidence と誤分類していた問題を分離して修正した。

owner aggregate constructor capability は owner-backed aggregate constructor を許可する capability なので、configured stdlib source であっても通常の enum variant construction だけを証拠にして付与してはいけない。修正後は constructor-like evidence を unqualified symbol に限定し、`Result::Ok` / `Option::Some` のような qualified enum variant は capability 証拠から外した。`field::get` / `field::get_ref` などの explicit field accessor evidence と、`Vec` のような unqualified owner aggregate constructor evidence は維持している。

これにより Stage 6 の raw-memory-backed stdlib migration を支える compiler 側の source proof が過大付与にならず、ordinary result/option construction が owner aggregate boundary の静的検査を緩める経路を閉じた。親 issue は引き続き open とし、`OwnedBuffer<T>` / compiler-issued owner token / initialized prefix / collection drop traversal の最終移行を継続する。

## 2026-05-15 Agent 1 owner aggregate capability 分割追記

`ISS-20260514T212804383Z-OWNER-AGGREGATE-CONSTRUCTOR-AND-OWNE-58143AB3` として、compiler の `OwnerAggregateBoundary` が owner-backed aggregate constructor と owner token field projection を 1 つの file-wide capability で共有していた問題を分離して修正した。

今回の修正では `SourceCapability::OwnerAggregateBoundary` を削除し、`OwnerAggregateConstructorBoundary` と `OwnerAggregateFieldBoundary` に分けた。source evidence walker は constructor-like evidence と field accessor evidence を別々に判定し、loader は対応する capability だけを付与する。typecheck 側も direct constructor と owner token field projection で別 method を見るため、field accessor evidence だけの source が constructor 権限まで得たり、constructor evidence だけの source が owner token projection 権限まで得たりしない。

この修正は Stage 6 の compiler 側 source proof 精度を上げるものであり、stdlib public API の最終移行ではない。親 issue は `OwnedBuffer<T>` / compiler-issued owner token / initialized prefix / collection drop traversal が残るため open のまま継続する。

## 2026-05-15 Agent 1 Vec empty Copy-only boundary 追記

`ISS-20260514T215003679Z-VEC-EMPTY-CONSTRUCTOR-ACCEPTS-NON-CO-258C7574` として、`Vec` の zero-allocation public constructor `vec_empty<T>` が non-Copy payload を受け入れていた問題を分離して修正した。

Empty state は runtime allocation を持たないが、public API としては `VecStorageState::Empty` と private `RegionToken<T>` sentinel を持つ `Vec<T>` owner aggregate を生成する。`Vec.free` / `clear` / `push` / `pop` / raw element helper は現行 `OwnedBuffer<T>` 未完成のため Copy-only に閉じているので、`vec_empty<T>` だけを generic に残すと raw-memory-backed collection migration の safe surface が unsupported `Vec<NonCopyPayload>` を作れてしまう。

今回の修正では `vec_empty<T: Copy>` とし、collection cleanup contract の compile-fail regression と source policy で固定した。これは Stage 6 の中間安全境界であり、親 issue は `OwnedBuffer<T>` / compiler-issued owner token / initialized prefix / collection drop traversal の最終移行が残るため open のまま継続する。

## 2026-05-15 Agent 1 std/text raw UTF-8 facade boundary 追記

`ISS-20260514T223113919Z-STD-TEXT-ROOT-RE-EXPORTS-RAW-UTF-8-M-7F3A2723` として、`std/text` root が raw UTF-8 validation / decode helper を public `@merge` していた問題を分離して修正した。

今回の修正では、root `std/text` を checked `ByteBuf -> str` conversion facade に戻し、`text/validate` / `text/decode` は explicit submodule import 境界へ閉じた。raw decode / encode を本当に検証する doctest は `std/text/decode` を明示 import し、ordinary conversion doctest は root facade だけを使う。

invalid UTF-8 fixture も raw `i32` address store から checked `MemPtr` `store_u8` / `dealloc_ptr` cleanup へ移した。これは UTF-8 converter が invalid byte を拒否する性質を維持しつつ、通常 doctest が raw memory authority を要求しないようにする Stage 6 の public/raw boundary split である。

focused consumer verification 中に `std/io` doctest が `WriteStream` の定義元を import していない既存 drift を確認したため、`ISS-20260514T223843320Z-STD-IO-DOCTEST-OMITS-EXPLICIT-IOTARG-12D221C3` として分離した。この親 issue は `OwnedBuffer<T>` / compiler-issued owner token / initialized prefix / collection drop traversal が残るため open のまま継続する。

## 2026-05-15 Agent 1 transitive owner aggregate policy 追記

`ISS-20260514T230404748Z-OWNER-BACKED-AGGREGATE-POLICY-DOES-N-7D995A6B` として、compiler の owner-backed aggregate constructor policy が direct owner token field にしか効かず、nested owner field を持つ aggregate に伝播しない問題を分離して修正した。

今回の修正では、`OwnerBackedAggregateBoundaryOnly` の判定を fixed-point の構造判定に変更した。`Vec<T>` のように `RegionToken<T>` を直接持つ型だけでなく、`Vec<T>` を field に持つ user wrapper、`HashMapStorage<K,V>`、さらに `HashMap<K,V,H>` のような二段目以降の aggregate も owner-backed として扱う。これにより、通常 user source が `HashMapStorage` や `HashMap` constructor を直接呼び出し、collection storage state / count / capacity の不変条件を再構築する経路を typecheck で拒否する。

この修正は stdlib 名の allowlist ではなく、compiler owner token policy と struct field 型から性質を導出する compiler-core 側の防壁である。親 issue は raw-memory-backed stdlib API 全体の `OwnedBuffer<T>` / compiler-issued owner token / initialized prefix / collection drop traversal が残るため open のまま継続する。

## 2026-05-15 Agent 1 owner aggregate field projection 追記

`ISS-20260514T231627302Z-OWNER-BACKED-AGGREGATE-FIELD-PROJECT-290DED97` として、constructor 側で閉じた owner-backed aggregate を field projection から取り出せる問題を分離して修正した。

今回の修正では、`typecheck/field_access.rs` も `target_contains_owner_backed_aggregate` を使い、base または projected field が owner-backed aggregate の場合は `OwnerAggregateFieldBoundary` capability を要求する。これにより通常 user source が `HashMap.storage` や `Vec` wrapper の `items` field を `field::get` で取り出し、collection/storage owner invariant を public API の外へ分解する経路を拒否する。

同時に owner-backed aggregate 判定を generic application / enum payload / tuple / box まで再帰する構造判定に拡張した。これは stdlib path allowlist ではなく型構造から性質を導出する compiler-core 側の防壁であり、親 issue は `OwnedBuffer<T>` / compiler-issued owner token / initialized prefix / collection drop traversal が残るため open のまま継続する。

## 2026-05-15 Agent 1 generic owner aggregate constructor 追記

`ISS-20260514T233136936Z-GENERIC-OWNER-BACKED-AGGREGATE-CONST-6E024598` として、generic type application 後に owner-backed になる aggregate constructor が boundary を迂回できる問題を分離して修正した。

`StructConstructorPolicy` は definition に付くため、`struct OwnerBox<.T>: item <.T>` のような generic definition は policy 上は public のまま残る。しかし `OwnerBox<Vec<i32>>` は適用後に owner-backed aggregate になるため、constructor call 時の concrete result type を構造判定へ通して `OwnerAggregateConstructorBoundary` を要求するようにした。

これにより constructor と field projection は同じ `target_contains_owner_backed_aggregate` に基づき、generic wrapper だけで collection owner / storage owner invariant を public source 側へ持ち出す経路を閉じる。親 issue は `OwnedBuffer<T>` / compiler-issued owner token / initialized prefix / collection drop traversal が残るため open のまま継続する。

## 2026-05-15 Agent 1 HashMap/HashSet facade 追記

`ISS-20260514T234418963Z-HASHMAP-AND-HASHSET-ROOT-FACADES-RE--68724B49` として、`alloc/collections/hashmap` / `alloc/collections/hashset` の root safe facade が storage/probe/rehash implementation helper を public merge していた問題を分離して修正した。

今回の修正では、root facade を public type/API の再公開に限定し、storage allocation や probing helper は implementation module の明示 import 境界だけで見えるようにした。これは compiler の owner-backed aggregate 構造判定を stdlib public surface 側から補強するものであり、safe root import から storage owner helper を直接呼べる経路を閉じる。

この親 issue は raw-memory-backed stdlib API 全体の migration issue として open のまま継続する。残件は `OwnedBuffer<T>`、initialized cell、compiler-issued owner token、non-Copy payload drop traversal を含む最終 collection memory model である。

## 2026-05-15 Agent 1 Vec storage facade 追記

`ISS-20260515T000336641Z-VEC-STORAGE-FACADE-RE-EXPORTS-ALLOCA-4F004371` として、`alloc/collections/vec` root から `vec/storage` 経由で `vec_alloc_empty` / `vec_free_storage` が見えていた問題を分離して修正した。

今回の修正では、public allocation constructor を `vec/storage/api.nepl` へ分け、`vec/storage.nepl` から `storage/alloc` / `storage/cleanup` の public re-export を削除した。root facade は `new` / `with_capacity` / `filled` / `vec_empty` などの public API を維持するが、allocation helper と storage-only cleanup helper は explicit implementation module import に限定される。

この親 issue は引き続き open とする。今回閉じたのは root public surface の漏れであり、`OwnedBuffer<T>`、initialized cell、compiler-issued owner token、non-Copy payload drop traversal の最終移行は継続する。

## 2026-05-15 Agent 1 alloc/string facade source policy 追記

`ISS-20260515T002636772Z-ALLOC-STRING-FACADE-SOURCE-POLICY-ST-1530FB1C` として、`alloc/string` root の source policy が Stage 6 の修正後も `string/storage` / `string/utf8` の public re-export を要求していた問題を分離して修正した。

今回の修正では、root safe facade が再公開する module を `access` / `builder` / `search` / `slice` / `split` / `integer` / `float` / `concat` / `builder_ext` / `find` に限定し、`storage` / `utf8` は root から再公開されないことを policy で固定した。一方で `storage` / `utf8` 自体は explicit raw-boundary import 用 module として存在し、raw memory boundary evidence を持つことも検査する。

この親 issue は引き続き open とする。今回閉じたのは policy 側の古い期待であり、raw-memory-backed stdlib 全体の `OwnedBuffer<T>` / compiler-issued owner token / initialized prefix migration は継続する。

## 2026-05-15 Agent 1 Vec sort/merge source policy 追記

`ISS-20260515T003514038Z-VEC-SORT-MERGE-SOURCE-POLICY-STILL-E-BD811427` として、`Vec` merge sort の unsafe unwrap policy が `sort/merge` facade から `merge/buffer` / `merge/range` の raw helper を再公開することを要求していた問題を分離して修正した。

今回の修正では、`sort/merge.nepl` は `merge/api` だけを再公開し、`merge/buffer` / `merge/range` は explicit raw-boundary implementation module として残ることを policy にした。`merge/buffer` の Copy-only scratch buffer helper、`merge/range` の Copy-only traversal、`merge/api` の explicit `./range` import も同じ policy で監視する。

2026-05-19 の `ISS-20260519T134548652Z-VEC-MERGE-SORT-RAW-HELPERS-ARE-DIREC-18BA8A0F` で、この設計はさらに更新した。facade から re-export しないだけでは、ordinary source が `alloc/collections/vec/sort/merge/buffer` や `alloc/collections/vec/sort/merge/range` を明示 import して unchecked `MemPtr` helper を呼べるため、raw helper module 自体を削除した。merge scratch access と range traversal は `merge/api.nepl` の private helper に統合し、public caller は `sort_merge` / `sort_merge_ret` だけを通る。

この親 issue は引き続き open とする。今回閉じたのは policy 側の古い re-export 期待であり、`OwnedBuffer<T>` / compiler-issued owner token / initialized prefix / non-Copy payload sort の最終移行は継続する。

## 2026-05-15 Agent 1 raw memory operation capability 分離追記

`ISS-20260515T024851827Z-RAW-MEMORY-OPERATIONS-SHARE-A-FILE-W-2A06192D` として、compiler の raw memory source capability が file-wide raw boundary へ潰れていた問題を分離して修正した。

今回の修正では、raw source authority を `RawMemoryStructuralBoundary`、`RawMemoryOperationBoundary(RawMemoryOp)`、`RawBodyMemoryOperationBoundary(RawBodyMemoryOp)` に分けた。raw address identity helper や `MemPtr` / `RegionToken` constructor は structural capability だけを付与し、`load` / `store` / `alloc` などの actual raw helper と intrinsic は operation enum として記録する。`#wasm` / `#llvm` body の memory instruction も backend operation enum として記録し、typecheck と ResourceEffectBoundary diagnostic suppression は使用中の operation と file capability を照合する。

これにより、compiler-owned stdlib source で raw `load` evidence があるだけでは raw `store` を許可できず、structural raw address helper だけでは unsafe memory operation diagnostic を抑制できない。親 issue は `OwnedBuffer<T>` / compiler-issued owner token / initialized prefix / collection drop traversal の最終移行が残るため open のまま継続する。

同じ修正で、pure checked wrapper 名 `alloc` / `dealloc` / `realloc` を raw operation として扱う設計は採用しなかった。`alloc` は `Result<i32,str>` を返す checked API であり、名前だけで raw op にすると `Result` 全体へ free obligation を割り当てて `alloc_ptr` の owner transfer を壊すためである。`ResourceEffectBoundaryDiagnostic::RawAddressEscapeFromInternalAlloc` は raw identity の発生元 `RawMemoryOp` を保持し、source capability とは `Alloc` / `Realloc` 由来 identity として照合する。

`ISS-20260515T031921532Z-OWNER-AGGREGATE-FIELD-EVIDENCE-IGNOR-943C9579` として、owner aggregate field evidence walker が `#intrinsic "get_field_ref"` / `#intrinsic "get_field"` 自体を field accessor evidence として分類していなかった問題も分離して修正した。これにより `core/mem/types.nepl` の compiler-owned field projection は `OwnerAggregateFieldBoundary` capability として証明され、同時に top-level `struct` / `enum` / `trait` 定義を value shadow として扱わないことで same-module constructor evidence も失わない。

## 2026-05-15 Agent 1 RegionToken realloc 境界追記

`ISS-20260515T141747916Z-VEC-GROW-REIMPLEMENTS-REGIONTOKEN-RE-255A043F` として、`Vec` / `ByteBuilder` が `RegionToken` の `ptr` / `size` を直接分解して `realloc_ptr` を呼ぶ設計を分離して修正した。

今回の修正では、`core/mem/pointer/region.nepl` が `realloc_region_bytes_keep<T>` を所有し、stdlib collection / byte builder 側は `RegionToken` owner をそのまま渡す。realloc 失敗時に旧 owner を返す `RegionReallocError<T>` を型に出すことで、caller は cleanup するか owner-preserving error として返すかを明示できる。

この親 issue は引き続き open とする。今回閉じたのは RegionToken realloc の重複実装であり、raw-memory-backed stdlib 全体を `OwnedBuffer<T>` / compiler-issued owner token / initialized prefix / collection drop traversal に移す作業は継続する。

## 2026-05-16 Agent 1 RegionToken raw owner field 追記

`ISS-20260515T155626605Z-REGIONTOKEN-STILL-STORES-MEMPTR-AS-O-C0F9E4D1` として、最後に残っていた `RegionToken.ptr: MemPtr<T>` owner-like field を分離して修正した。

今回の修正では、`RegionToken<T>` の layout を `raw: i32, size: i32` にし、`MemPtr<T>` は参照から得る non-owning projection としてだけ扱う。`region_ptr<T>(&RegionToken<T>)` は checked projection helper であり、free obligation owner は `RegionToken.raw` と Resource IR の owner summary が追跡する。`nodesrc/test_stdlib_memptr_owner_field_policy.js` の transitional allowlist は 0 件になった。

同時に Resource IR owner summary は、直接 raw memory op だけでなく callee summary が消費する raw owner alias も raw owner seed に反映するようにした。これにより `dealloc_region<T>` が `RegionToken.raw -> MemPtr.raw -> dealloc_ptr<T>` と helper 経由で owner を閉じる場合も、caller 側では `RegionToken<T>` 引数の消費として証明される。

この親 issue は引き続き open とする。今回の修正で `MemPtr` owner field の transitional debt は 0 件になったが、`RegionToken<T>` 自体はまだ stdlib raw boundary 内で構築できる過渡 owner token であり、最終的な `OwnedBuffer<T>` / compiler-issued owner token / initialized prefix / collection drop traversal は Stage 6 残件として継続する。

## 2026-05-16 Agent 1 public alloc_ptr owner carrier 追記

`ISS-20260515T163252091Z-PUBLIC-ALLOC-PTR-APIS-STILL-ENCODE-F-EECDD686` として、`alloc_ptr` / `realloc_ptr` / `dealloc_ptr` が public API の形で `MemPtr<T>` に free obligation を残している問題を分離した。

`RegionToken<T>` の struct field から `MemPtr<T>` は消えたが、`alloc_ptr<T> -> Result<MemPtr<T>, str>` は ordinary safe source から allocation owner を `MemPtr<T>` として取得できる入口である。これは Stage 6 の「`MemPtr<T>` は non-owning pointer projection」という表面 contract と、Resource IR が `alloc_ptr` / `dealloc_ptr` に owner summary を付けている内部 contract の食い違いである。

この親 issue は引き続き open とする。次段階では stdlib scratch buffer を token boundary へ移し、public example / root facade / Resource IR summary から `MemPtr<T>` owner carrier を取り除く。即時に削除すると `std/fs` / `std/stdio` / `std/env/cliarg/raw` の scratch cleanup まで広く崩れるため、`OwnedRegion` / `OwnedBytes` / `OwnedBuffer` への段階移行 issue として扱う。

## 2026-05-16 Agent 1 std/fs fd read RegionToken 境界追記

`ISS-20260515T200013147Z-STD-FS-FD-READ-SCRATCH-STILL-USES-ME-7F2B4F1E` で、`std/fs/read/fd.nepl` の fd_read growable buffer / iovec / nread scratch を `RegionToken<u8>` owner に移した。

この対応により `std/fs/read` の public read path は `MemPtr<u8>` を free obligation carrier とせず、raw ABI へは `region_ptr` 由来の non-owning view だけを渡す。`std/fs/raw/fd_io.nepl` の finish/discard helper も `RegionToken<u8>` owner を消費するため、read buffer の shrink / ByteBuf 確定 / cleanup が `RegionToken` 境界に揃った。親 issue の残件は `std/fs/dir/read_fd.nepl`、`std/fs/raw/llvm.nepl`、`std/env/cliarg/raw.nepl` などの raw-backed boundary と、最終的な `OwnedBuffer<T>` / compiler-issued owner token / initialized prefix / collection drop traversal である。

## 2026-05-16 Agent 1 fs dir read RegionToken owner 境界追記

`ISS-20260515T201227745Z-STD-FS-DIR-READ-SCRATCH-STILL-USES-M-92BCD4BA` で、`std/fs/dir/read_fd.nepl` の `fd_readdir` buffer と `used` out-pointer scratch を `RegionToken<u8>` owner 境界へ移した。

この修正は `MemPtr = non-owning pointer` / `RegionToken = free obligation owner` の分離を directory listing の raw ABI path にも適用するもの。`wasi_fd_readdir` に渡す raw address は `region_ptr` 由来の view からだけ得て、entry name の所有と cleanup は既存の `Vec<str>` public boundary で閉じる。

この親 issue は引き続き open とする。`std/fs/raw/llvm.nepl` / `std/env/cliarg/raw.nepl` など raw-backed boundary の direct allocation owner と、`OwnedBuffer<T>` / compiler-issued owner token / initialized prefix / collection drop traversal は Stage 6 残件として継続する。

## 2026-05-16 Agent 1 fs llvm cstr RegionToken owner 境界追記

`ISS-20260515T202108805Z-STD-FS-LLVM-CSTR-SCRATCH-STILL-RETUR-69733E05` で、`std/fs/raw/llvm.nepl` の LLVM `path_open` fallback 用 C string scratch を `RegionToken<u8>` owner 境界へ移した。

この修正により、Linux syscall に渡す NUL-terminated path buffer の free obligation は `RegionToken<u8>` に残り、`MemPtr<u8>` は byte copy と `mem_ptr_addr` のための non-owning view に限定される。LLVM fallback も WASI/raw helper 境界と同じ責務分割になり、`std/fs` module 群から direct low-level allocation owner API を取り除いた。

この親 issue は引き続き open とする。`std/env/cliarg/raw.nepl` の argv scratch と、`OwnedBuffer<T>` / compiler-issued owner token / initialized prefix / collection drop traversal は Stage 6 残件として継続する。

## 2026-05-16 Agent 1 cliarg raw RegionToken owner 境界追記

`ISS-20260515T202737251Z-STD-ENV-CLIARG-RAW-SCRATCH-STILL-USE-D6D56ABD` で、`std/env/cliarg/raw.nepl` の argv raw boundary scratch を `RegionToken<u8>` owner 境界へ移した。

argc metadata、argv pointer array、argv byte buffer、LLVM `/proc/self/cmdline` C string、cmdline temporary buffer は token owner と non-owning `MemPtr<u8>` view に分離した。これにより root facade だけでなく raw boundary implementation 内でも、free obligation を `MemPtr` に保持し続ける経路を閉じた。

この親 issue は引き続き open とする。今回の focused verification で `cliarg_get_checked` の負 index 下限検査不足と、`cstr.nepl` doctest の ordinary raw memory fixture を分離した。Stage 6 全体では `OwnedBuffer<T>` / compiler-issued owner token / initialized prefix / collection drop traversal が残件である。

## 2026-05-16 Agent 1 cliarg raw negative index gate 追記

`ISS-20260515T203122854Z-STD-ENV-CLIARG-GET-CHECKED-ACCEPTS-N-19FA44EB` で、`std/env/cliarg/raw.nepl` の `cliarg_get_checked` が負 index を拒否せず `arg_slot_raw = argv_raw + idx * 4` へ進む問題を修正した。

root facade の `cliarg_get` は既に負 index を拒否していたが、raw boundary helper は explicit import 可能なので、helper 自体も slot address 計算前に下限検査を持つ必要がある。修正後は `idx < 0`、`idx >= argc`、`buf_size <= 0` を同じ gate で拒否し、doctest は `cli_raw::cliarg_get_checked -1` が `None` になることを確認する。

この親 issue は引き続き open とする。`cstr.nepl` doctest fixture と Stage 6 の `OwnedBuffer<T>` / compiler-issued owner token / initialized prefix / collection drop traversal は残件である。

## 2026-05-16 Agent 1 cliarg cstr doctest raw boundary 追記

`ISS-20260515T203123090Z-STD-ENV-CLIARG-CSTR-DOCTESTS-USE-MEM-B6DEAC7B` で、`std/env/cliarg/cstr.nepl` の doctest が ordinary source から `mem_ptr_add` / `store_u8` を呼んでいた stale fixture を修正した。

修正後の doctest は NUL を含む string literal `"nep\0"` の `string_data_ptr` を `cstr_len` / `cstr_to_str` に渡す。これにより C string helper の典型例を維持しながら、ordinary doctest に raw memory write authority を要求しない。Resource IR の `resource.raw.memory_outside_boundary` は緩めない。

この親 issue は引き続き open とする。Stage 6 の `OwnedBuffer<T>` / compiler-issued owner token / initialized prefix / collection drop traversal は残件である。

## 2026-05-16 Agent 1 memory model doc 同期追記

`ISS-20260516T041311764Z-STAGE-6-MEMORY-MODEL-DOC-STILL-DESCR-AEC8348B` で、`doc/neplg2/stdlib_collection_mem_string_static_safety_design.md` に残っていた削除済み `alloc_ptr` owner path / `core/mem/pointer/alloc` direct import 前提を修正した。

実装上は `stdlib/core/mem/pointer/alloc.nepl` が削除済みであり、safe facade だけでなく direct import でも `MemPtr<T>` allocation owner API へ戻れない。文書もこれに合わせ、scratch storage は `RegionToken` owner と `region_ptr` 由来の non-owning ABI view に分ける方針へ同期した。

この親 issue は引き続き open とする。今回閉じたのは設計文書の stale guidance であり、`OwnedBuffer<T>` / compiler-issued owner token / initialized prefix / collection drop traversal は Stage 6 残件である。

## 2026-05-17 Agent 1 RegionToken raw identity helper 境界追記

`ISS-20260517T031453210Z-REGIONTOKEN-RAW-IDENTITY-REFERENCE-R-BB2D917B` で、`region_token_raw_ref` が safe `core/mem` facade から見えていた問題を分離して修正した。

`RegionToken.raw` の direct field projection は `type.owner_token.field_access_restricted` で拒否していたが、同じ raw free-obligation identity を `region_token_raw_ref` 経由で通常 source が読める状態では、Stage 6 の public/internal 境界として不十分だった。修正後は `region_token_raw_ref` を `mem/internal` に閉じ、safe `core/mem` facade には `region_size` / `region_in_bounds` のような metadata observer だけを残す。

この親 issue は引き続き open とする。今回閉じたのは RegionToken raw identity の public observer 漏れであり、forgeable `RegionToken` を compiler-issued owner token / `OwnedBuffer<T>` / initialized prefix / collection drop traversal へ移す作業は継続する。

## 2026-05-17 Agent 1 core/mem raw doctest 境界追記

`ISS-20260517T032727149Z-CORE-MEM-RAW-DOCTESTS-CALL-RAW-APIS--057347FB` で、`core/mem/allocator` と `core/mem/raw` の doctest が ordinary source から raw API を成功例として呼んでいた問題を分離して修正した。

Stage 6 では raw memory operation は compiler-owned raw-memory boundary の source proof がある場所だけで許可される。doctest entry は通常利用者 source として扱われるため、`alloc_raw`、`mem_size`、`memset_u8`、`fill_i32` の直接呼び出しを成功例にするのは設計と矛盾していた。修正後はこれらを `compile_fail` の境界回帰テストにし、raw boundary を緩めずに `stdlib/core/mem` focused doctest を通す。

この親 issue は引き続き open とする。今回閉じたのは raw module docs の stale executable fixture であり、`OwnedBuffer<T>` / compiler-issued owner token / initialized prefix / collection drop traversal は Stage 6 残件として継続する。

## 2026-05-17 Agent 1 WASIX TTY state owner 境界追記

`ISS-20260517T033712430Z-WASIX-TTY-RAW-MODE-EXPOSES-RAW-I32-S-45184629` で、`platforms/wasix/tui/tty.nepl` の raw mode state owner が public raw `i32` として露出していた問題を分離して修正した。

以前の `enter_raw_mode` / `restore_mode` は、WASIX TTY state buffer の free obligation を raw `i32` pointer と sentinel `0` で表していた。修正後は `TtyState` が `RegionToken<u8>` owner を保持し、public raw-mode API は `Result<TtyState,i32>` を返して `restore_mode` が `TtyState` を消費する。`tty_get` / `tty_set` に必要な raw address 変換は TTY module 内の helper に閉じ、`alloc_raw` / `dealloc_raw` を public state owner contract から取り除いた。

この親 issue は引き続き open とする。今回閉じたのは WASIX TTY の raw state owner 漏れであり、`OwnedBuffer<T>` / compiler-issued owner token / initialized prefix / collection drop traversal は Stage 6 残件として継続する。

## 2026-05-17 Agent 1 ByteBuf/ByteBuilder raw MemPtr owner 偽造入口追記

`ISS-20260517T034837136Z-BYTEBUF-PUBLIC-API-CAN-FORGE-OWNERSH-16F30AE5` で、`ByteBuf` / `ByteBuilder` が caller supplied `MemPtr<u8>` を `RegionToken<u8>` owner に包む helper を持っていた問題を分離して修正した。

`MemPtr<u8>` は non-owning view であり、free obligation owner は `RegionToken<u8>` / `ByteBuf` / `ByteBuilder` 側に集約する必要がある。`io_bytebuf_from_owned_ptr` と `byte_builder_from_owned_ptr` はこの境界を public source から迂回でき、doctest でも fake huge ByteBuf を作る fixture に使われていた。修正後は raw `MemPtr` ingestion helper を削除し、ByteBuf/ByteBuilder owner は checked allocation / `RegionToken` finalization からだけ作る。

この親 issue は引き続き open とする。今回閉じたのは byte buffer API の raw MemPtr owner forging entry であり、`OwnedBuffer<T>` / compiler-issued owner token / initialized prefix / collection drop traversal は Stage 6 残件として継続する。

## 2026-05-18 Agent 1 internal raw address helper 境界追記

`ISS-20260518T052853094Z-USER-SOURCE-CAN-FORGE-RAW-MEMPTR-THR-109B2F18` で、ordinary source が `core/mem/internal` を import して `mem_ptr_wrap` / `mem_ptr_addr` を直接呼ぶ raw pointer identity 偽造経路を分離して修正した。

Resource IR は raw address alias と raw address view を `InternalHelper` / `Transparent` / `NonOwningProjection` の enum 種別として保持する。`mem_ptr_wrap` / `region_new` は raw address alias boundary、`mem_ptr_addr` / `region_token_raw_ref` / `str_addr` は raw address view boundary として effect gate に渡し、`region_ptr` / `region_ptr_at` は checked public projection として残す。source capability proof も alias boundary と view boundary を別 fact として証明するため、raw helper 名の file/module allowlist ではなく、compiler-owned source の exact use-site evidence からだけ許可できる。

この親 issue は引き続き open とする。今回閉じたのは raw memory backed API 移行中に残っていた `core/mem/internal` 直接 import による raw identity 偽造入口であり、`OwnedBuffer<T>` / compiler-issued owner token / initialized prefix / collection drop traversal は Stage 6 残件として継続する。

## 2026-05-18 Agent 1 Vec data_mem_ptr checked-store regression 追記

`ISS-20260518T062457553Z-VEC-DATA-MEM-PTR-BOUNDARY-LACKS-DIRE-DF309C77` として、safe `Vec` root facade から `data_mem_ptr<T>` を取得し、それを通常 source の checked store authority として使えないことを直接固定した。

`data_mem_ptr<T>(&Vec<T>) -> MemPtr<T>` は現行の Copy-only collection 実装で内部 storage view として使われているが、`MemPtr<T>` は free obligation owner ではなく non-owning view である。今回の regression は、ordinary source が `Vec<i32>` の backing view を取り出して `store_i32` へ渡すと `resource.raw.memory_outside_boundary` で拒否されることを `tests/stdlib/memory_safety.n.md` に追加した。これにより、将来の Resource IR return summary / checked MemPtr provenance 修正が、collection mutation API と initialized/drop state discipline を迂回する方向へ退行した場合に検出できる。

この親 issue は引き続き open とする。今回閉じたのは regression gap であり、`OwnedBuffer<T>` / compiler-issued owner token / initialized prefix / collection drop traversal の最終移行は継続する。

## 2026-05-18 Agent 1 alloc/string raw address public helper 境界追記

`ISS-20260518T071033883Z-ALLOC-STRING-STORAGE-EXPOSES-UNCHECK-9EA051F0` として、`alloc/string` module 群が raw `str` address observer / unchecked finalizer を public API として露出していた問題を分離して修正した。

`string_finish_base` は `MemPtr<u8>` から `str` を確定できる不要 helper であり、現行の live path はすべて `RegionToken<u8>` を消費する `string_finish` に集約済みだったため削除した。`string_addr` / `string_from_addr_unchecked` は `storage.nepl` 内の private helper に閉じ、`access.nepl` / `scanner.nepl` が必要とする `str_addr` observer も各 module の private helper にした。

この親 issue は引き続き open とする。今回閉じたのは raw address helper の public surface 漏れであり、`RegionToken` を compiler-issued owner token / `OwnedStringRegion` / `OwnedBuffer<T>` / initialized prefix へ移す作業は Stage 6 残件として継続する。

## 2026-05-18 Agent 1 alloc/string scanner unchecked byte reader 境界追記

`ISS-20260518T072300172Z-ALLOC-STRING-SCANNER-EXPOSES-UNCHECK-BE608F25` として、`alloc/string/scanner.nepl` が `scanner_string_byte_at_unchecked` を public API として公開していた問題を分離して修正した。

scanner の public helper は `str_find_byte_range` / `str_line_end` / `str_next_line_pos` などで範囲を正規化してから内部 unchecked reader を呼ぶ設計であり、外部 caller が任意 index で raw string layout read を直接呼ぶ必要はない。修正後は unchecked reader を private にし、direct import でも到達できないことを memory safety doctest と source policy で固定した。

この親 issue は引き続き open とする。今回閉じたのは scanner module の不要な unchecked public reader であり、root `alloc/string/access` 側の広範な `string_byte_at_unchecked` 利用整理や、compiler-issued owner token / initialized prefix は Stage 6 残件として継続する。

同じ監査で、root `alloc/string` facade から見える `string_byte_at_unchecked` は stdlib / selfhost の多数の範囲証明済みループで使われており、単純な private 化ではなく bounded-index proof / checked helper migration が必要であることを確認した。この大きな設計残件は `ISS-20260518T072713737Z-ALLOC-STRING-PUBLIC-UNCHECKED-BYTE-A-896205D1` として分離し、この親 issue の Stage 6 残件に含める。

## 2026-05-18 Agent 1 alloc/string byte index witness 化追記

`ISS-20260518T072713737Z-ALLOC-STRING-PUBLIC-UNCHECKED-BYTE-A-896205D1` を解決した。一次修正で隔離していた `alloc/string/unchecked_access` は proof を型に持たない public unchecked boundary だったため削除し、`alloc/string/byte_index` の private `StringByteIndex` witness に置き換えた。

`checked_string_byte_index` は `0 <= idx < len(s)` を確認した場合だけ witness を返し、raw layout reader `string_byte_at_checked` はこの witness を要求する。既存 stdlib / selfhost hot path は `string_byte_at_checked_or_unreachable` へ移行したが、この helper も checked factory を必ず通るため、範囲外 index が raw read に届かない。root facade / access / byte_index の public unchecked reader、public constructor、任意 `i32` からの checked reader 呼び出しは source policy と memory safety doctest で固定した。

この親 issue は引き続き open とする。今回閉じたのは string byte reader の public unchecked surface であり、`RegionToken` の compiler-issued owner token 化、`OwnedStringRegion` / `OwnedBuffer<T>`、initialized prefix / collection drop traversal は Stage 6 残件として継続する。

## 2026-05-18 Agent 1 UTF-8 raw byte reader 境界追記

`ISS-20260518T112925248Z-UTF-8-RAW-BYTE-READERS-EXPOSE-UNCHEC-35257411` を解決した。`alloc/string/utf8` と `std/text/validate` は root facade からは raw helper を再公開していなかったが、explicit submodule import では `string_utf8_byte_at` / `text_utf8_byte_at` と sequence validator が public に見えていた。これは `validate_mem` / `decode_next` が持つ byte_len discipline を通らない単一 byte raw read surface であり、Stage 6 の raw-memory-backed API 境界として不十分だった。

修正後、`alloc/string/utf8` の単一 byte reader と sequence validator は private に閉じ、内部では `string_utf8_byte_at_checked(data, byte_len, idx)` を必ず通す。`std/text/validate` は `decode` との共有に必要な `text_utf8_byte_at_checked(data, byte_len, idx)` だけを public に残し、旧 helper 名と sequence validator は公開しない。source policy と memory-safety doctest は、byte_len を持たない raw byte reader と public sequence validator の再導入を拒否する。

この親 issue は引き続き open とする。今回閉じたのは UTF-8 validation submodule の unchecked public byte-reader surface であり、`OwnedBuffer<T>` / compiler-issued owner token / initialized prefix / collection drop traversal は Stage 6 残件として継続する。

## 2026-05-18 Agent 1 cliarg C string bounded 境界追記

`ISS-20260518T114636396Z-CLIARG-C-STRING-CONVERSION-LACKS-ARG-21786F27` を解決した。`std/env/cliarg/cstr` は `MemPtr<u8>` だけを受ける `cstr_len_result` / `cstr_len` / `cstr_to_str` を public にしており、NUL まで走査する際に argv byte buffer の owner extent を受け取っていなかった。`cliarg_get_checked` 側は `argv_buf_raw` と `buf_size` を知っていたが、`arg_ptr` だけを渡すことで範囲証明を落としていた。

修正後は unbounded public API を削除し、`cstr_len_bounded_result(data, max_len)` と `cstr_to_str_bounded_result(data, max_len)` に一本化した。`cliarg_get_checked` は `arg_offset = arg_ptr - argv_buf_raw` を検査し、`buf_size - arg_offset` を C string conversion へ渡す。さらに `cstr_to_str_bounded_result` は `string_from_utf8_mem_result` を使うため、外部 argv byte 列を UTF-8 検証なしに `str` へ昇格する経路も閉じた。

## 2026-05-19 cliarg raw helper public surface 整理追記

`ISS-20260519T173256333Z-STD-ENV-CLIARG-RAW-MEMPTR-HELPERS-RE-6C50B75B` で、`std/env/cliarg/raw.nepl` の implementation-only raw helper 公開面を閉じた。

`cli_args_sizes_result` / scratch zeroing / checked byte load / LLVM cmdline shim は root facade から直接使わない実装詳細なので private にし、公開 helper は `cliarg_count_result` / `cliarg_get_checked` に限定した。これにより `MemPtr<u8>` と size/offset を受け取る raw argv helper へ ordinary source が direct import で依存する経路を減らした。親 issue の残件は引き続き `OwnedBuffer<T>` / compiler-issued owner token / initialized prefix / collection drop traversal と、collection free の drop traversal である。

この親 issue は引き続き open とする。今回閉じたのは argv C string conversion の buffer extent proof 漏れであり、`OwnedBuffer<T>` / compiler-issued owner token / initialized prefix / collection drop traversal は Stage 6 残件として継続する。

## 2026-05-18 Agent 1 SHA256 owner aggregate field 境界追記

`ISS-20260518T081055566Z-SHA256-DOCTEST-BLOCKED-BY-OWNER-AGGR-B2EE3B20` を解決した。SHA256 incremental state は `Sha256.buffer` として `Vec<i32>` owner を保持しているため、通常 source から aggregate field を直接読む設計にはできない。

修正では `sha256/api.nepl` の実装内 field access を `core/field` 経由に揃え、compiler-owned stdlib source の owner aggregate field access 証拠が構造化された形で残るようにした。利用者側には `sha256_update_error_kind(&Sha256UpdateError)` と `sha256_update_error_ctx(Sha256UpdateError)` を公開し、error kind の borrow read と state owner の消費回収を分離した。

この親 issue は引き続き open とする。今回閉じたのは SHA256 の owner aggregate field boundary であり、`OwnedBuffer<T>` / compiler-issued owner token / initialized prefix / collection drop traversal の最終移行は Stage 6 残件として継続する。

## 2026-05-18 Agent 1 external IO iovec owner extent 追記

`ISS-20260518T095629679Z-RESOURCE-IR-EXTERNAL-IO-IOVEC-PAYLOA-EBED3E34` を解決した。`stdio` / `fs` の fd read/write scratch は `RegionToken` owner と non-owning `MemPtr` view へ移行済みだったが、Resource IR owner checker は external IO の iovec payload span を `OwnerStorageExtent` と照合していなかった。

修正後は `ExternalIoOp` の typed match から iovec payload を持つ `fd_read` / `fd_write` / `fd_pread` / `fd_pwrite` を分類し、shared iovec layout helper で descriptor cell から payload pointer と length を導出する。payload pointer は raw alias を通じて backing owner へ解決し、length が allocation / RegionToken extent と一致しない場合は `ResourceOwnerOperation::ExternalIoPayloadExtent` として拒否する。

同時に、非所有 raw address view を iovec descriptor へ保存する場合に free obligation を移動せず、descriptor cell へ non-owning alias を残すようにした。これは stdlib 名 allowlist ではなく、Resource IR の raw view mark、alias table、owner extent proof を接続する修正であり、Stage 6 の `MemPtr = non-owning pointer` / `RegionToken = free obligation owner` 分離を compiler 側でも強める。

## 2026-05-18 Agent 1 ByteBuilder typed source copy 境界追記

`ISS-20260518T115751573Z-BYTEBUILDER-PUBLIC-RAW-BYTE-APPEND-D-D94BB3A0` を解決した。`byte_builder_push_bytes_ref` は `alloc/io/bytebuilder` facade から public に見えており、通常 source が任意の `MemPtr<u8>` と任意 length を組み合わせて `mem_copy` へ到達できた。`StringBuilder` の既存 caller は `str` の length / pointer や UTF-8 境界確認済み slice から pair を作っていたが、その証明は public API signature に残っていなかった。

修正後は raw copy helper を private に閉じ、public API は `byte_builder_push_str` と `byte_builder_push_str_slice` に限定した。full append は `len s` と `string_data_ptr s` を同じ `str` から導出し、slice append は `0 <= start <= end <= len(s)` を確認してから pointer と length を導出する。`StringBuilder` 側もこの typed helper に委譲するため、source object と readable extent の対応を caller convention ではなく stdlib API 境界で保持できる。

この親 issue は引き続き open とする。今回閉じたのは ByteBuilder の public raw pointer/length append surface であり、`OwnedBuffer<T>` / compiler-issued owner token / initialized prefix / collection drop traversal は Stage 6 残件として継続する。

## 2026-05-18 Agent 1 ByteBuilder structural empty storage 追記

`ISS-20260518T130359928Z-BYTEBUILDER-EMPTY-STORAGE-STILL-USES-D46A4A9A` を解決した。`ByteBuilder` は public `byte_builder_empty_region` を削除した後も、内部では `region_new(mem_ptr_wrap 0, 0)` により empty storage を zero-size `RegionToken<u8>` sentinel として表していた。

修正後は `ByteBuilderStorage::Empty | Owned(RegionToken<u8>)` を導入し、`ByteBuilder` は `storage: ByteBuilderStorage`、`len`、`cap` を持つ。`byte_builder_empty` は owner payload を持たない `Empty` を返し、`byte_builder_from_owned_region` だけが `Owned(region)` を作る。`byte_builder_reserve` / `byte_builder_push_u8` / `byte_builder_push_bytes_ref` / `byte_builder_finish` / `byte_builder_free` は storage enum を `match` し、free obligation owner を持つ branch だけが `RegionToken<u8>` を消費または借用する。

source policy は `ByteBuilderStorage` enum、`storage` field、`ByteBuilderStorage::Empty` constructor、`Owned(region)` wrapping、append/free/reserve の storage match を要求し、`byte_builder_empty_region`、zero-size `region_new` sentinel、旧 `"region"` field access の再導入を拒否する。これで ByteBuf / ByteBuilder の空 storage はどちらも enum state として表現され、次の残件は forgeable `RegionToken` を compiler-issued owner token / `OwnedBytes` / `OwnedBuffer<T>` へ移すことである。

## 2026-05-18 Agent 1 ByteBuf removed-helper doctest raw fixture 整理追記

`ISS-20260518T200613054Z-BYTEBUF-DOCTEST-IMPORTS-RAW-INTERNAL-387EC456` を解決した。`io_bytebuf_from_owned_ptr` が削除済みであることを確認する compile-fail doctest が、`core/mem/internal` と `mem_ptr_wrap` を使って raw `MemPtr` fixture を作っていた。

修正後の doctest は `alloc/io` だけを import し、`io_bytebuf_from_owned_ptr 0 1` が未定義になることを確認する。source policy もこの removed-helper fixture に raw memory module import や raw `MemPtr` construction が戻らないことを監視する。これは ByteBuf owner boundary の本体修正ではなく、Stage 6 の regression fixture が raw boundary privilege を正常な前提として示さないための整理である。

## 2026-05-18 Agent 1 stdio read raw fd_read helper 境界追記

`ISS-20260518T213314500Z-STD-STDIO-READ-BUFFER-EXPOSES-RAW-ME-5BAB55A1` を解決した。`std/stdio/read/buffer` の raw `MemPtr` fd_read helper は direct import で public に見えていたため、caller が任意の iov / nread / data pointer と length を組み合わせて fd_read 境界へ渡せた。

修正後は `stdio_fd_read_into_result` を private helper に戻し、lower-level な fd_read slice wrapper も public surface に出さない。public surface は `stdio_read_all_buffer_result` / `stdio_read_line_buffer_result` に限定し、iov / nread scratch、destination buffer、non-owning `MemPtr` view、fd_read loop、cleanup を `std/stdio/read/buffer` の local `RegionToken<u8>` owner 境界へ集約した。`read_all` / `read_line` は buffer module 外で raw fd_read span を再構築せず、高水準 `ByteBuf` API へ委譲するため、fd_read payload extent は caller convention ではなく source object と local owner proof に接続される。

## 2026-05-18 Agent 1 fs directory entry raw byte conversion 境界追記

`ISS-20260518T203331363Z-STD-FS-PATH-ENTRY-EXPOSES-RAW-DIRECT-E3B0CD92` を解決した。`std/fs/path/entry` は safe path facade から re-export される module であるにもかかわらず、`fs_string_from_bytes(i32,i32)` を public に持ち、任意 raw address を `mem_ptr_wrap` で `MemPtr<u8>` へ包んで directory entry name の `str` 化を行っていた。

修正後は directory entry byte conversion を `std/fs/dir/read_fd` の private `fs_dirent_name_to_string(MemPtr<u8>, i32)` に閉じた。`fs_read_dir_fd` は `fd_readdir` buffer を所有する `RegionToken<u8>` から得た `buf_ptr` に `mem_ptr_add` して name pointer を作るため、safe path facade は raw `i32` address conversion を公開しない。source policy と compile-fail doctest は、`std/fs/path` から `fs_string_from_bytes` が戻らないこと、`dir/read_fd` が raw address を再包装しないことを監視する。

## 2026-05-18 Agent 1 fs write raw span writer 境界追記

`ISS-20260518T205653919Z-STD-FS-WRITE-FACADE-EXPOSES-RAW-MEMP-A8C64961` を解決した。`std/fs/write` は `std/fs/write/fd` を re-export しており、`fs_write_fd_mem_result(i32, MemPtr<u8>, i32)` が public のままだったため、通常 source が `ByteBuf` owner 境界を通らず任意の pointer/length pair を fd write loop へ渡せた。

修正後は `fs_write_fd_mem_result` を `std/fs/write/fd` 内の private helper に閉じ、public fd write API は `fs_write_fd_bytes(fd, ByteBuf)` に限定した。`ByteBuf` から `data` と `data_len` を同時に導出した場合だけ raw ABI write helper へ進むため、`MemPtr = non-owning pointer` と owner-backed readable extent の対応を caller convention に押し出さない。source policy と compile-fail doctest は、safe facade と direct fd module import の両方で raw span writer が公開されないことを監視する。

## 2026-05-18 Agent 1 stdio write raw span writer 境界追記

`ISS-20260518T210549005Z-STD-STDIO-WRITE-FACADE-EXPOSES-RAW-M-11591E6E` を解決した。`std/stdio/write` は `std/stdio/write/fd` を re-export しており、`stdio_write_fd_mem_result(i32, MemPtr<u8>, i32)` と stdout/stderr 用 raw span wrapper が public のままだったため、通常 source が `str` / `ByteBuf` / `ByteBuilder` の owner 境界を通らず任意の pointer/length pair を fd write loop へ渡せた。

修正後は raw fd write loop を `std/stdio/write/fd` の private helper に閉じ、public surface を `stdio_write_fd_str_result(fd, str)`、`stdio_write_fd_bytebuf_result(fd, ByteBuf)`、`stdio_write_fd_bytebuilder_prefix_result(fd, &ByteBuilder, byte_len)`、`stdio_write_fd_byte_result(fd, i32)` に限定した。`streamio/writer` の flush も `ByteBuilder` の raw pointer view を直接取り出さず、typed ByteBuilder prefix wrapper に委譲する。これにより readable span は source object と checked length から導出され、caller convention ではなく API shape と source policy で監視できる。

## 2026-05-19 Agent 1 StreamScanner token UTF-8 境界追記

`ISS-20260519T064344008Z-STREAMSCANNER-TOKEN-SLICES-CONSTRUCT-54D1E67F` を解決した。`StreamScanner` は `ReadStream::Bytes` や file input から任意 byte 列を受け取り得るにもかかわらず、token slice を `str` にする境界で `string_from_mem_unchecked_result` を使っていた。byte range check は owner extent の検査であり、UTF-8 invariant の証明にはならない。

修正後、`stream_scanner_slice_to_str_result` は range check 後に `string_from_utf8_mem_result` へ委譲する。これにより `ByteBuf` owner から導出した readable span と `str` の UTF-8 invariant が同じ scanner state boundary で接続され、invalid UTF-8 token bytes は `str` として公開されない。source policy は checked constructor 利用と unchecked constructor 退行禁止を監視し、doctest は `ReadStream::Bytes` の invalid token が空 token に丸められることを stdout report として固定する。

この親 issue は引き続き open とする。今回閉じたのは StreamScanner token slice の unchecked string construction であり、`OwnedBuffer<T>` / compiler-issued owner token / initialized prefix / collection drop traversal は Stage 6 残件として継続する。

## 2026-05-19 Agent 1 Vec raw element direct import 境界追記

`ISS-20260519T124203801Z-VEC-RAW-ELEMENT-HELPERS-ARE-DIRECTLY-85EBD72F` を解決した。`alloc/collections/vec` root facade から `vec/raw` re-export は削除済みだったが、通常 source は `#import "alloc/collections/vec/raw" as raw` を明示でき、`data_mem_ptr(&v)` と unchecked `vec_write_at` / `vec_read_at` を直接組み合わせられた。この経路は `Vec.len` / storage variant / initialized slot の検査を通らず、raw operation span が compiler-owned stdlib callee 側にあるため caller 側の任意 `MemPtr<T>` と範囲の対応を型で証明していなかった。

修正後は `stdlib/alloc/collections/vec/raw.nepl` と `vec/raw/element.nepl` を削除し、Copy element の raw load/store は `get` / `pop` / `push` / `replace` / `map` / `filter` / `partition` / `take_while` / `drop_while` の検査済み分岐と同じ source file 内に置いた。これにより public/direct-import 可能な unchecked element helper は残らず、source policy は `vec/raw` facade の復活、`../raw` import 依存、shared `vec_read_at` / `vec_write_at` helper の復活を拒否する。

この親 issue は引き続き open とする。今回閉じたのは Vec Copy element raw helper の public bypass であり、`OwnedBuffer<T>` / compiler-issued owner token / initialized prefix / moved slot / non-Copy drop traversal は Stage 6 残件として継続する。

## 2026-05-20 Agent 1 Vec OwnedBuffer initialized prefix 追記

`ISS-20260519T190908506Z-OWNEDBUFFER-CONFLATES-LOGICAL-LENGTH-60540914` を解決した。`OwnedBuffer<T>` が `len/cap/storage` だけを持つ状態では、public API の logical length と raw storage 上で初期化済みとして扱う prefix が同じ field に閉じ込められ、後続の moved slot / drop traversal を型として接続できなかった。

修正後は `OwnedBuffer<T>` が `len/initialized_len/cap/storage` を持つ。現行 Copy-only API では両者を同値に保つが、`initialized_len` は drop / move state 用の独立した metadata として保存される。source policy は旧 3-field constructor 形状への退行を拒否する。

この親 issue は引き続き open とする。今回閉じたのは initialized prefix metadata の分離であり、`RegionToken<T>` を compiler-issued owner token / `OwnedBuffer<T>` final owner へ移す作業と、non-Copy payload の drop traversal は Stage 6 残件として継続する。

## 2026-05-19 Agent 1 Resource IR raw memory span summary 追記

`ISS-20260519T142436433Z-RESOURCE-IR-RAW-MEMORY-SPAN-SUMMARIE-FB862D7E` を解決した。raw-memory-backed stdlib API の direct import surface を狭めても、compiler-owned stdlib callee 内の `load_u8` / `mem_copy` / `fill` が caller 側で owner extent requirement に戻らなければ、`MemPtr = non-owning pointer` の設計は caller convention に依存したままになる。

修正後は Resource IR summary の `host_memory_span_requirements` を operation 付きの `memory_span_requirements` へ広げ、external IO payload span と raw memory payload span を同じ owner extent proof machinery で扱う。`string_from_mem_unchecked_result` / `string_from_utf8_mem_result` へ 1 byte region と length 100 を渡す direct import は `resource.owner.unavailable` で拒否されるため、raw string construction の span proof は stdlib module 名 allowlist ではなく callee summary と backing owner extent の照合に移った。

一方で、loop 条件付き scanner の `load_u8(mem_ptr_add p i)` から `p[0..max_len]` を要約する汎用 path-conditioned span proof はまだ不足している。これは `ISS-20260519T144811685Z-RESOURCE-IR-RAW-SPAN-SUMMARIES-MISS--FA49E19D` として分離し、この親 issue の Stage 6 core 残件として継続する。個別 `cstr` 用証明器ではなく、bounded scanner 全体へ適用する Resource IR summary proof として実装する。
