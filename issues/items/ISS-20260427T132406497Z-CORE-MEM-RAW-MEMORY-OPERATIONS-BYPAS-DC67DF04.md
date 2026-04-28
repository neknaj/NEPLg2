---
id: ISS-20260427T132406497Z-CORE-MEM-RAW-MEMORY-OPERATIONS-BYPAS-DC67DF04
title: "core/mem raw memory operations bypass effect and ownership checks"
area: core
status: open
resolved: false
priority: P1
type: bug
created: 2026-04-27
updated: 2026-04-28
target: "nepl-core/src/effects.rs, nepl-core/src/typecheck.rs, nepl-core/src/passes/move_check.rs, nepl-core/src/passes/drop_insertion.rs, stdlib/core/mem.nepl, tests/compiler/move_effect.n.md"
---

# ISS-20260427T132406497Z-CORE-MEM-RAW-MEMORY-OPERATIONS-BYPAS-DC67DF04: core/mem raw memory operations bypass effect and ownership checks

## 概要

`stdlib/core/mem.nepl` は `alloc_raw` / `dealloc_raw` / `realloc_raw` / `load` / `store` を pure function signature として公開している。一方、`nepl-core` の effect 判定は既知 WASI call だけを impure とするため、raw memory 操作が pure 文脈から観測可能なまま呼べる。

## 対象

- `nepl-core/src/effects.rs, nepl-core/src/typecheck.rs, nepl-core/src/passes/move_check.rs, nepl-core/src/passes/drop_insertion.rs, stdlib/core/mem.nepl, tests/compiler/move_effect.n.md`

## 根拠

- `nepl-core/src/ast.rs` の `Effect` は `Pure` / `Impure` の 2 値だけで、`InternalAlloc` や `UnsafeMemory` の内部効果を表現できない。
- `nepl-core/src/effects.rs` の `intrinsic_effect` は既知 WASI marker だけを `Impure` とし、それ以外の intrinsic を `Pure` とする。
- `stdlib/core/mem.nepl` の `alloc_raw` / `dealloc_raw` / `realloc_raw` / `load<T>` / `store<T>` は `*` なしの pure signature で公開されている。
- `nepl-core/src/passes/move_check.rs` と `nepl-core/src/passes/drop_insertion.rs` は intrinsic `load` / `store` を field move などの局所 pattern として扱うが、任意 raw address が所有 place かどうかは追跡しない。
- `doc/compare/memory_model.md` は Phase 0 で `alloc/dealloc/realloc/load/store` を `Effect::Pure` から `Effect::InternalAlloc` へ移す計画を明記しているが、実装側 issue としては未分離だった。

## 問題

`move_check` と `drop_insertion` は intrinsic `load` / `store` を field move などの局所 pattern として special-case しているが、任意の `MemPtr` / raw address がどの owning place に属するかは追跡しない。そのため、raw memory から non-Copy 値を浅く読み出す経路や、pure 関数内で raw address identity を観測しながら allocate/free する経路を、effect / ownership 検査が正しく表現できない。

## 影響

pure source code が observable raw address を allocate / free / load / store でき、non-Copy 値を owned place 外の raw memory から浅く複製できる。self-host compiler の AST / diagnostic / buffer が owning value を増やすほど、effect、borrow、type safety の前提が崩れる。

## 2026-04-27 部分対応

`move_check` に raw memory place の所有権状態を追加し、`load<T>` / `store<T>` および lowered intrinsic `load` / `store` が non-Copy 値を扱う場合は、raw address を owning place として検査するようにした。

- non-Copy `load<T>` は raw place からの move として扱い、同じ place からの二重 load を `D3100` で拒否する。
- non-Copy `store<T>` は raw place の初期化として扱い、未 move の non-Copy 値を含む place への上書きを `D3100` で拒否する。
- `let q p` や `let q add p 4` のような i32 raw address alias を scope / branch snapshot に追従して正規化し、alias 経由の二重 load を拒否する。
- branch 間で raw place 状態が分岐する場合は `PossiblyMoved` として合流し、後続の non-Copy load / store を安全側で拒否する。

この対応は ownership 検査の穴を塞ぐもので、effect model の不足はまだ残る。`alloc_raw` / `dealloc_raw` / `realloc_raw` / `load` / `store` の pure API、`InternalAlloc` / `UnsafeMemory` 相当の effect 導入、stdlib API 移行が必要になる場合の stdlib 側修正は、この issue の残件または別 issue として扱う。

## 2026-04-28 compiler / mem 責務分割レビュー追記

今回の責務分割レビューでは、この issue はまだ閉じられないと判断した。`move_check` の raw place state は non-Copy raw load/store の二重 move をかなり塞いだが、根本の境界はまだ `core/mem.nepl` の public raw API と compiler の effect / provenance model に残っている。

- `stdlib/core/mem.nepl:104` / `107` の `mem_ptr_wrap` / `mem_ptr_addr` により safe source code が raw `i32` address と `MemPtr<T>` を相互変換できる。
- `stdlib/core/mem.nepl:278` / `386` / `450` の allocator primitive と、`1101` / `1117` の generic raw `load<T>` / `store<T>` は pure signature のまま公開されている。
- `nepl-core/src/typecheck.rs:2491` の raw body effect validation は direct callee だけを確認し、memory instruction 自体を分類しない。
- `nepl-core/src/runtime_helpers.rs:8` 以降は compiler 内部 allocator helper discovery を public `alloc_raw` / `dealloc_raw` / `realloc_raw` 名に依存している。

この issue は raw memory operation 全体の親 issue とし、今回のレビューで不足していた追跡単位を次の issue に分割した。

- `ISS-20260427T152947135Z-RAW-BODY-MEMORY-INSTRUCTIONS-BYPASS--162A8C00`: raw body memory instruction が pure effect validation を通らない。
- `ISS-20260427T152951013Z-RUNTIME-ALLOCATOR-HELPER-LOOKUP-DEPE-D070168E`: compiler runtime helper lookup が public `core/mem` 名に依存している。
- `ISS-20260427T152954558Z-CORE-MEM-EXPOSES-RAW-ADDRESS-ESCAPE--4185EA5D`: `core/mem` が safe API として raw address escape hatch を公開している。
- `ISS-20260427T152958303Z-MEMPTR-AND-REGIONTOKEN-LACK-COMPILER-0BC8ECDF`: `MemPtr` / `RegionToken` に compiler-owned provenance model がない。

追加レビューでは、raw API の公開面だけでなく、typed に見える `MemPtr` API と compiler 内部許可境界にも問題が残っていることを確認した。

- `ISS-20260427T164412420Z-CORE-MEM-TYPED-MEM-COPY-AND-MEM-MOVE-621A41C7`: typed `mem_copy<T>` / `mem_move<T>` が `T: Copy` なしに non-Copy owner を byte copy できる。
- `ISS-20260427T164432612Z-CORE-MEM-DEALLOC-APIS-DO-NOT-ENCODE--204F1F47`: `dealloc_*` API が initialized storage の drop obligation を表現しない。
- `ISS-20260427T164425727Z-CORE-MEM-RAW-BODY-PRIVILEGE-IS-GRANT-043DAD95`: raw body / raw intrinsic の特権付与が SourceMap path suffix に依存している。
- `ISS-20260427T164419173Z-MEMORY-LAYOUT-RULES-ARE-DUPLICATED-A-FDB20787`: memory layout 規則が複数 pass/backend に重複し、raw byte range の検査と codegen がずれるリスクがある。

## 2026-04-28 raw memory intrinsic effect 部分対応

`#intrinsic "load"` / `#intrinsic "store"` が `intrinsic_effect` で pure 扱いになっていたため、user source が `core/mem` の wrapper を通さず raw memory を直接読み書きできる穴を `ISS-20260427T160936494Z-RAW-MEMORY-INTRINSICS-ARE-TREATED-AS-C0657AB6` として分離し、修正した。これにより direct raw memory intrinsic は pure context で `D3025` になる。移行中の `stdlib/core/mem.nepl` は SourceMap path による compiler-owned memory boundary として限定許可している。

## 2026-04-28 stdlib effect migration blocker 追記

compiler 側で `core/mem.nepl` の raw primitive を外部向け `Effect::Impure` として登録する試作を行ったところ、現行 stdlib の多くの pure API が raw memory backed helper に依存しているため、`Vec`、`string`、`io`、`fs`、`diag`、`stdio`、`streamio` で大量の D3025 が発生した。これは compiler の局所修正だけでは閉じられず、stdlib/API の段階的な effect migration または `InternalAlloc` / `UnsafeMemory` 相当の effect 導入が必要である。

この残件は `ISS-20260427T204839136Z-STDLIB-RAW-MEMORY-BACKED-APIS-REQUIR-E503CD84` として分離した。この親 issue は、raw primitives の最終的な external impure 化と Resource IR 化が完了するまで open のまま維持する。

## 2026-04-28 pure raw body helper call 部分対応

raw body の direct memory instruction は拒否済みだったが、`call $store_i32` / `call @mem_grow` のように Pure signature の raw helper wrapper を直接呼ぶ経路が残っていたため、`ISS-20260427T182409751Z-PURE-RAW-BODIES-CAN-CALL-RAW-MEMORY--7C283F24` として分離して修正した。compiler-owned raw memory boundary 以外の pure raw body では、既知 raw memory helper symbol への direct call も `D3025` で拒否する。

## 2026-04-28 suffixed raw helper symbol 部分対応

raw body direct callee の raw helper 判定が完全一致のみだったため、`store_i32__i32_i32__unit__pure` のような compiler generated / mangled symbol で raw memory helper boundary を迂回できる問題を `ISS-20260427T212039819Z-RAW-BODY-HELPER-EFFECT-DETECTION-MIS-8D69E368` として分離し、修正した。`raw_callee_is_raw_memory_effect` は direct callee を helper base name に正規化してから marker と照合するため、suffix 付き raw helper symbol も pure raw body では `D3025` になる。

## 2026-04-28 raw aggregate field read / branch merge 部分対応

`field::get load<T> p "field"` のような raw aggregate load 直後の field access が、raw address `p + offset` から field だけを読む HIR ではなく、`load<T> p` で non-Copy aggregate 全体を shallow load してから field を読む HIR になっていた。このため Copy field を読むだけでも raw place 全体が moved になり、collection helper の temporary raw storage が D3100 で誤検出されていた。

今回の対応では、field accessor lowering が raw aggregate load を検出した場合に `load<Field>(raw_addr + field_offset)` へ直接下げるようにし、不要な aggregate copy と所有権誤検出を避けた。あわせて raw place state に byte size を保持し、raw aggregate と raw field の重なりを検査するようにした。non-Copy field を raw aggregate から move した後に aggregate 全体を取り出す経路は D3100 のまま拒否する。

また、branch / loop の raw place state merge が、最初の branch と accumulator 初期値 `None` を merge していたため、全 branch で同じ `Initialized` の raw place でも `PossiblyMoved` に悪化する問題を修正した。

この修正で `tests/stdlib/bloom_filter_collections.n.md` の D3100 は解消した。一方、`tests/stdlib/byte_builder.n.md` の D3100 は `stdlib/std/test.nepl` が `Vec<Result<(),str>>` を同じ raw temp から複数回 by-value load している実際の所有権問題であり、`ISS-20260427T163710082Z-STD-TEST-LOADS-VEC-RESULT-FROM-RAW-T-BDF60069` として分離した。

## 2026-04-28 MemPtr raw alias 部分対応

同じ raw memory place を `MemPtr<T>` 経由で複数の i32 address に戻すと、既存の i32 alias tracking を迂回できる問題を `ISS-20260427T183234007Z-MOVE-CHECK-DOES-NOT-CANONICALIZE-MEM-CE6E5F55` として分離し、修正した。`mem_ptr_wrap` / `mem_ptr_addr` / `MemPtr` copy 由来の raw address は同じ raw place key に正規化され、non-Copy raw load の二重 move が D3100 になる。

## 2026-04-28 raw dealloc live payload 部分対応

raw place tracking は non-Copy `load<T>` / `store<T>` を扱っていたが、`dealloc_raw` / `dealloc_ptr` は initialized raw place state を見ていなかったため、live payload を storage-only dealloc で捨てられる穴が残っていた。この問題を `ISS-20260427T184214411Z-MOVE-CHECK-ALLOWS-RAW-DEALLOC-WITH-L-6543A0A2` として分離し、修正した。

今回の対応で、initialized または possibly moved の non-Copy raw place を含む range を dealloc すると D3100 になる。`load<T>` で ownership を消費してから storage を解放する経路は維持している。effect model と public raw API の設計不足は引き続きこの親 issue の残件である。

## 2026-04-28 RegionToken dealloc alias 部分対応

`RegionToken<T>` 経由の `dealloc_region` が raw place tracking に接続されず、raw/MemPtr dealloc 検査を迂回できる問題を `ISS-20260427T185057228Z-MOVE-CHECK-DOES-NOT-CONNECT-REGIONTO-665927E2` として分離し、修正した。`region_new` / `RegionToken` construct / `region_ptr` / `get token "ptr"` / `dealloc_region` は同じ raw place に正規化される。

この対応で RegionToken 経由の live non-Copy payload dealloc も D3100 になるが、raw API の effect 境界と compiler-owned Resource IR は未解決である。

## 2026-04-28 raw realloc live payload 部分対応

`realloc_raw` / `realloc_ptr` が old range の live non-Copy payload を検査せず、bytes として移動できる問題を `ISS-20260427T185656579Z-MOVE-CHECK-ALLOWS-REALLOCATING-RAW-S-45B12E2B` として分離し、修正した。

今回の対応で、old range に initialized または possibly moved の non-Copy raw place が残る realloc は D3100 になる。`load<T>` で payload ownership を消費してから storage を realloc する経路は維持している。realloc の public effect/API 境界と Resource IR での owner token 表現は引き続きこの親 issue の残件である。

## 2026-04-28 raw bulk copy live payload 部分対応

raw `mem_copy` / `mem_move` が `move_check` の raw place state を見ず、live non-Copy payload を shallow duplicate または byte overwrite できる問題を `ISS-20260427T190303188Z-MOVE-CHECK-ALLOWS-RAW-MEM-COPY-AND-M-AA0F96F9` として分離し、修正した。

今回の対応で、source range に initialized / possibly moved non-Copy raw place が残る bulk copy/move は D3100 になる。destination range が live non-Copy raw place を上書きする場合も D3100 になる。Copy bytes と payload consume 後の storage-only range は許可している。raw API の public safe surface と effect 境界は引き続きこの親 issue の残件である。

## 2026-04-28 raw byte write live payload 部分対応

`store_i32` / `store_u8` / generic Copy `store<T>` / `memset_u8` / `fill_i32` が live non-Copy raw place を byte overwrite できる問題を `ISS-20260427T190852368Z-MOVE-CHECK-ALLOWS-RAW-BYTE-WRITES-TO-B56A7B43` として分離し、修正した。

今回の対応で、copy-valued raw write でも destination range が initialized / possibly moved non-Copy raw place と重なる場合は D3100 になる。non-Copy `store<T>` は initialized state を作る経路のまま維持し、Copy storage への byte write と payload consume 後の storage-only write は許可している。

## 2026-04-28 MemPtr byte write / bulk copy 部分対応

raw address 版の byte write / bulk copy は拒否済みだったが、typed `MemPtr` overload の `store_i32` / `memset_u8` / `fill_i32` / `mem_copy` / `mem_move` が call-site raw place 分類から漏れていた問題を `ISS-20260427T212724800Z-MOVE-CHECK-ALLOWS-MEMPTR-BYTE-WRITES-9D19BC9D` として分離し、修正した。`MemPtr<T>` 由来の destination/source も raw place state に接続し、typed bulk copy の element count は byte size に換算して重なりを検査する。

## 2026-04-28 function raw memory effect summary 部分対応

direct call site の raw memory 検査は進んだが、helper 関数の内部に `store_i32` / `mem_copy` / `dealloc_ptr` などを隠すと caller の raw place state に副作用が伝わらない問題を `ISS-20260427T214055047Z-MOVE-CHECK-IGNORES-RAW-MEMORY-WRITES-417A7103` として分離し、修正した。

今回の対応で、関数サマリは戻り値 raw alias だけでなく raw memory load/store/dealloc/realloc/bulk copy/byte write effect も保持する。user function call では callee summary を caller 引数へ instantiate し、direct raw call と同じ D3100 ownership 検査を caller context で実行する。これにより、stdlib/self-host helper に raw memory operation を分割しても compiler の raw ownership state を迂回できなくなった。

## 2026-04-28 indirect function raw memory effect 部分対応

direct user call の raw memory effect は伝播するようになったが、`apply_clobber(p, f): f p` のような higher-order helper が function value 引数に raw memory operation を隠す問題を `ISS-20260427T215657067Z-MOVE-CHECK-LOSES-RAW-MEMORY-EFFECTS--BDFF8DD5` として分離し、修正した。

今回の対応で、`move_check` は `@fn` と function-typed parameter の function value alias を追跡し、`CallIndirect` で既知 callee の raw memory effect summary を indirect call 引数へ instantiate する。function-typed parameter が多段 helper に渡される場合も placeholder を保持し、outer call で concrete function value が渡された時点で D3100 raw ownership 検査を実行する。

## 2026-04-28 mem_ptr_add raw alias 部分対応

`mem_ptr_add<T>` が raw place 正規化に入っておらず、`mem_ptr_add p 0` で同じ storage を別 place として扱える問題を `ISS-20260427T191722304Z-MOVE-CHECK-DOES-NOT-CANONICALIZE-MEM-FEAEF49B` として分離し、修正した。

今回の対応で、literal offset の `mem_ptr_add` は base raw place + offset に正規化され、same-place alias 経由の non-Copy 二重 load や live-payload cleanup 検査回避は D3100 になる。未知 offset の aliasing と Resource IR による provenance model は引き続きこの親 issue の残件である。

## 2026-04-28 mem_ptr_add unknown offset 部分対応

`mem_ptr_add<T>` と raw `i32` address `add base off` の offset が literal でない場合に base provenance まで失われ、実行時に same-place / existing-offset alias になれる pointer が untracked になる問題を `ISS-20260427T192528620Z-MOVE-CHECK-LOSES-PROVENANCE-FOR-MEM--A1AE98CC` として分離し、修正した。

今回の対応で、non-literal offset の pointer add は `base+?` の unknown-offset raw place として保持され、同じ base の known raw place と保守的に overlap する。これにより、dynamic pointer arithmetic 経由の non-Copy 二重 load、live-payload store/dealloc、既知 nonzero offset payload との alias は D3100 になる。Resource IR と public raw API / effect 境界は引き続きこの親 issue の残件である。

## 2026-04-28 region_ptr_at Ok binding 部分対応

`region_ptr_at<T,U> token off` の `Result::Ok` payload を match bind した `MemPtr<U>` が元 `RegionToken` の raw place provenance を失い、bounds-checked projection 経由で raw ownership state を迂回できる問題を `ISS-20260427T194024586Z-MOVE-CHECK-LOSES-REGIONTOKEN-PROVENA-711BD515` として分離し、修正した。

今回の対応で、`region_ptr_at` の Ok payload bind は token raw place + offset に正規化される。literal offset は known place、non-literal offset は `base+?` として扱い、dynamic projection 経由の non-Copy 二重 load / live payload dealloc は D3100 になる。Result payload 全般の Resource IR 化と owner/non-owner 分離は引き続きこの親 issue の残件である。

## 2026-04-28 enum payload raw alias 部分対応

`Result::Ok p` や `region_ptr_at token off` の結果を一度 enum 変数へ束縛すると、match bind 時に payload の raw alias が復元されず、direct match 修正を迂回できる問題を `ISS-20260427T194927207Z-MOVE-CHECK-LOSES-RAW-ALIASES-STORED--5E0586DB` として分離し、修正した。

今回の対応で、`let` / `set` は enum payload raw alias を variant ごとに保存し、`match` payload bind はその alias を bind local へ引き継ぐ。branch merge では全 continuing branch で一致する alias だけ保持する。これにより、enum wrapper 変数経由の non-Copy 二重 load は D3100 になる。

## 2026-04-28 enum payload function raw effect 部分対応

function value / higher-order helper の raw memory effect は伝播するようになったが、`Option::Some @callback` のように callback を enum payload に包むと function value alias が保存されず、match-bind 後の `f p` が caller の raw ownership state に伝播しない問題を `ISS-20260427T221533970Z-MOVE-CHECK-LOSES-RAW-EFFECTS-THROUGH-308A8AC3` として分離し、修正した。

今回の対応で、`move_check` は enum payload function alias を `let` / `set`、branch merge、function summary instantiation、match payload bind に接続した。function-typed enum payload を持つ parameter には placeholder を seed し、outer call で concrete callback へ展開する。これにより、enum wrapper 経由の callback 内 raw memory write も D3100 raw ownership 検査を迂回できなくなった。

## 修正方針

`InternalAlloc` / `UnsafeMemory` のような内部 memory effect を導入し、raw identity が観測できない場合だけ surface `Pure` へ畳み込む。raw `load` / `store` / `alloc` / `dealloc` は unsafe 層または compiler-owned boundary に閉じ込める。Resource IR では memory token / place を表現し、non-Copy raw load は unrestricted copy ではなく owning place からの move として扱う。

## 検証

raw identity が観測可能な public raw memory operation を pure function から呼ぶ compile_fail を追加する。同じ raw place から non-Copy 値を繰り返し `load` する case も、将来の明示 unsafe escape がない限り拒否する ownership test を追加する。`MemPtr` safe overload の正常系は別途維持する。

2026-04-27 の部分対応では、`tests/compiler/move_effect.n.md` に non-Copy raw load の二重 move、raw address alias 経由の二重 move、未 move raw place への store overwrite、load 後の再初期化の回帰テストを追加した。

## 関連ドキュメント

- [NEPLg2 静的検査の複雑化解消計画](../../doc/neplg2/static_check_complexity_reduction_plan.md)

## 2026-04-28 issue 整理

この issue は静的検査大規模修正の Stage 5、つまり raw memory operation の effect / ownership boundary を追跡する親 issue とする。個別の D3100 / D3025 回帰修正は現行 checker を安全側に保つための child issue として扱い、この issue では最終的な `InternalAlloc` / `UnsafeMemory` / surface fold / Resource IR event 化を追跡する。

今後の修正では、raw memory helper をさらに call name summary で追い続ける方向を最終設計にしない。raw operation は Resource IR の `EffectOp` と storage/cell state の変化として表し、safe public surface から raw identity が漏れない場合だけ internal effect を `Pure` へ fold する。

## 2026-04-28 Stage 5 internal effect 分類追記

`doc/neplg2/static_check_complexity_reduction_plan.md` の Stage 5 commit 単位 1 として、surface `Effect::Pure` / `Effect::Impure` を壊さずに compiler 内部の effect 分類を追加した。

今回追加した `InternalEffect` は `Pure`、`InternalAlloc`、`UnsafeMemory`、`ExternalIo`、`Nondet` を分ける。`internal_effect_surface_fold` は `InternalAlloc -> Pure`、`ExternalIo` / `Nondet -> Impure`、`UnsafeMemory -> fold 不可` を表す。現行の `intrinsic_effect` は untrusted surface へ畳み込むため、`load` / `store` の raw intrinsic は引き続き pure context で D3025 になる。

Resource IR lowering では、raw allocation / deallocation / reallocation / memory size/grow を `EffectOp::InternalAlloc` とし、raw load/store/bulk copy/fill を `EffectOp::UnsafeMemory` として dump に残すようにした。通常の HIR user function call は名前だけで `ExternalIo` 扱いにせず、宣言済み surface effect を維持する。これにより、stdlib や user code の関数名に依存して Resource IR が過剰に impure 化されることを避ける。

回帰テストでは、mangled raw helper symbol の分類、`UnsafeMemory` の fold 不可、WASI I/O と nondet の分類、Resource IR dump の `internal_alloc` / `unsafe_memory(load|store)` を固定した。この段階では stdlib public API を一括変更せず、Stage 5 の後続で internal effect の escape 判定と stdlib boundary migration を進める。

## 2026-04-28 Stage 5 Resource IR effect boundary 追記

Stage 5 commit 単位 4 の public escape diagnostics へ進む前段として、Resource IR 上で internal effect boundary を非強制に検査する `check_resource_effect_boundaries` を追加した。

今回の検査は、pure function 内の `EffectOp::UnsafeMemory` と、`RawMemoryOp::Alloc` / `Realloc` 由来の raw identity が pure function の戻り値として外へ出る経路を診断する。前者は `UnsafeMemoryInPureFunction`、後者は `RawAddressEscapeFromInternalAlloc` として保持する。`DeclareLocal` / `Read` / `Assign` / `Move` と branch / loop / match の合流では raw identity alias を追跡するため、単純な temp だけでなく local や分岐値経由の return escape も検出対象になる。

この check はまだ compiler error には接続していない。現行 stdlib の raw-memory-backed API と self-host 用 collection が残っているため、enforcement へ切り替える前に stdlib boundary と owner token API を同期する必要がある。代わりに Stage 4 で追加した Resource IR shadow report に `effect_boundaries` を組み込み、verbose 時に effect 診断件数を観測できるようにした。

回帰テストでは、pure function が `alloc_raw` の raw address を返すケースと、pure function 内で raw `store` intrinsic を実行するケースを `nepl-core/tests/resource_ir.rs` に固定した。

## 2026-04-28 Stage 5 raw identity escape gate 追記

Resource IR effect boundary のうち、`InternalAlloc` 由来の raw address identity が pure user function の戻り値として public surface へ漏れる経路を compiler pipeline の D3025 に接続した。これは `doc/neplg2/static_check_complexity_reduction_plan.md` の Stage 5 commit 単位 4「public escape diagnostics」の最小 enforcement である。

今回 compiler error 化したのは `RawAddressEscapeFromInternalAlloc` だけである。`UnsafeMemoryInPureFunction` は現行 stdlib の raw-memory-backed API 移行と衝突しやすいため、引き続き shadow report に残す。`stdlib/core/mem.nepl` の compiler-owned raw memory boundary では raw identity を扱う互換 API が残るため、SourceMap capability を持つ file は今回の gate から除外する。

また、既存の D3100 raw ownership violation がある場合は旧 `move_check` の診断を先に返すようにした。例えば live non-Copy payload を `realloc_raw` で byte move するケースでは、raw identity escape より D3100 の方が根本原因を正確に示すためである。

回帰テスト:

- `tests/compiler/move_effect.n.md` に `pure から alloc_raw の raw address を返せない` を追加し、`diag_id: 3025` を固定した。

検証:

- `rustfmt --check nepl-core/src/compiler.rs`: pass
- `cargo check -p nepl-core --tests`: pass
- `cargo test -p nepl-core --test resource_ir -- --nocapture`: 23 passed
- `cargo test -p nepl-core --test effects -- --nocapture`: 21 passed
- `trunk build`: pass
- `node nodesrc/tests.js -i tests/compiler/move_effect.n.md --no-tree -o tmp/stage5-raw-identity-escape-focused.json -j 1`: total=98, passed=98

## 2026-04-28 Stage 5 aggregate raw identity escape 追記

`ISS-20260428T100333122Z-RESOURCE-EFFECT-GATE-MISSES-RAW-ALLO-0E0A15D1` として、internal allocation identity を struct / tuple / enum constructor に包むと public escape diagnostics から漏れる問題を分離した。

Resource IR の effect boundary check で `ResourceOp::Construct` の input identity を output へ伝播するようにし、`alloc_raw` 由来 address を aggregate に包んで pure function から返す経路も D3025 で拒否される。これは Stage 5 commit 単位 4 の public escape diagnostics の補強であり、raw memory operation 全体の effect migration や Stage 6 stdlib API 移行は引き続き本 issue の残件として扱う。

## 2026-04-28 Stage 5 call raw identity summary 追記

`ISS-20260428T101126311Z-RESOURCE-EFFECT-GATE-LOSES-RAW-ALLOC-CAE6E35F` として、internal allocation identity を pure helper call に通すと `ResourceOp::Call` で public escape diagnostics から漏れる問題を分離した。

Resource IR effect boundary checker に direct user function の parameter-to-return raw identity summary を追加し、`fn raw_id(p): p` のような helper の戻り値へ caller 側の allocation identity を伝播するようにした。summary 計算では raw allocation 自体を parameter identity と混ぜないため、raw pointer を読んで通常値を返す helper と、raw identity をそのまま返す helper を区別する。

## 2026-04-28 Stage 5 function value raw identity summary 追記

`ISS-20260428T101959179Z-RESOURCE-EFFECT-GATE-LOSES-RAW-ALLOC-1B0FA4ED` として、known function value 経由の indirect call で internal allocation identity が途切れる問題を分離した。

Resource IR effect boundary checker に function value alias table を追加し、`ResourceOp::FunctionValue` で得た関数名を local copy / branch / match を通して保持するようにした。`ResourceOp::IndirectCall` の callee が known alias を持つ場合は、direct call と同じ parameter-to-return raw identity summary を適用する。これにより `let f @raw_id; f p` のような first-class function value 経由でも raw identity escape が D3025 で拒否される。

## 2026-04-28 Stage 3 raw memory operation lowering 追記

`doc/neplg2/static_check_complexity_reduction_plan.md` の Stage 3 commit 単位 2 として、raw memory operation を Resource IR event として下げる入口を追加した。

今回の変更では、`ResourceOp::RawMemory` と `RawMemoryOp` を追加し、`alloc_raw` / `dealloc_raw` / `realloc_raw` / runtime ABI helper、`load` / `store`、`mem_copy` / `mem_move`、`mem_size` / `mem_grow` / `mem_fill` を call name summary だけでなく Resource IR dump に残る operation として表すようにした。`FuncRef::Builtin` だけでなく `FuncRef::User` でも `raw_callee_is_raw_memory_effect` を使うため、stdlib wrapper や mangled helper 名を経由しても `EffectOp::UnsafeMemory` と Resource IR event の両方に分類される。

この段階では storage owner、initialized cell、byte range overlap の enforcement はまだ旧 `move_check` 側が担う。Stage 4 以降で `RawMemoryOp` を `StorageId`、`CellState`、`PointerProvenance` の遷移へ接続し、現在の raw memory summary を Resource IR summary へ移行する。

## 2026-04-28 core / stdlib 全体レビューでの再発確認

`origin/main` の `8d0c6ab` 取り込み後に `node nodesrc/tests.js -i tests\compiler\move_effect.n.md --no-tree -o tmp\move-effect-audit-20260428.json -j 1` を実行したところ、`total=95`, `passed=43`, `failed=52` だった。多くは `compile_fail` を期待する raw memory / move effect 回帰テストが compile success になっており、この親 issue の残件である Resource IR / raw provenance / effect boundary がまだ安定していないことを示す。

その後 `origin/main` の `2ccb013` を merge した直後、`trunk build` 前の古い `web/dist` では remote 側で追加された 2 件を含めて `node nodesrc/tests.js -i tests\compiler\move_effect.n.md --no-tree -o tmp\move-effect-after-merge-20260428.json -j 1` が `total=97`, `passed=43`, `failed=54` になっていた。

ただしこれは build 未更新による古い compiler での確認だった。`trunk build` 後に `node nodesrc/tests.js -i tests\compiler\move_effect.n.md --no-tree -o tmp\move-effect-after-trunk-20260428.json -j 1` を実行すると、`total=97`, `passed=97`, `failed=0` になった。したがって、今回 remote main に入った borrow/raw memory 修正は回帰テスト上は有効である。

この親 issue は、raw memory operation 全体の public API / Resource IR / internal effect boundary を追跡する設計 issue として open のまま残す。個別 regression が通ったことは確認済みだが、safe surface から raw address escape を閉じる設計移行は未完了である。

borrow checker 周辺は別 agent の作業範囲だが、self-host compiler の AST / diagnostic / collection 実装では owner を持つ値を `Result` / `Option` / container / callback 経由で移動するため、ここが未解決のまま S3 以降に進むと所有権不整合を compiler 自身の実装へ持ち込む。S1/S2 の lexer/parser/module loader の純粋データ構造設計は開始できるが、raw memory backed buffer や move-sensitive lowering に依存する実装はこの issue の収束を待つ。
