---
id: ISS-20260427T204839136Z-STDLIB-RAW-MEMORY-BACKED-APIS-REQUIR-E503CD84
title: "stdlib raw-memory-backed APIs require staged effect migration"
area: stdlib
status: open
resolved: false
priority: P1
type: architecture
created: 2026-04-27
updated: 2026-05-14
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

`data_ptr` / `data_mem_ptr` / `vec_storage_mem_ptr` は `.T: Copy` に限定済みである。これは raw-memory-backed public API migration の完了ではなく、`OwnedBuffer<T>` と borrow projection が入るまで unsafe な storage identity escape を Copy payload に限定する局所前進である。

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

`MemPtr` は non-owning pointer / projection として固定する方針であり、owned byte storage を `Option<MemPtr<u8>>` field に置く設計は Stage 6 の過渡例外を stdlib public state に残していた。修正後は `ByteBuf.region` / `ByteBuilder.region` が `RegionToken<u8>` owner を保持し、payload pointer は `io_bytebuf_data_ptr_ref` / `byte_builder_data_ptr_ref` で参照から non-owning view として得る。

この移行中に ResourceIR function summary が `region_ptr` 由来の non-owning projection に `mem_ptr_add` を重ねた値を owner alias として扱い、append path を maybe leak と誤診断する問題も露出した。stdlib allowlist で回避せず、summary traversal に raw view state を持たせ、non-owning projection 由来の offset view が owner alias を伝播しないよう compiler 側を修正した。

これにより `MemPtr` owner-like field policy の transitional baseline は `RegionToken.ptr`、`Vec.data` の 2 件になった。残る Stage 6 の主対象は `Vec` / `OwnedBuffer<T>` と、forgeable `RegionToken` を compiler-issued owner token へ移す設計である。

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
