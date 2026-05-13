# NEPLg2 静的検査の複雑化解消計画

作成日: 2026-04-28

## 目的

NEPLg2 の Rust compiler は、型検査、effect 判定、move/borrow/lifetime、drop 挿入、raw memory provenance を後付けで積み重ねてきた。その結果、`typecheck.rs` と `passes/move_check.rs` が巨大化し、修正ごとに局所的な summary や alias map を増やす構造になっている。

この文書は、静的検査を弱めずに、不必要な複雑化を解消するための大規模修正の仕様と実装計画を定める。目標は「検査を形だけ残す」ことではなく、memory safety、type safety、effect safety を compiler が一貫した中間表現で検査できる状態にすることである。

関連 issue:

- [ISS-20260425T000000Z-RV-CORE-002-D17C4B3C](../../issues/items/ISS-20260425T000000Z-RV-CORE-002-D17C4B3C.md): `typecheck.rs` / `move_check.rs` の責務集中。
- [ISS-20260425T000000Z-RV-CORE-009-58589A3F](../../issues/items/ISS-20260425T000000Z-RV-CORE-009-58589A3F.md): Resource IR 不在による move/borrow/drop の後付け実装。
- [ISS-20260427T132406497Z-CORE-MEM-RAW-MEMORY-OPERATIONS-BYPAS-DC67DF04](../../issues/items/ISS-20260427T132406497Z-CORE-MEM-RAW-MEMORY-OPERATIONS-BYPAS-DC67DF04.md): raw memory operation の effect / ownership 境界。
- [ISS-20260427T152958303Z-MEMPTR-AND-REGIONTOKEN-LACK-COMPILER-0BC8ECDF](../../issues/items/ISS-20260427T152958303Z-MEMPTR-AND-REGIONTOKEN-LACK-COMPILER-0BC8ECDF.md): `MemPtr` / `RegionToken` の provenance / owner model。
- [ISS-20260427T204839136Z-STDLIB-RAW-MEMORY-BACKED-APIS-REQUIR-E503CD84](../../issues/items/ISS-20260427T204839136Z-STDLIB-RAW-MEMORY-BACKED-APIS-REQUIR-E503CD84.md): stdlib raw-memory-backed API の段階移行。
- [ISS-20260429T040748194Z-RUST-COMPILER-DIAGNOSTICS-ARE-NOT-AL-1617747D](../../issues/items/ISS-20260429T040748194Z-RUST-COMPILER-DIAGNOSTICS-ARE-NOT-AL-1617747D.md): Resource IR / self-host model に合わせた compiler diagnostic 再設計。
- [ISS-20260506T022705566Z-MOVE-CHECK-DOCTESTS-ARE-STALE-AFTER--EDD8402F](../../issues/items/ISS-20260506T022705566Z-MOVE-CHECK-DOCTESTS-ARE-STALE-AFTER--EDD8402F.md): Resource IR field projection / Never merge / move_check doctest authority の同期。
- [ISS-20260506T091634072Z-RESOURCE-DROP-ELABORATION-CANNOT-BRI-88F5F95B](../../issues/items/ISS-20260506T091634072Z-RESOURCE-DROP-ELABORATION-CANNOT-BRI-88F5F95B.md): monomorphized Resource IR function と source HIR function の対応を `origin_name` metadata で保持する。
- [ISS-20260506T093453445Z-RESOURCE-DROP-ELABORATION-PLAN-IS-DI-E004B013](../../issues/items/ISS-20260506T093453445Z-RESOURCE-DROP-ELABORATION-PLAN-IS-DI-E004B013.md): checked `ResourceDropElaborationPlan` を compiler pipeline artifact として保持する。
- [ISS-20260506T094754766Z-RESOURCE-DROP-ELABORATION-PLAN-LACKS-5D3C7F54](../../issues/items/ISS-20260506T094754766Z-RESOURCE-DROP-ELABORATION-PLAN-LACKS-5D3C7F54.md): checked drop plan が source HIR origin / binding / scope span へ戻せることを gate する。
- [ISS-20260506T104708173Z-RESOURCE-IR-LOWERING-TREATS-CALLABLE-02721744](../../issues/items/ISS-20260506T104708173Z-RESOURCE-IR-LOWERING-TREATS-CALLABLE-02721744.md): bare callable value reference を local read と誤認しない Resource IR lowering / coverage rule。
- [ISS-20260506T111001704Z-RESOURCE-DROP-ELABORATION-PLAN-OMITS-C984305E](../../issues/items/ISS-20260506T111001704Z-RESOURCE-DROP-ELABORATION-PLAN-OMITS-C984305E.md): assignment overwrite drop obligation を checked Resource IR drop elaboration plan に含める。
- [ISS-20260506T113709479Z-RESOURCE-DROP-ELABORATION-PLAN-IS-NO-6CFFA860](../../issues/items/ISS-20260506T113709479Z-RESOURCE-DROP-ELABORATION-PLAN-IS-NO-6CFFA860.md): checked ResourceDropElaborationPlan を実 drop call 生成で消費し、旧 HIR VarState drop walker を削除する。
- [ISS-20260506T122605377Z-STDLIB-ALLOC-STRING-RAW-MEMORY-BOUND-7853986C](../../issues/items/ISS-20260506T122605377Z-STDLIB-ALLOC-STRING-RAW-MEMORY-BOUND-7853986C.md): configured stdlib `alloc/string.nepl` と `alloc/string/storage.nepl` を exact raw-memory-boundary capability として扱う。
- [ISS-20260506T123740149Z-STDLIB-RAW-MEMORY-BACKED-SCANNER-AND-338A3B52](../../issues/items/ISS-20260506T123740149Z-STDLIB-RAW-MEMORY-BACKED-SCANNER-AND-338A3B52.md): raw-memory-backed scanner / byte helper の Stage 5 boundary と Stage 6 public API 移行を整理する。
- [ISS-20260506T130126516Z-RESOURCE-OWNER-SUMMARIES-REJECT-FS-A-7E58243F](../../issues/items/ISS-20260506T130126516Z-RESOURCE-OWNER-SUMMARIES-REJECT-FS-A-7E58243F.md): fs / stdio read scratch owner cleanup の Resource IR owner summary。
- [ISS-20260506T130138471Z-KP-STREAM-SCANNER-FLOAT-DOCTESTS-EXC-0D4A3BF8](../../issues/items/ISS-20260506T130138471Z-KP-STREAM-SCANNER-FLOAT-DOCTESTS-EXC-0D4A3BF8.md): KP stream scanner float doctest runtime timeout。
- [ISS-20260506T134653279Z-RESOURCE-OWNER-SUMMARY-MISSES-RAW-DE-007EB7EA](../../issues/items/ISS-20260506T134653279Z-RESOURCE-OWNER-SUMMARY-MISSES-RAW-DE-007EB7EA.md): `unwrap_ok dealloc` 経由の checked raw owner consumption を Resource IR summary に反映する。
- [ISS-20260512T033056386Z-RESOURCE-OWNER-SUMMARIES-MATERIALIZE-BAE331D3](../../issues/items/ISS-20260512T033056386Z-RESOURCE-OWNER-SUMMARIES-MATERIALIZE-BAE331D3.md): `Result` payload owner を unconditional projection return として materialize せず variant owner return として扱う。
- [ISS-20260512T230732771Z-RESOURCE-OWNER-SUMMARY-TREATS-VARIAN-0C3269E3](../../issues/items/ISS-20260512T230732771Z-RESOURCE-OWNER-SUMMARY-TREATS-VARIAN-0C3269E3.md): Resource owner summary の variant path 条件を OR alternatives として保持し、到達可能 arm を誤って落とさない。
- [ISS-20260506T135746003Z-STRING-ACCESS-SPLIT-LOSES-RAW-MEMORY-8C64A912](../../issues/items/ISS-20260506T135746003Z-STRING-ACCESS-SPLIT-LOSES-RAW-MEMORY-8C64A912.md): `alloc/string/access.nepl` / `alloc/string/scanner.nepl` 分割後の exact raw-memory-boundary capability 追従。
- [ISS-20260506T150445017Z-STRING-INTEGER-SPLIT-LOSES-RAW-MEMOR-36A59D71](../../issues/items/ISS-20260506T150445017Z-STRING-INTEGER-SPLIT-LOSES-RAW-MEMOR-36A59D71.md): `alloc/string/integer.nepl` 分割後の exact raw-memory-boundary capability 追従。
- [ISS-20260506T152038161Z-STRING-BUILDER-SPLIT-LOSES-RAW-MEMOR-637613A4](../../issues/items/ISS-20260506T152038161Z-STRING-BUILDER-SPLIT-LOSES-RAW-MEMOR-637613A4.md): `alloc/string/builder.nepl` 分割後の exact raw-memory-boundary capability 追従。
- [ISS-20260506T154254968Z-RESOURCE-AUTHORITY-DEEP-PREFIX-REGRE-14D21232](../../issues/items/ISS-20260506T154254968Z-RESOURCE-AUTHORITY-DEEP-PREFIX-REGRE-14D21232.md): Resource IR authority path の deep-prefix compile-time budget 監査。
- [ISS-20260512T235355207Z-IMPORT-VISIBILITY-DOES-NOT-ENFORCE-P-30FB5573](../../issues/items/ISS-20260512T235355207Z-IMPORT-VISIBILITY-DOES-NOT-ENFORCE-P-30FB5573.md): Stage 6 の `core/mem` internal/public 分離の前提となる import visibility enforcement。
- [ISS-20260506T172100644Z-FS-AND-STDIO-SCRATCH-RAW-DEALLOC-LOS-57895CB4](../../issues/items/ISS-20260506T172100644Z-FS-AND-STDIO-SCRATCH-RAW-DEALLOC-LOS-57895CB4.md): fs / stdio private scratch dealloc が owner alias move 後の free obligation を失う。
- [ISS-20260506T173138867Z-RESOURCE-RAW-ADDRESS-LOWERING-EXCEED-B64EB6D1](../../issues/items/ISS-20260506T173138867Z-RESOURCE-RAW-ADDRESS-LOWERING-EXCEED-B64EB6D1.md): raw address lowering の return/source classification 責務が再集中している。
- [ISS-20260506T180609091Z-RESOURCE-INITIALIZED-ALIAS-MODULE-EX-BA05D57A](../../issues/items/ISS-20260506T180609091Z-RESOURCE-INITIALIZED-ALIAS-MODULE-EX-BA05D57A.md): initialized alias tracking module の raw alias group / value origin / i32 scalar fact 責務分割。

## 現状の問題

### 1. HIR 直走査に検査責務が集中している

現在の `move_check` は HIR tree を直接走査しながら、local variable state、borrow state、field move、raw memory place、function raw effect summary、enum payload alias、aggregate field alias を同時に扱っている。これにより、ある経路を塞ぐたびに別の容器や関数境界に対応する補助 map が増える。

この構造では、次の問いに単一の答えを持てない。

- この値は所有値か、borrow projection か、raw pointer か。
- この storage の free obligation は誰が持つか。
- この cell は initialized / moved / uninitialized のどれか。
- この borrow はどの resource id に紐づくか。
- この関数呼び出しは resource state をどう変化させるか。

### 2. `MemPtr` が複数の責務を兼ねている

`MemPtr<T>` は stdlib 上では Copy な non-owning pointer と説明されている。一方で、collection storage、single-cell owner、self-host outcome では owning storage handle としても使われている。

今後の方針は `MemPtr` を拡張し続けることではない。役割を次のように分ける。

| 役割 | 型・IR 表現 | 意味 |
|---|---|---|
| non-owning pointer | `MemPtr<T>` | Copy 可能な projection。free obligation を持たない。 |
| storage owner | `OwnedRegion<T>` / `Storage<T>` | allocator が発行した free obligation owner。Copy 不可。 |
| initialized cell | `InitializedCell<T>` / Resource IR cell state | 値が入っている、move 済み、drop obligation が残る、を表す。 |
| compiler capability | compiler-issued token | stdlib code から forge できない resource id / provenance。 |

### 3. effect が surface 表現しか持たない

現行の effect は主に `Pure` / `Impure` であり、raw allocation や internal buffer mutation を「外部から観測できない内部効果」として扱う層がない。そのため raw primitive を単純に impure 化すると stdlib が広範囲に壊れ、逆に pure のままにすると safe source から raw memory discipline を構成できる。

必要なのは、内部効果と surface effect の分離である。

| 内部 effect | surface fold | 条件 |
|---|---|---|
| `InternalAlloc` | `Pure` | raw identity / owner token が public surface へ漏れない。 |
| `UnsafeMemory` | fold 不可 | 明示 unsafe / compiler-owned boundary 内だけで許可する。 |
| `ExternalIO` | `Impure` | I/O など外部観測可能な効果。 |
| `Nondet` | `Impure` | 時刻、乱数、環境依存など。 |

### 4. drop 挿入と move check が別々に状態を推測している

`drop_insertion` は scope exit と structural field drop を見て drop を後付けする。`move_check` は別の走査で moved / borrowed / raw place state を推測する。この 2 つが同じ resource state を共有していないため、stdlib 側で drop loop を追加した場合に、将来の auto drop と衝突する危険がある。

## 目標仕様

### 検査の層

静的検査は次の依存方向に分ける。

| 層 | 責務 | 出力 |
|---|---|---|
| resolve | import、name、scope、overload candidate の収集 | resolved AST / symbol table |
| type inference | 型変数、trait capability、overload 決定 | typed HIR |
| effect inference | 関数 effect、internal effect、surface fold | effect signature |
| resource lowering | HIR から resource operation へ変換 | Resource IR |
| resource check | move、borrow、lifetime、initialized、drop obligation、raw provenance | checked Resource IR / diagnostics |
| drop elaboration | Resource IR 上で auto drop を挿入 | drop-elaborated Resource IR |
| backend lowering | WASM / LLVM 用 HIR または backend IR へ変換 | backend input |

後段は前段の内部実装へ戻ってはならない。特に resource check は `typecheck.rs` の local helper や HIR の表面的な call name 推測に依存しない。

### Resource IR の最小モデル

Resource IR は CFG を持つ中間表現とし、少なくとも次を第一級に表す。

| 要素 | 説明 |
|---|---|
| `ResourceId` | 所有値、storage owner、borrow target を識別する compiler-owned id。 |
| `Place` | local、field、enum payload、tuple field、storage offset、projection を表す。 |
| `StorageId` | allocator が発行した storage。byte range と layout plan を持つ。 |
| `CellState` | `Uninit` / `Initialized(T)` / `Moved` / `Dropped` / `MaybeMoved`。 |
| `OwnerState` | free obligation の有無、owner token の移動状態。 |
| `BorrowState` | shared borrow set、unique borrow、borrow lifetime end。 |
| `PointerProvenance` | `MemPtr` projection の base resource、offset、unknown-offset 保守情報。 |
| `EffectOp` | internal allocation、unsafe memory、external I/O、user call の resource effect。 |

### `MemPtr` / `Storage` / `InitializedCell` の規則

1. `MemPtr<T>` は non-owning pointer であり、Copy できる。
2. `MemPtr<T>` の copy は free obligation を複製しない。
3. `Storage<T>` / `OwnedRegion<T>` は Copy 不可であり、free obligation を持つ。
4. `Storage<T>` の projection から `MemPtr<T>` を作れるが、`MemPtr<T>` から `Storage<T>` は作れない。
5. initialized value を持つ cell を storage-only free することは禁止する。
6. `load<T>` は `T: Copy` の read と、non-Copy の move-out を分ける。
7. `store<T>` は uninitialized cell の initialize と、initialized cell の overwrite を分ける。non-Copy overwrite は既存 value の drop/consume が証明された場合だけ許可する。
8. raw address `i32` は compiler-owned internal boundary 外へ出さない。移行中は既存 API を `resource.cell.*` / `resource.owner.*` / `resource.raw.*` / `effect.*` 系の検査で保守的に塞ぎ、cell state と owner obligation の原因分類を混ぜない。

### function effect と resource summary

Resource IR 導入後の関数 summary は、現行の raw alias summary の延長ではなく、関数境界をまたぐ resource effect として表す。

- 引数 resource の consume / borrow / projection。
- 戻り値 resource の owner transfer / borrowed projection / copy value。
- storage cell の initialized / moved / dropped 変化。
- `InternalAlloc` が外部へ漏れたかどうか。
- unknown callback は保守的に effect set を上げる。

function value、enum payload、aggregate field、branch merge を別々の alias map で扱わず、Resource IR の `Place` と `EffectOp` に統合する。

## 実装計画

### Stage 0: 現状固定と回帰境界の明確化

目的: 大規模修正中に安全検査を弱めないため、既存の暫定防壁を固定する。

作業:

- `tests/compiler/move_effect.n.md` の raw ownership / raw effect regression を現行 baseline として維持する。
- raw memory / borrow / function effect 関連の compile_fail に `diag_code` を可能な範囲で固定する。
- `node nodesrc/issues.js check` と focused compiler test を CI / local の確認手順へ明記する。

commit 単位:

1. test naming と出力 JSON baseline の整理。
2. Resource IR 導入前提の regression 一覧更新。

### Stage 1: module 境界の切り出し

目的: behavior を変えず、`typecheck.rs` と `move_check.rs` の責務境界を作る。

作業:

- `typecheck.rs` から symbol/env、overload、trait lookup、effect inference、HIR lowering 補助を分割する。
- `move_check.rs` から raw helper classifier、place/provenance 型、branch merge、function summary 型を分割する。
- この段階では検査 semantics は変えない。

commit 単位:

1. 型定義と helper の移動のみ。
2. raw helper classifier の module 化。
3. function summary / branch merge 型の module 化。
4. diagnostics と tests の import path 調整。

### Stage 2: Resource IR 型定義と dump の追加

目的: 新しい検査モデルを実装前に可視化する。

作業:

- `nepl-core/src/resource/` を作成し、`ResourceModule`、`ResourceFunction`、`ResourceBlock`、`ResourceOp`、`Place`、`ResourceState` を定義する。
- HIR から Resource IR へ lowering する skeleton を追加する。
- 最初は enforcement しない dump / snapshot 用の IR として扱う。

commit 単位:

1. Resource IR data structure。
2. HIR lowering skeleton。
3. dump / debug snapshot test。

### Stage 3: Resource IR lowering の充実

目的: HIR の静的検査情報を Resource IR に移す。

作業:

- local let/set、function call、branch、loop、match、aggregate construction、field projection を Resource IR op に下げる。
- `MemPtr` projection、storage owner、raw load/store/dealloc/realloc/bulk copy を Resource IR op に下げる。
- HIR 直走査の raw alias 推測と Resource IR lowering の結果を比較する debug check を追加する。

commit 単位:

1. local / aggregate / branch lowering。
2. raw memory operation lowering。
3. function call / callback effect lowering。
4. old checker との comparison diagnostics。

### Stage 4: resource check への移行

目的: move/borrow/lifetime/drop obligation を Resource IR 上の検査へ移す。

作業:

- `CellState` と `OwnerState` による move / initialized 検査を実装する。
- `BorrowState` による shared / unique / lifetime end 検査を実装する。
- branch / loop merge を Resource IR state merge に統一する。
- old `move_check` は比較用に残し、差分がある場合は issue 化する。
- Resource IR diagnostic を粗い互換 bucket へ押し込まず、[compiler diagnostic 再設計計画](./compiler_diagnostics_redesign_plan.md)に従って cell / owner / borrow / lowering の stable code を保持する。

commit 単位:

1. initialized / moved state。
2. owner token / free obligation。
3. borrow / lifetime。
4. branch / loop merge。
5. old checker との gating 切り替え。

進捗:

- 2026-04-29: Resource IR owner obligation gate が generic aggregate store/load regression を拒否していた件を再確認し、原因が compiler 側の false positive ではなく test helper の `alloc_raw` storage leak であることを切り分けた。generic helper は `load<T>` 結果を保持してから `dealloc_raw` する形へ直し、free obligation model を弱めずに generic aggregate 回帰を通した。`List` / `HashMap` の `RawMemoryLoadCell Uninit` は stdlib raw-memory-backed collection / Resource IR lowering の別残件として扱う。
- 2026-05-06: Resource IR cell gate を raw-memory cell operation 専用から通常 read/move/drop/call/construct/branch/match/return まで広げた。`ResourceCheckDiagnostic::CellUnavailable` はすべて `resource.cell.*` として compiler diagnostic へ写像され、old move checker が見逃した通常 cell-state violation も Resource IR boundary で止める。残る Stage 4 の主な未完了点は old move checker と HIR drop insertion の統合削除である。
- 2026-05-06: `run_move_check` の実行順序を見直し、Resource IR lowering coverage / cell / borrow / effect / owner gate を旧 `passes::move_check::run` より先に実行するようにした。旧 checker は Resource IR gate 通過後の fallback 防壁として残す。これにより Resource IR diagnostic が legacy HIR diagnostic に fail-fast で隠される問題を解消した。回帰防止として `nodesrc/test_resource_gate_order.js` を source policy runner に追加した。残る Stage 4 の主な未完了点は、fallback として残る old move checker の削除と HIR drop insertion の Resource IR drop elaboration への統合である。
- 2026-05-06: `tests/compiler/move_effect.n.md` を Resource IR / effect gate 後の authority に合わせ直し、pure raw operation は `effect.pure.calls_impure`、raw cell state は impure fixture、move 後の raw load は `resource.cell.*` で検証する形へ整理した。あわせて direct `Result::Ok` payload match を介した raw address alias で canonical address が新規束縛名へ揺れ、moved cell が uninit と誤診断される問題を `RawCellAddressAliases` の合流規則で修正した。
- 2026-05-06: `tests/compiler/move_check.n.md` を Stage 4 Resource IR authority に合わせ直し、52/52 passing にした。`field::get_ref` は typed `get_field_ref` intrinsic と Resource IR `Borrow` lowering で field cell state を保持し、compiler-lowered `add &owner offset` も field projection として coverage / initialized check が扱う。`Never` value の branch / match arm は initialized-state merge から除外し、到達不能 path が reachable cell state を汚染しないようにした。残る Stage 4 の主な未完了点は、旧 `passes::move_check::run` fallback の削除と HIR drop insertion の Resource IR drop elaboration への統合である。
- 2026-05-06: `ISS-20260506T025727360Z-REMOVE-LEGACY-MOVE-CHECK-FALLBACK-AF-C143E79B` として、compiler pipeline から旧 `passes::move_check::run` fallback を削除し、`nepl-core/src/passes/move_check*` も compiled pass から除去した。`run_resource_static_check` は Resource IR lowering coverage / cell / borrow / effect / owner gate だけを実行する。fallback 削除で露呈した deep prefix chain の owner gate 膨張は、user function return raw-address alias を lowering で二重 materialize していたことが原因だったため、plain user call の identity / owner transfer は Resource IR summary gate に一本化した。残る Stage 4 の主な未完了点は、HIR `passes::insert_drops` を Resource IR drop elaboration へ統合することである。
- 2026-05-06: 旧 fallback 削除後の `tests/compiler/move_check.n.md` 52 件を Resource IR だけで通すため、borrow/lifetime gate は borrow token を aggregate / enum payload / field projection を含む prefix/suffix tree として伝播するようにした。Read / Move / Assign / Construct / Match bind / call return summary は exact local ではなく projected `Place` を基準に token を移す。branch / match arm の検査は外側 continuation を順序付きに見て、使用より前に token scope が終わる場合は外側 EndScope による過剰保持を避ける。これにより `move_check.n.md` は 52/52 passing になった。一方で `move_effect.n.md` は 105/110 で、raw address helper literal offset と higher-order / aggregate / enum payload function value raw write の effect/cell summary 残件があるため `ISS-20260506T005443213Z-MOVE-EFFECT-DOCTESTS-ARE-STALE-AFTER-6955AAD2` を再オープンした。
- 2026-05-06: `ISS-20260506T005443213Z-MOVE-EFFECT-DOCTESTS-ARE-STALE-AFTER-6955AAD2` を再解決し、`tests/compiler/move_effect.n.md` は 110/110 passing になった。専用 lowering を持たない user helper について、return expression が引数由来の raw address projection だけで構成される場合に限り Resource IR lowering で透明な return projection を発行する。unknown impure indirect call は `MemPtr` / `RegionToken` 引数を保守的な raw cell store release requirement として summary に反映し、高階関数、aggregate field、enum payload に保存された callback raw write を caller 側の initialized cell state 上書きとして検出する。旧 HIR checker fallback は復活させない。
- 2026-05-06: `ISS-20260425T000000Z-RV-CORE-009-58589A3F` の Stage 4 進捗として、compiler pipeline の Resource IR gate を HIR `passes::insert_drops` より前へ移した。Resource IR gate は typecheck 直後の未単相化 HIR 全体ではなく、drop 未挿入 source semantics を保持したまま monomorphize した reachable HIR を検査する。未単相化 HIR 全体を検査すると `#target std` の未使用 stdlib まで対象になり `move_effect.n.md::doctest#108` が timeout するためである。deep prefix chain では HIR の再帰 `clone()` も stack overflow するため、Resource IR 用 HIR と codegen 用 HIR は clone ではなく typecheck を二度実行して分離する。残る Stage 4 の主な未完了点は、HIR `passes::insert_drops` 自体を Resource IR drop elaboration へ置き換え、この二重経路を統合することである。
- 2026-05-06: Resource IR initialized/cell checker が `EndScope` で live non-Copy local を auto-drop state transition として扱うようにした。これにより、source Resource IR check は HIR `passes::insert_drops` の生成済み `drop` 式に依存せず scope exit の drop obligation を表現できる。同名・同型 shadowing では inner auto-drop が outer local を壊さないように、Resource IR lowering が有効範囲内の shadowed local place を `x#N` 形式で固有化する。残る Stage 4 の未完了点は、codegen 側の HIR `passes::insert_drops` を Resource IR drop elaboration の結果から生成する構造へ置き換えることである。
- 2026-05-06: EndScope auto-drop を checker 内部の暗黙処理に閉じず、`ResourceDropPlan` / `ResourceDropFunctionPlan` / `ResourceAutoDrop` / `ResourceAutoDropKind` として明示データ化した。`compute_resource_drop_plan` は nested Branch / Loop / Match を含む Resource IR を走査し、non-Copy scope local の auto-drop 候補を列挙する。initialized/cell checker も同じ候補列挙を使うため、次に codegen 側の HIR `passes::insert_drops` を置き換える際に、checker と codegen が別々の drop 対象推定を持たない。
- 2026-05-06: `ResourceDropPlan` の auto-drop 候補へ `ResourceDropRequirement` を追加し、`StateOnly` / `WholeValue` / `DynamicEnumPayload` / `Structural` を enum として分類するようにした。これにより、direct Drop impl、structural field Drop、runtime tag 依存の enum payload Drop を codegen 側が文字列や独自 flag で再推定しない。残る Stage 4 の未完了点は、この分類済み plan を実 drop call 生成へ接続し、HIR `passes::insert_drops` を削除することである。
- 2026-05-06: HIR `passes::insert_drops` の内部に残っていた drop-needed 再推定を削除し、`ResourceDropRequirement` を消費する `drop_lines_for_requirement` へ統合した。旧 `structural_drop_fields` / `structural_enum_field_drop_lines` / `type_needs_structural_drop` は削除済みで、partial field move でも残存 field の requirement を enum match で扱う。残る Stage 4 の未完了点は、HIR scope walker 自体を Resource IR drop elaboration へ置き換え、compiler pipeline から `passes::insert_drops` を外すことである。
- 2026-05-06: `ResourceDropFunctionPlan` に `drop_points` を追加し、EndScope 単位の auto-drop group を保持するようにした。flat `auto_drops` は `drop_points` から flatten した互換 view として維持する。これにより codegen 移行時に、nested block / branch / match の scope end を HIR 側で再推定せず、Resource IR の drop point を消費できる。残る Stage 4 の未完了点は、drop point を実 drop call 生成へ接続することである。
- 2026-05-06: `ResourceDropPoint` に `ResourceDropPointPath` を追加し、block id と `Op` / `BranchThen` / `BranchElse` / `LoopCondition` / `LoopBody` / `MatchArm` の enum step で EndScope の Resource IR 構造上の位置を保持するようにした。span だけに依存せず、codegen が typed path を辿れる形へ進める。残る Stage 4 の未完了点は、この path を実 drop call 挿入位置へ接続し、HIR scope traversal を削除することである。
- 2026-05-06: `ResourceDropPointPath` を実際の Resource IR op へ解決する `resolve_resource_drop_point_path` / `resolve_resource_drop_point_end_scope` を追加した。無効 path は `ResourceDropPointResolutionError` enum で分類し、block 不在、op index 範囲外、container step と実 op の不一致、match arm 範囲外、EndScope 以外の選択を黙って無視しない。これにより drop point path は単なる metadata ではなく、codegen が消費前に検証できる typed insertion anchor になった。残る Stage 4 の未完了点は、この EndScope resolver を HIR/Wasm drop call 生成へ接続し、`passes::insert_drops` の scope walker を削除することである。
- 2026-05-06: `ISS-20260506T083026784Z-RESOURCE-IR-DROP-PLAN-LACKS-LIVE-DRO-358D2C7E` として、candidate drop plan と live drop fact の混同を分離した。`ResourceFunctionCheck::auto_drop_points` は initialized-state traversal が実際に `Initialized` と判定して drop した point だけを保持し、move 済み outer local は live drop fact に出ない。あわせて non-Copy function parameter の EndScope anchor を Resource IR lowering に追加し、HIR `insert_drops` の outer parameter scope に残っていた drop obligation を Resource IR 上にも表現した。残る Stage 4 の未完了点は、この live drop fact を HIR/Wasm drop call 生成へ接続し、candidate plan ではなく checked state を codegen authority にすることである。
- 2026-05-06: `ISS-20260506T084621972Z-RESOURCE-IR-LIVE-DROP-FACTS-LACK-COD-9EB91BC5` として、`ResourceDropElaborationPlan` を追加した。これは candidate `ResourceDropPlan` ではなく、initialized-state checker が実際に auto-drop した `ResourceFunctionCheck::auto_drop_points` だけから作る codegen-facing plan である。構築時に function/check 対応、typed path の EndScope 解決、auto-drop place と EndScope locals の一致を `ResourceDropElaborationPlanError` enum で検証し、compiler pipeline でも Resource IR cell gate 直後に hard gate として実行する。残る Stage 4 の未完了点は、HIR `passes::insert_drops` の scope walker をこの checked live plan の消費側へ置き換えることである。
- 2026-05-06: `ISS-20260506T090109381Z-RESOURCE-DROP-ELABORATION-PLAN-LACKS-82B39C85` として、drop elaboration plan に source binding 名を持たせた。Resource IR lowering は `DeclareLocal` に `source_name` を記録し、shadowed local の内部 place が `x#...` になっても backend/HIR が参照する source 名 `x` を失わない。`ResourceDropElaborationDrop` は checked place、source_name、drop requirement を一体で保持し、binding が解決できない場合は `MissingDropBinding` enum error で hard gate する。残る Stage 4 の未完了点は、この source binding 付き plan を実際の HIR/Wasm drop call 挿入へ接続することである。
- 2026-05-06: `ISS-20260506T091634072Z-RESOURCE-DROP-ELABORATION-CANNOT-BRI-88F5F95B` として、monomorphized Resource IR function から source HIR function へ戻すための `origin_name` metadata を追加した。`HirFunction` は typecheck 時点の source 関数名を保持し、monomorphize で `name` が specialized symbol へ変わっても `origin_name` は維持される。`ResourceFunction` と `ResourceDropElaborationFunction` も `origin_name` を持つため、次の HIR/Wasm drop call 生成は mangled name の prefix parsing ではなく構造化 metadata で source function と対応できる。残る Stage 4 の未完了点は、この function origin / source binding / checked drop point を消費して HIR `passes::insert_drops` を削除することである。
- 2026-05-06: `ISS-20260506T093453445Z-RESOURCE-DROP-ELABORATION-PLAN-IS-DI-E004B013` として、`run_resource_static_check` が checked `ResourceDropElaborationPlan` を返し、`PreparedProgram` がそれを保持するようにした。これにより plan は gate で検証されて捨てられる metadata ではなく、codegen bridge が消費する compiler pipeline artifact になった。残る Stage 4 の未完了点は、この prepared plan を HIR/Wasm drop call 生成に渡し、旧 `passes::insert_drops` の scope walker を削除することである。
- 2026-05-06: `ISS-20260506T094754766Z-RESOURCE-DROP-ELABORATION-PLAN-LACKS-5D3C7F54` として、checked drop plan が source HIR 側の origin / binding / scope span へ戻せることを `validate_resource_drop_elaboration_hir_bridge` で検証するようにした。compiler pipeline は HIR `passes::insert_drops` の前にこの bridge gate を実行し、欠落は `ResourceDropElaborationHirBridgeError` enum から `resource.lower.incomplete` へ写像する。残る Stage 4 の未完了点は、この bridge 済み plan を実際の HIR/Wasm drop call 生成へ渡し、旧 scope walker の drop 対象推定を削除することである。
- 2026-05-06: `ISS-20260506T104708173Z-RESOURCE-IR-LOWERING-TREATS-CALLABLE-02721744` として、裸の callable value reference が `HirExprKind::Var` として Resource IR lowering に届いた場合でも、active local binding がなければ typed `origin_name` / function type から canonical function symbol を解決し、`ResourceOp::FunctionValue` として lowering するようにした。HIR coverage gate も同じ local-shadowing-aware callable rule へ更新し、coverage の scope state は `coverage_hir_scope.rs` に分離した。これにより function value を未初期化 local と誤診断せず、cell checker を弱めずに first-class function / branch return / lambda 系の false positive を解消する。残る Stage 4 の未完了点は、bridge 済み drop elaboration plan を HIR/Wasm drop call 生成へ接続し、旧 `passes::insert_drops` の scope walker を削除することである。
- 2026-05-06: `ISS-20260506T111001704Z-RESOURCE-DROP-ELABORATION-PLAN-OMITS-C984305E` として、`set` / `Assign` による initialized non-Copy target の上書き前 Drop obligation を `ResourceAutoDropKind::AssignmentOverwrite` として明示した。initialized-state traversal は target が到達時点で `Initialized` の場合だけ live overwrite drop fact を記録し、move 済み target の再初期化では記録しない。`ResourceDropElaborationPlan` は assignment path を typed resolver で検証し、source HIR bridge も `set` span / target binding を確認する。残る Stage 4 の未完了点は、ScopeLocal と AssignmentOverwrite の両方を消費して実 drop call を生成し、旧 `passes::insert_drops` の VarState scope walker を削除することである。
- 2026-05-06: `ISS-20260506T113709479Z-RESOURCE-DROP-ELABORATION-PLAN-IS-NO-6CFFA860` として、compiler pipeline の実 drop call 生成を checked `ResourceDropElaborationPlan` consumer へ置き換えた。`passes::insert_resource_drops` は `ResourceAutoDropKind::ScopeLocal` / `AssignmentOverwrite` を enum で分岐し、`ResourceDropRequirement` の exhaustive match から Drop call / structural field Drop / dynamic enum payload Drop を生成する。旧 HIR `VarState` / `var_stacks` scope walker と `passes::insert_drops` 呼び出しは削除済みであり、drop 対象を HIR から再推定する二重 authority は残さない。`prepare_module_for_codegen_with_source_map` は drop 未挿入の monomorphized HIR を Resource IR check し、その同じ HIR へ plan-based drop insertion を行い、final monomorphize で生成 Drop trait call を解決する。後挿入された Drop call の impl method body が欠落しないよう、`monomorphize_internal` は `HirModule.impls` に保持されている impl method function も function table へ再登録する。Stage 4 の主な残件は、この新 authority で full review / regression を通し、Stage 5/6 の raw memory / stdlib public API 境界へ進めることである。
- 2026-05-06: `ISS-20260425T000000Z-RV-CORE-009-58589A3F` の完了監査として、compiler pipeline に旧 `passes::move_check::run` fallback と旧 `passes::insert_drops` 呼び出しが残っておらず、checked `ResourceDropElaborationPlan` が `insert_resource_drops` で消費されることを再確認した。この親 issue は Resource IR authority 化完了として fixed にし、raw-memory-backed stdlib API / `MemPtr` owner token 分離 / collection drop obligation は既存 Stage 5/6 issue で追跡する。監査中に deep-prefix `check_pipeline` focused regression が local 240 秒 budget を超えたため、compile-time complexity / regression sizing 問題を `ISS-20260506T154254968Z-RESOURCE-AUTHORITY-DEEP-PREFIX-REGRE-14D21232` として分離した。
- 2026-05-06: `ISS-20260506T154254968Z-RESOURCE-AUTHORITY-DEEP-PREFIX-REGRE-14D21232` を解決した。通常 i32 copy を raw-address alias group として seed していた `copy_alias_or_seed` を廃止し、既存 raw relation だけを伝播する `copy_alias_if_tracked` と、`RawAddressAlias` / `RawAddressView` だけが seed する `copy_explicit_raw_address_alias` に分けた。raw memory address の local origin は alias group ではなく value-origin fact として保持し、canonicalize 時にだけ使う。さらに transparent raw-address return lowering は bare i32 parameter return を raw helper とみなさず、`add` / `sub` / `mem_ptr_*` / `region_*` など raw-address operation の operand に限定した。これにより deep-prefix Resource IR static check は 240 秒 timeout から 9.33 秒、prepare_codegen は 9.39 秒へ戻り、higher-order function value raw write regression は維持した。Stage 4 authority path の残件は、full review / regression を継続しつつ Stage 5/6 の raw memory boundary と stdlib public API 分離へ進むことである。
- 2026-05-06: `ISS-20260506T130126516Z-RESOURCE-OWNER-SUMMARIES-REJECT-FS-A-7E58243F` として、owner summary の false positive を checker 緩和ではなく Result owner effect の materialization と同一 storage replacement の明示で修正した。branch / match / return 境界では pending `Result` payload owner transfer を外側 state に渡す前に実体化し、unconditional consumption と variant-conditioned consumption の二重消費を避ける。fs/stdio private scratch は checked API の `Err` 握りつぶしではなく internal raw boundary の exact `dealloc_raw` に統一した。残る Stage 4 の主な残件は、`ISS-20260430T012045983Z-RESOURCE-IR-CANNOT-SUMMARIZE-RETURNE-2FDA4B38` の dynamic initialized range summary と、`ISS-20260506T134653279Z-RESOURCE-OWNER-SUMMARY-MISSES-RAW-DE-007EB7EA` の `unwrap_ok dealloc` checked consumption summary である。
- 2026-05-06: `ISS-20260506T134653279Z-RESOURCE-OWNER-SUMMARY-MISSES-RAW-DE-007EB7EA` として、`unwrap_ok` のような reachable `Result::Ok` arm だけを返す helper を `resolved_parameter_variants` summary として表現した。summary 収集は `Read` / `Move` / local initializer / assignment の透明な値 alias を辿り、`expr LocalRead` などの注釈 op では alias を消さない。一方で call / construct / borrow / raw / match output は変換値として alias を切る。これにより `dealloc` の `Result::Ok` success branch に保留された owner consume が `unwrap_ok dealloc` 経由で呼び出し元の raw owner に適用され、checked cleanup API を raw API へ落とさずに false positive を解消した。残る Stage 4 の主な残件は、`ISS-20260430T012045983Z-RESOURCE-IR-CANNOT-SUMMARIZE-RETURNE-2FDA4B38` の dynamic initialized range summary である。
- 2026-05-06: `ISS-20260506T172012873Z-RESOURCE-IR-DYNAMIC-RAW-ADDRESS-VIEW-77E94B53` として、dynamic raw address view が stable local origin を失う問題を修正した。`ValueOrigin` を exact place だけでなく prefix にも適用し、`tmp[+?]` を `%pref[+?]` のような stable origin plus suffix へ正規化する。通常 i32 copy は raw alias group を seed しないため、deep-prefix alias explosion を再発させずに、`fill_i32 pref pref_len 0` の dynamic initialized Copy range と後続の別 read 由来 `load_i32 add pref off` が同じ cell fact を参照できる。`kpread_to_kpwrite_prefixsum_i32` の `resource.cell.uninit` blocker は解消し、次の別件として fs/stdio scratch dealloc の `resource.owner.no_free_obligation` を `ISS-20260506T172100644Z-FS-AND-STDIO-SCRATCH-RAW-DEALLOC-LOS-57895CB4` に分離した。親 issue `ISS-20260430T012045983Z-RESOURCE-IR-CANNOT-SUMMARIZE-RETURNE-2FDA4B38` は、length/guard と結び付いた dependent range summary の残件として open のまま維持する。
- 2026-05-07: `ISS-20260506T172100644Z-FS-AND-STDIO-SCRATCH-RAW-DEALLOC-LOS-57895CB4` を解決した。原因は stdlib cleanup ではなく、`RawMemory::Alloc` temporary から local / enum payload / field へ owner transfer した後に `RawCellAddressAliases::move_owner_aliases` が owner mark だけを移し、raw owner value の alias group を再作成していなかったことだった。moved target と moved marked projection を再度 alias group に入れることで、通常 i32 copy は raw alias group を seed しない方針を維持しつつ、owner mark 済み storage root の exact read copy だけが `dealloc_raw` の free obligation へ解決される。`fs_open_with_flags__`、`fs_read_fd_bytes__`、`stdio_read_all_bytes_result__`、`stdio_write_fd_mem_result__` の scratch owner diagnostics を固定する Resource IR 回帰を追加し、`kpread_to_kpwrite_prefixsum_i32` も通過した。Stage 4 authority path の残件は full review / regression と、Stage 5/6 の raw-memory-backed stdlib API 境界整理へ移ることである。
- 2026-05-07: `ISS-20260506T173138867Z-RESOURCE-RAW-ADDRESS-LOWERING-EXCEED-B64EB6D1` を解決した。`lower_raw_address.rs` から transparent user return projection の解析を `lower_raw_address_return.rs` へ分離し、raw wrapper / actual call semantics と user return-expression classification の責務を分けた。`nodesrc/test_resource_checker_responsibility.js` には新 module の存在、`mod` 宣言、line limit、主要 entry point を追加した。これにより `lower_raw_address.rs` は 620 line limit を下回った。source policy は次の別件として `initialized_alias.rs` の責務集中に到達したため、`ISS-20260506T180609091Z-RESOURCE-INITIALIZED-ALIAS-MODULE-EX-BA05D57A` に分離した。
- 2026-05-07: `ISS-20260506T180609091Z-RESOURCE-INITIALIZED-ALIAS-MODULE-EX-BA05D57A` を解決した。`RawCellAddressAliases` から stable value origin を `initialized_alias_origin.rs` へ、i32 value / condition fact store を `initialized_alias_scalar.rs` へ分離した。raw address alias group と owner cell canonicalization は `initialized_alias.rs` に残し、branch merge は alias group / origin / scalar fact の各責務へ委譲する。これにより memory-safety-critical な raw owner alias table と、補助的な value-origin / condition fact が同一 file に再集中しない。source policy は Resource IR checker responsibility を warning 0 で通過した。
- 2026-05-12: `ISS-20260512T033056386Z-RESOURCE-OWNER-SUMMARIES-MATERIALIZE-BAE331D3` を解決した。`Result` payload owner が generic `ok` / `err` helper 経由で通常 `projection_returns` に残り、caller で runtime 不可能な `Ok` / `Err` payload owner まで同時に materialize される問題を、複数 variant payload が混在する projection return だけの variant return 正規化で修正した。あわせて raw owner summary alias walk は `read` / `DeclareLocal` / branch / match bind の projection suffix と `RawMemoryOp::Store` value consumption を保持し、`Result::Ok.field0` や raw node field に移した owner seed を見落とさない。`EndScope` owner auto-drop は `str` などの状態所有 leaf を落としつつ、`i32` raw address / `MemPtr` のような非所有 pointer を `StateOnly` として自動消費しないため、実 drop code が存在しない raw owner leak を隠さない。Stage 4 Resource IR owner summary は resolved variant と owner materialization の境界がより明確になった。
- 2026-05-07: `ISS-20260506T130138471Z-KP-STREAM-SCANNER-FLOAT-DOCTESTS-EXC-0D4A3BF8` を解決した。KP doctest timeout の主因は stdlib runtime ではなく、Resource IR summary builder が caller/callee 依存を見ずに全関数を全反復で再計算していた compile-time complexity だった。`initialized` / `owner` summary は in-place fixed point 更新と関数 summary dependency worklist に移行し、direct call / function value / nested branch / loop / match / self recursion の依存抽出を単体回帰で固定した。`NEPL_COMPILE_STAGE_TIMING=1` の host-only stage timing で `resource_static_check` は約 15.9 秒から約 6.7 秒へ低下し、`tests/stdlib/kp.n.md` focused suite は 7/7 passing になった。Stage 4 authority path の残件は、full review / regression を継続しつつ、残る compile-time hot path を別 issue として必要に応じて切り分けることである。
- 2026-05-07: `ISS-20260506T201433509Z-RESOURCE-CONDITION-FACTS-DROP-NONZER-5EE6B7A6` を解決した。`ResourceConditionFact` に typed `I32Relation` と `ResourceI32RelationOp` を追加し、`lt i len` のような nonzero relational guard を Resource IR に残す。zero-value fact は owner / realloc refinement 用に維持し、relation fact は exhaustive `match` で明示的に扱う。これは `ISS-20260430T012045983Z-RESOURCE-IR-CANNOT-SUMMARIZE-RETURNE-2FDA4B38` の returned raw header range summary に必要な typed precondition であり、残件は relation fact と symbolic raw offset を結び付ける range model である。
- 2026-05-07: `ISS-20260506T202600181Z-RESOURCE-RAW-OFFSETS-ERASE-SYMBOLIC--E5DDB5A0` を解決した。`ResourceOffset { bytes: Option<usize> }` に known byte offset / dynamic index / unknown wildcard を混在させる設計をやめ、`ResourceOffset::Known` / `Symbolic` / `Unknown` の enum に分離した。`RawAddressOffset` も simple dynamic offset place を `Symbolic` として保持し、`mem_ptr_add ptr idx` が `base[+symbolic]` として Resource IR に残る。general overlap 判定は `Symbolic` / `Unknown` を may-overlap として安全側に維持するため、静的検査を緩めずに後続 range summary の identity 情報だけを失わない。Stage 4 の残件は、`I32Relation` と `ResourceOffset::Symbolic` を照合して initialized byte range を typed fact として伝播する本体実装である。
- 2026-05-07: `ISS-20260506T203942617Z-RESOURCE-BRANCH-PATHS-DO-NOT-RETAIN--4242E13D` を解決した。`ResourceConditionFact::I32Relation` を branch path の `I32RelationFacts` に保存し、truthy branch は元 relation、false branch は negated relation として保持する。relation fact は value / unary condition から分離しつつ copy / prefix replacement / clear / merge の対象になり、query は reversed relation も扱う。これにより後続 range summary は HIR 条件式を再走査せず Resource IR state から `i < len` / `i >= len` を問い合わせられる。Stage 4 の残件は、保存済み relation fact と `ResourceOffset::Symbolic` を cell availability / initialized range fact へ接続することである。

- 2026-05-07: `ISS-20260506T210407334Z-INITIALIZED-RESOURCE-BRANCH-PATHS-DO-F88296F7` を解決した。owner checker だけでなく initialized checker の branch path でも `record_condition_fact_value_constraints` を適用し、then / else の `RawCellAddressAliases` に typed condition fact を反映する。既存 realloc condition handling はその後に適用するため、realloc 成否と relation proof の両方が path state に残る。Stage 4 の残件は、この initialized checker から参照可能になった relation proof と `ResourceOffset::Symbolic` を raw memory load の availability 判定へ接続することである。

- 2026-05-07: `ISS-20260506T211740745Z-SYMBOLIC-COPY-STORES-ERASE-UNKNOWN-O-0BD91F6C` を解決した。`RawMemoryOp::Store` の汎用 clear を store 専用の typed clearing に分け、symbolic Copy store が `pref[+?].deref` の initialized Copy fact を過剰に消さないようにした。non-Copy / moved / uninit state は従来どおり保守的に消すため、memory safety の緩和ではない。Stage 4 の残件は、loop condition fact と guarded initialized range summary を接続して、明示 proof のある dynamic raw load だけを通すことである。

- 2026-05-07: `ISS-20260506T212446487Z-RESOURCE-LOOPS-DO-NOT-CARRY-TYPED-CO-FD0086F2` を解決した。`ResourceOp::Loop` に `condition_fact` を追加し、`while lt i len` のような typed relation guard を Branch と同じ `ResourceConditionFact::I32Relation` として Resource IR に保持する。initialized / owner checker は condition evaluation 後、body path に truthy fact、exit path に negated false fact を適用してから state merge するため、loop body の range summary が HIR 条件式を再走査せず `RawCellAddressAliases` から `i < len` を問い合わせられる。Stage 4 の残件は、保持された loop relation fact と `ResourceOffset::Symbolic` を raw cell availability / initialized range fact へ接続することである。

- 2026-05-07: `ISS-20260506T215615927Z-RESOURCE-RAWADDRESSVIEW-TREATS-ORDIN-B3C620DA` を解決した。`RawAddressView` は lowering で `add` / `sub` から広めに生成されるため、checker 側で既存の raw-address proof がある場合だけ raw alias / non-owning view として扱う。proof は alias table の exact/prefix raw address、initialized checker の raw cell / owned raw storage / external raw storage、owner checker の owner state / storage origin に限定し、scalar `ValueOrigin` だけでは raw pointer とみなさない。さらに storage-offset view を local に束縛しても `pref[+?].deref` の broad initialized fact を view local へ rekey しない。これにより unrelated impure `i32` arithmetic が raw alias state を seed せず、`fill_i32 pref pref_len 0` 後の symbolic load と `kpread_to_kpwrite_prefixsum_i32` は通過する。Stage 4 の残件は、returned header / length field / guard relation をまたぐ dependent initialized range summary を typed model として表現することである。

- 2026-05-07: `ISS-20260506T203121413Z-COMPILER-STATIC-CHECKER-TIMES-OUT-ON-5B942F4A` を解決した。self-host lexer の empty `lex_all_with_file_id` smoke で `resource_initialized_moves` に入る前に止まっていた原因は、raw cell address return summary が `idx` / `file_id` のような通常 `i32` parameter まで raw address seed として扱い、token construction と branch merge を通じて bogus alias を膨張させていたことだった。summary 計算を `SummaryWorklist` に移行し、seed 対象を `MemPtr` / `RegionToken` / それらを含む aggregate / reference に限定した。明示的な `RawAddressAlias` / `RawAddressView` は引き続き raw relation を作れるため、raw helper の正当な i32 address return は lowering の typed ResourceOp が authority になる。timeout は owner diagnostics まで進む状態に改善し、残る lexer owner flow は `ISS-20260506T224618064Z-SELF-HOST-LEXER-OWNER-FLOW-FAILS-AFT-23CB5BBE` として分離した。Stage 4 の残件は、compiler summary の計算量を抑えたまま Resource owner diagnostics の正確性を維持し、self-host lexer の owner transfer を通すことである。

- 2026-05-07: `ISS-20260507T052014018Z-RESOURCE-IR-RETURNED-AGGREGATE-FIELD-F78CD903` を解決した。returned aggregate の `buf` / `len` field をまたぐ initialized raw range が caller local へ束縛される時に失われていた原因は、値コピーが range count だけを複写し、address / count の dependent pair を同時に projection copy していなかったことだった。`CellTable::copy_initialized_raw_byte_ranges_through_value` を `cell_state_raw_range_value.rs` に追加し、`DeclareLocal` / `Read` / `Assign` / `Move` / branch / match / raw memory `Load` / raw memory `Store` / aggregate `Construct` で initialized range の address と count を value projection として複写する。assignment / raw memory store では overwritten target 配下の stale range fact を消す。guard なし symbolic load は引き続き拒否し、`0 <= i && i < len` が Resource IR relation fact として証明された場合だけ通すため、静的検査の正確性は緩めていない。責務分割 policy は緩めず、追加 helper と既存 cover test を module 分離した。Stage 4 の残件は、returned aggregate projection 修正後も残る full scanner-style の `fd_read` loop / realloc / capacity field を含む dependent range model と、別件 `ISS-20260507T054543555Z-INITIALIZED-EXTERNAL-IO-EFFECT-EXCEE-5C420730` の external I/O effect helper 分割である。

- 2026-05-07: `ISS-20260507T054543555Z-INITIALIZED-EXTERNAL-IO-EFFECT-EXCEE-5C420730` を解決した。`fd_read` / `fd_pread` の single-iov payload range construction を `initialized_external_io_payload.rs` に分離し、`initialized_external_io_effect.rs` は external I/O effect entry point と nread exact-cell initialization に戻した。iovec descriptor 探索、single-iov 判定、payload alias filtering、`nread` を count とする `InitializedRawRangeUnit::Bytes` 登録は新 module が担当する。line limit は緩めず、`node nodesrc/test_resource_checker_responsibility.js` は passed になった。Stage 4 の残件は、full scanner-style の `fd_read` loop / realloc / capacity field を含む dependent range model である。

- 2026-05-07: `ISS-20260507T051545017Z-CELL-STATE-RAW-RANGE-EXCEEDS-SPLIT-L-76536EAC` を fixed として整理した。`cell_state_raw_range.rs` の責務超過は、raw range mutation / value projection / guarded cover proof / cover test を module 分離した結果、129/140 lines まで戻っている。`node nodesrc/test_resource_checker_responsibility.js` と `node nodesrc/run_source_policy_regressions.js --warn-only` はどちらも warning なしで通過しており、line limit を上げる形の回避はしていない。Stage 4 の残件は、full scanner-style の `fd_read` loop / realloc / capacity field を含む dependent range model である。

- 2026-05-07: `ISS-20260430T012045983Z-RESOURCE-IR-CANNOT-SUMMARIZE-RETURNE-2FDA4B38` を解決した。full scanner-style の `fd_read` loop / `realloc_raw` / returned header capacity field をまたぐ initialized raw range は、`sub next_cap cap` の typed scalar difference fact、append fill range composition、raw alias aware branch/loop/match range merge、`canonicalize_scalar` による count と non-owning pointer view の分離で表現する。guard なし symbolic load は引き続き拒否し、caller 側は `0 <= i && i < len && i < cap` が Resource IR fact として証明される場合だけ `load_u8 add data i` を許可する。Stage 4 の残件は、Resource IR authority path の full review / regression と、以後発見された owner / effect / borrow の個別 issue 解決である。

- 2026-05-07: `ISS-20260507T050025343Z-SHA256-HASH-DOCTEST-FAILS-RESOURCE-I-A4EE25CE` を解決した。match payload binding が外側 local / parameter を shadow した場合、Resource IR lowering が payload 初期化対象を `%e`、arm body の参照を `%e#0` に分裂させ、`sha256_rounds_loop` の `Result::Err e` を `resource.cell.uninit` と誤診断していた。match arm binding は `ctx.declare_local` が返す固有 Place を checked authority とし、drop elaboration bridge 用の source binding 名は `bind_source_name` として別に保持する。checker の cell-state 判定は緩めず、shadowed Copy payload と non-Copy payload drop bridge の Rust 回帰、および SHA-256 known-vector doctest で固定した。Stage 4 の残件は、full scanner-style の dependent range model と、Resource IR authority path の継続的な full regression である。

- 2026-05-07: `ISS-20260427T152958303Z-MEMPTR-AND-REGIONTOKEN-LACK-COMPILER-0BC8ECDF` の部分対応として、`str_addr` と borrowed `region_ptr` の lowering を `RawAddressViewKind::NonOwningProjection` として表現するようにした。`RawAddressViewKind::Offset` と分けることで、通常 i32 arithmetic の raw pointer proof gate は維持しつつ、仕様上 non-owning な pointer projection は source 未解決でも dealloc / realloc の owner として扱わない。`mem_ptr_addr` と `str_from_addr_unchecked` の owner transfer 経路は残しており、静的検査を緩めずに `str_addr` helper 経由の free bypass を拒否する。Stage 4 の残件は、`MemPtr = non-owning pointer` と `OwnedRegion/Storage = free obligation owner` の最終分離である。

- 2026-05-07: `ISS-20260507T085434323Z-RESOURCE-OWNER-CHECKER-LOSES-NON-OWN-344F2372` を解決した。`str_addr` 由来の non-owning raw address view は direct local では owner として拒否されていたが、`Result::Ok` などの aggregate payload に入れてから match bind / read を通ると non-owning raw view fact が落ちていた。Resource IR owner summary は payload projection に non-owning raw view marker を生成できるため、summary 生成を緩めるのではなく、`RawAddressViewTable` で通常 raw address view と non-owning raw address view を分け、construct / branch / match / call return summary / read の value-preserving owner-flow に non-owning fact copy を接続した。これにより `OwnerState::NoFreeObligation` を pointer authority として流用せず、`MemPtr = non-owning pointer` と `OwnedRegion/Storage = free obligation owner` の分離方針を弱めず、payload 経由の `dealloc_raw` / `realloc_raw` も `OwnerUnavailable` で拒否する。Stage 4 の残件は、Resource IR authority path の full review / regression と、stdlib public API 側の owner token 分離である。
- 2026-05-07: `ISS-20260507T125821563Z-RESOURCE-INITIALIZED-SUMMARY-MODEL-E-992DF2EE` を解決した。returned / param / variant param の raw byte range と `KnownI32` / projection count enum を `initialized_summary_byte_range_model.rs` へ分離し、`initialized_summary.rs` を function summary と raw cell/variant requirement contract に戻した。line limit は緩めていない。Stage 4 の残件として、次に露出した `initialized_summary_apply.rs` の responsibility split を `ISS-20260507T130937432Z-RESOURCE-INITIALIZED-SUMMARY-APPLY-E-7FFA13D6` で追跡する。
- 2026-05-07: `ISS-20260507T130937432Z-RESOURCE-INITIALIZED-SUMMARY-APPLY-E-7FFA13D6` を解決した。caller-side param cell / param byte range application と `RawCellInitializationParamCount` の解決を `initialized_summary_apply_param.rs` へ分離し、`initialized_summary_apply.rs` を summary lookup と application orchestration に戻した。Stage 4 の残件として、次に露出した `initialized_summary_byte_ranges.rs` の builder 分割を `ISS-20260507T131613193Z-RESOURCE-INITIALIZED-SUMMARY-BYTE-RA-F56D00D0` で追跡する。
- 2026-05-07: `ISS-20260507T131613193Z-RESOURCE-INITIALIZED-SUMMARY-BYTE-RA-F56D00D0` を解決した。returned / param raw byte range builder と count-source extraction を 4 module に分離し、`initialized_summary_byte_ranges.rs` の集中を削除した。Stage 4 の残件として、次に露出した `initialized_summary_variant_build.rs` の分割を `ISS-20260507T132339456Z-RESOURCE-INITIALIZED-SUMMARY-VARIANT-32AEE691` で追跡する。
- 2026-05-07: `ISS-20260507T132339456Z-RESOURCE-INITIALIZED-SUMMARY-VARIANT-32AEE691` を解決した。variant param cell / byte range の uniqueness helper を `initialized_summary_variant_unique.rs` へ分離し、`initialized_summary_variant_build.rs` を variant return path traversal と variant-gated summary construction に戻した。`node nodesrc/test_resource_checker_responsibility.js` と `node nodesrc/run_source_policy_regressions.js --warn-only` は warning なしで通過しており、Stage 4 ResourceIR proof module の責務分割 policy は現時点で緑である。
- 2026-05-07: `ISS-20260507T134613401Z-RESOURCE-OWNER-SUMMARY-IGNORES-NON-O-9A39F228` を解決した。`str_addr` 由来の non-owning raw view を `mem_ptr_wrap` / `region_new` で `RegionToken` に詰め直し、`dealloc_region` の helper parameter consumption 経由で free obligation owner に見せる経路を拒否する。callee owner summary が consumed parameter projection を要求する場合、caller actual が non-owning raw address view なら `resource.owner.no_free_obligation` を出すようにし、owner consumption の責務は `owner_consumption.rs` に分離した。これは `MemPtr = non-owning pointer` と `OwnedRegion/Storage = free obligation owner` の分離を Stage 4 owner summary 適用境界へ拡張する対応である。
- 2026-05-07: `ISS-20260507T143247279Z-RESOURCE-IR-OWNER-CHECKER-LOSES-NON--66D5734F` を解決した。`region_ptr_at<T,U>` の `Result::Ok(MemPtr<U>)` payload は borrowed `RegionToken` 由来の bounds-checked non-owning projection だが、stdlib 実装が `region_token_ptr_ref` / `mem_ptr_addr` / `mem_ptr_wrap` / `Result::Ok` を経由するため owner summary に non-owning view fact が残らず、`region_new` で forged owner token に見せられた。`region_ptr_at` の Ok payload raw field と borrowed `region_token_ptr_ref` の raw field を Resource IR lowering 時点で `NonOwningProjection` として表現し、coverage gate は reference projection の HIR/ResourceIR 対応を保ったまま `RawAddressView` target を alias metadata として扱うようにした。これにより `region_ptr_at` の正常な pointer read/write と元 token の dealloc は維持しつつ、Ok payload を owner に昇格する経路は `resource.owner.no_free_obligation` で拒否する。Stage 4 の残件は、compiler-issued owner token と stdlib public API の最終分離である。
- 2026-05-13: `ISS-20260512T202418246Z-RESOURCE-OWNER-POLICY-DOES-NOT-GUARD-2B46D8D5` を解決した。Stage 4 の `NonOwningProjection` は `MemPtr` projection が free obligation owner に戻らないための compiler-side 境界だが、source policy はこれまで enum の存在と module 分割しか見ていなかった。`nodesrc/test_resource_checker_responsibility.js` で `raw_address_view_carries_owner_alias` と `non_owning_raw_view_return_kind` の match arm を直接監視し、`RawAddressViewKind::NonOwningProjection => false` と `RawAddressViewOwnership::NonOwningProjection => ProjectionView` を wildcard なしで固定した。これにより `MemPtr = non-owning pointer` と `OwnedRegion/Storage = free obligation owner` の分離を regression policy として維持する。
- 2026-05-13: `ISS-20260512T202946482Z-TYPECHECK-CONSTRUCTOR-CAPABILITY-BOU-14965EAB` を解決した。`MemPtr` / `RegionToken` の direct constructor restriction は compiler-issued raw pointer / owner token capability の入口であり、後段 Resource IR だけに任せると forged token が typed HIR に残る。`nodesrc/test_static_check_boundary_responsibility.js` で `StructConstructorPolicy` / `RestrictedStructConstructor` enum、raw-memory-boundary based の `MemPtr => RawPointer` / `RegionToken => OwnerToken` 分類、`apply_struct_constructor` の `RawMemoryBoundaryOnly` gate、個別 diagnostic code への分岐を監視する。これにより Stage 4 の owner capability boundary を typecheck source policy でも維持する。
- 2026-05-13: `ISS-20260427T152958303Z-MEMPTR-AND-REGIONTOKEN-LACK-COMPILER-0BC8ECDF` を compiler core 側では fixed / resolved とした。`MemPtr` / `RegionToken` direct constructor boundary、`NonOwningProjection` raw view、`StorageOrigin::Owned`、returned owner summary、helper consumption gate、source policy が揃い、fixed raw address や borrowed projection を free obligation owner に偽装する主要経路は Resource IR / typecheck で拒否される。`tests/stdlib/memory_safety.n.md` は 23/23、`tests/compiler/move_effect.n.md` は 110/110、`region_token_forged` は 6/6 passing である。残る `core/mem` public API、collection/string/self-host buffer の safe public discipline は Stage 6 stdlib issue へ分離して継続する。
- 2026-05-07: `ISS-20260507T143850332Z-CLI-CHECK-DOES-NOT-RUN-RESOURCEIR-ME-D1F139FF` を解決した。`check_module_with_source_map` は typecheck で止まらず、compile preparation と同じ非再帰 prepare phase を共有して Resource IR lowering coverage、initialized cell、drop plan、borrow、effect、owner、drop bridge gate を通す。`--check` は成果物 emission には進まないが、compile/codegen pipeline と同じ memory/resource safety authority を通るため、CI や selfhost tooling が check-only を使っても Resource IR diagnostic を取りこぼさない。deep prefix regression と CLI Resource IR regression で、stack overflow 回避と static gate enforcement の両方を固定した。
- 2026-05-12: `ISS-20260512T032320909Z-RESOURCE-OWNER-SUMMARY-REPORTS-STDIO-C9FC40C9` を解決した。`print_i32` / `ansi_text_style_code` で露出した `str` temporary の `resource.owner.maybe_leak` は、stdio が `str` を消費すべきという問題ではなく、Resource IR lowering が allocation-returning Copy `str` temporary の statement lifetime を表現していなかったことが原因だった。HIR block line ごとに新規 top-level op の temporary output を確認し、Copy だが state-only owner scoping が必要な `str` temporary へ line-end `EndScope` を挿入する。非 dropped line result は `EndScope.result` で保存するため、関数返却値や block result を誤って消費しない。これにより stdio/ANSI false positive は解消し、`resource.owner.maybe_leak` 自体は弱めていない。examples の残り 1 件は `cliarg_count` / `cliarg_get` の raw argv scratch owner flow であり、`ISS-20260512T041752474Z-RESOURCE-OWNER-SUMMARY-REPORTS-CLIAR-97FEDA3D` として分離した。

### Stage 5: effect model の拡張

目的: raw memory を safe surface から閉じつつ、stdlib 内部の正当な allocation を表現する。

作業:

- internal effect と surface fold を導入する。
- raw memory primitive は compiler-owned boundary では `InternalAlloc` / `UnsafeMemory` として扱う。
- public pure API から raw identity が漏れた場合は fold 不可として `resource.raw.*` / `effect.*` の diagnostic にする。
- user source から raw address escape を構成できる経路を compile_fail にする。
- raw identity escape と ordinary impure call を同じ表示 bucket に依存させず、[compiler diagnostic 再設計計画](./compiler_diagnostics_redesign_plan.md)の `resource.raw.*` / `effect.*` code へ分ける。

commit 単位:

1. effect enum / fold 関数。
2. raw primitive effect 分類。
3. stdlib internal boundary の暫定許可。
4. public escape diagnostics。

進捗:

- 2026-05-06: compiler-owned raw-memory-boundary capability は `SourceCapabilities` と SourceMap を通して Resource IR effect gate に届く。`UnsafeMemoryInPureFunction` は raw-memory-boundary でない source では `effect.pure.calls_impure` として error 化済みである。
- 2026-05-06: `ISS-20260506T122605377Z-STDLIB-ALLOC-STRING-RAW-MEMORY-BOUND-7853986C` として、configured stdlib の `alloc/string.nepl` と `alloc/string/storage.nepl` を `core/mem.nepl` と同じ exact raw-memory-boundary capability の対象に加えた。これは string / str owned storage helper の内部 raw `load` / `store` / `bulk_copy` を Stage 6 移行中に許可するもので、stdlib 全体や arbitrary suffix path を許可するものではない。`Loader` は configured `stdlib_root` から canonical path を計算し、該当する exact path だけを許可する。
- 2026-05-06: wasm doctest で、`alloc/io.nepl` / `alloc/string/utf8.nepl` / `std/text.nepl` / `std/streamio/scanner/state.nepl` にも同種の raw-memory-backed boundary 未整理が残ることを確認し、`ISS-20260506T123740149Z-STDLIB-RAW-MEMORY-BACKED-SCANNER-AND-338A3B52` として分離した。これらは安易に stdlib 全体を許可せず、true internal boundary と safe public wrapper の責務を確認してから exact capability か Stage 6 API 移行で解く。
- 2026-05-06: `ISS-20260506T123740149Z-STDLIB-RAW-MEMORY-BACKED-SCANNER-AND-338A3B52` として、`alloc/io.nepl` / `alloc/string/utf8.nepl` / `std/text.nepl` / `std/streamio/scanner/state.nepl` を configured exact boundary table に追加した。`tests/stdlib/kp.n.md` から `effect.pure.calls_impure` は消え、残りは fs/stdio read owner summary、`pref` dynamic range summary、f64/f32 runtime timeout として分離された。
- 2026-05-06: remote main の string responsibility split 後、`alloc/string/access.nepl` の `len` / `string_byte_at_unchecked` と `alloc/string/scanner.nepl` の scanner byte helper が exact raw-memory-boundary capability に追従しておらず、`effect.pure.calls_impure` が再発した。`ISS-20260506T135746003Z-STRING-ACCESS-SPLIT-LOSES-RAW-MEMORY-8C64A912` として、両 module を configured stdlib の exact boundary table に追加した。Stage 6 移行完了までの internal string layout boundary は、module split ごとに loader capability table と regression を同時更新する。
- 2026-05-06: remote main の integer conversion split 後、`alloc/string/integer.nepl` の `from_u128_radix` が raw `store_u8` で文字列 buffer を構築するにもかかわらず exact raw-memory-boundary capability に追従していなかった。`ISS-20260506T150445017Z-STRING-INTEGER-SPLIT-LOSES-RAW-MEMOR-36A59D71` として `alloc/string/integer.nepl` を loader の exact boundary table に追加した。併せて `alloc/string/float.nepl` は直接 raw memory 操作を持たず `StringBuilder` / integer conversion へ委譲していることを確認し、過剰な raw boundary capability は付与しない。
- 2026-05-06: KP doctest の次 blocker として、`alloc/string/builder.nepl` の `sb_append_result` / `sb_append_byte_result` / `sb_build_result` が raw `store_u8` / `mem_copy` を使うにもかかわらず exact raw-memory-boundary capability に追従していないことを確認した。`ISS-20260506T152038161Z-STRING-BUILDER-SPLIT-LOSES-RAW-MEMOR-637613A4` として `alloc/string/builder.nepl` を loader の exact boundary table に追加した。StringBuilder は owned byte buffer の内部構築境界であり、Stage 6 の owner-token API 移行が完了するまでは safe public surface ではなく compiler-owned internal boundary として扱う。
- 残件は、raw-memory-backed stdlib public API を Stage 6 で internal/public 境界へ分け、raw identity と owner token が safe surface へ漏れない最終 API に移行することである。
- 2026-05-13: `ISS-20260427T132406497Z-CORE-MEM-RAW-MEMORY-OPERATIONS-BYPAS-DC67DF04` を core compiler 側では verified / resolved とした。`UnsafeMemoryInPureFunction` hard gate、Resource IR raw identity / cell / owner gate、exact raw-memory-boundary capability regression が揃っており、user source から raw memory operation を pure bypass する元の問題は閉じている。Stage 6 の stdlib internal/public API 移行は引き続き stdlib issue 側で追跡する。
- 2026-05-13: `ISS-20260513T023254911Z-CORE-MEM-FACADE-STILL-CARRIED-RAW-ME-FEEF633F` を解決した。`core/mem.nepl` root は public facade に縮小し、`types` / `raw` / `allocator` / `pointer` submodule へ実装責務を分離した。loader の exact raw-memory-boundary capability も root facade から外し、実装 submodule のみに付与する。raw helper の public re-export 閉鎖は未完だが、Stage 6 の前提である「public facade 自体が raw boundary privilege を持たない」状態に進めた。
- 2026-05-13: `ISS-20260427T152954558Z-CORE-MEM-EXPOSES-RAW-ADDRESS-ESCAPE--4185EA5D` を解決した。`core/mem` root facade は `types` / `layout` / checked `pointer` API だけを公開し、`mem_ptr_wrap` / `mem_ptr_addr` / `region_new` / raw allocator / raw load-store は internal/raw implementation module へ閉じた。compiler resolver も private import を public facade 越しに推移公開しないため、`#import "core/mem" as *` から raw address escape を構成できない。残る Stage 6 の焦点は、direct internal/raw module の利用 discipline、Vec/StringBuilder/collection/self-host buffer の owner token API 移行、stdlib 全体の raw-memory-backed public API 整理である。

### Stage 6: stdlib memory API の段階移行

目的: compiler の Resource IR と stdlib の公開 API を同期する。

作業:

- `core/mem` を internal raw module と safe public wrapper に分ける。
- collection は `Copy` read、borrowed read、owned remove/pop、container drop を API と型制約で分ける。
- `dealloc_*` は storage-only dealloc と initialized payload destruction を分ける。
- self-host compiler の buffer / diagnostic / outcome は raw `MemPtr` を直接持たず、safe wrapper を使う。

commit 単位:

1. `core/mem` internal/public 境界。
2. `Vec` / `StringBuilder` の owner token 移行。
3. collection drop contract。
4. self-host buffer API 移行。

### Stage 7: 旧 summary の削除

目的: 複雑化の原因になっていた HIR 個別 summary を取り除く。

作業:

- raw alias / enum payload alias / aggregate field alias / function value raw effect summary を Resource IR summary へ統合する。
- `move_check.rs` の旧 state map を削除する。
- `drop_insertion` を Resource IR drop elaboration へ統合する。

commit 単位:

1. old summary read path の停止。
2. old summary 型の削除。
3. old move_check / drop_insertion の統合削除。

## Issue 整理方針

| issue | 位置づけ | 完了条件 |
|---|---|---|
| `RV-CORE-002` | Stage 1 の親 issue。module 境界と責務分離を追跡する。 | `typecheck.rs` / `move_check.rs` の主要責務が module 化され、focused regression が維持される。 |
| `RV-CORE-009` | Stage 2-4 の親 issue。Resource IR と resource check を追跡する。 | Resource IR 上で move/borrow/lifetime/drop obligation を検査し、旧 checker 依存を除去する。 |
| `CORE-MEM-RAW-MEMORY-OPERATIONS-BYPAS` | Stage 5 の compiler-core issue。raw memory effect / ownership boundary を追跡した。 | 2026-05-13 に core 側は verified / resolved。stdlib public API migration は Stage 6 の stdlib issue へ分離する。 |
| `MEMPTR-AND-REGIONTOKEN` | Stage 3/4 の compiler-core issue。`MemPtr` / owner token / initialized cell の compiler 側分離を追跡した。 | 2026-05-13 に core 側は fixed / resolved。stdlib public API と collection/string/self-host buffer の移行は Stage 6 の stdlib issue へ分離する。 |
| `CORE-MEM-EXPOSES-RAW-ADDRESS-ESCAPE` | Stage 5/6 の stdlib public API issue。 | 2026-05-13 に fixed。safe `core/mem` import から raw address escape は呼べない。direct internal/raw module と raw-memory-backed stdlib 全体の discipline は Stage 6 parent で継続する。 |
| `CORE-MEM-DEALLOC-APIS-DO-NOT-ENCODE` | Stage 4 の compiler-core drop obligation issue。 | 2026-05-13 に core 側は fixed / resolved。initialized payload を残した storage-only free は拒否され、collection element cleanup と raw-memory-backed public API migration は Stage 6 の stdlib issue へ分離する。 |
| `STDLIB-RAW-MEMORY-BACKED-APIS-REQUIR` | Stage 6 の stdlib migration parent。 | raw-memory-backed implementation が safe public discipline を漏らさない。 |
| `RV-STDLIB-004` | Stage 6 の collection API issue。 | collection drop / remove / borrowed read / Copy read の責務が分離される。 |

新しい個別 bug は、次の基準で追加する。

- 現行 checker の false negative / false positive が明確なら、既存 regression child issue として追加する。
- Resource IR 導入でまとめて直すべき構造問題なら、`RV-CORE-009` の子として追加する。
- stdlib API 移行が必要な場合は、compiler issue と混ぜず `STDLIB-RAW-MEMORY-BACKED-APIS-REQUIR` または該当 stdlib issue へ分ける。

## 検証計画

### focused local tests

大規模修正中は、変更箇所に応じて focused test を選ぶ。

| 変更 | local test |
|---|---|
| issue metadata / docs | `node nodesrc/issues.js check` |
| Resource IR 型定義 / lowering | `cargo test -p nepl-core --test move_check`、Resource IR snapshot test |
| move/borrow/lifetime | `cargo test -p nepl-core --test move_check`、`tests/compiler/move_check.n.md` focused run |
| effect / raw memory | `tests/compiler/move_effect.n.md` focused run |
| stdlib memory API | 該当 `tests/stdlib/*.n.md` focused run |

全体 test は GitHub Actions を主に使い、local では変更に関係する範囲に絞る。

### regression 必須カテゴリ

- same-place raw load の二重 move。
- `MemPtr` copy は許可されるが free obligation は複製されない。
- live non-Copy payload を含む storage dealloc / realloc / bulk copy / byte overwrite の拒否。
- enum payload / aggregate field / function return / callback 経由の resource effect 伝播。
- branch / loop merge 後の maybe-moved / maybe-borrowed の保守的検査。
- unique borrow 中の write / move / dealloc 拒否。
- shared borrow 中の mutation 拒否。
- internal allocation が public raw identity を漏らさない場合だけ surface pure へ fold されること。

## self-host への影響

NEPLg2 self-host compiler は、S1/S2 の lexer/parser/module loader など pure data model から進められる。ただし、S3 以降の resource checker、diagnostic buffer、AST arena、token buffer、byte/string builder は、この文書の memory model を前提にする。

self-host 実装側の禁止事項:

- `MemPtr` を owner として保持する新規 public API を増やさない。
- raw address `i32` を compiler data structure の通常 field に持ち込まない。
- drop obligation を stdlib の手作業 cleanup だけで完結させる設計にしない。

許容される移行措置:

- 既存 `Vec` / `StringBuilder` を使った S1/S2 実装。
- raw-backed implementation を internal module に閉じた wrapper として使う。
- Resource IR 導入前の暫定 compiler regression を維持するための保守的 `resource.cell.*` / `resource.owner.*`。

## 2026-04-30 設計確認

[静的検査設計確認 2026-04-30](./static_check_design_verification_20260430.md) で、現行 Rust 実装、self-host 計画、stdlib memory model の整合を再確認した。

[静的検査 soundness review 2026-04-30](./static_check_soundness_review_20260430.md) では、pass 順序、現在の authority、Resource IR gate の hard-error 範囲、旧 HIR checker / shadow-only behavior に残る未完了点を追加で確認した。

判定は次の通り。

- Resource IR の data model、coverage gate、CellState / OwnerState / BorrowState gate、enum-first diagnostic の方向性は妥当である。
- 現行 pipeline は drop 未挿入 source semantics を monomorphize した reachable HIR を Resource IR check に渡し、checked `ResourceDropElaborationPlan` を生成する。実 drop call 生成は `passes::insert_resource_drops` がこの plan を消費して行うため、旧 `passes::move_check::run` fallback と旧 HIR `passes::insert_drops` 呼び出しはいずれも 2026-05-06 に削除済みである。生成 drop が source violation を隠さないよう、Resource IR gate は drop 挿入前の HIR に対して実行する。
- `ResourceCheckDiagnostic::CellUnavailable` と `ResourceOwnerDiagnostic::*` は compiler diagnostic で `resource.cell.*` と `resource.owner.*` に分離済みである。今後も D3100 相当の粗い raw bucket に戻さず、原因分類を enum-first で維持する。
- `UnsafeMemoryInPureFunction` は 2026-05-06 時点で Resource IR gate から `effect.pure.calls_impure` へ error 化済みである。ただし、configured stdlib の `core/mem.nepl`、`alloc/string.nepl`、`alloc/string/storage.nepl` など compiler-owned raw-memory-boundary capability を持つ source では、Stage 6 の stdlib migration が完了するまで移行中許可を維持する。この許可は loader の configured `stdlib_root` から計算した exact path に限定し、任意の同名 suffix path は許可しない。
- self-host の S1/S2 は進められるが、S3 以降の typecheck / Resource IR / diagnostic aggregation では raw header collection や `MemPtr` owner discipline を中核に持ち込まない。

追加精査で、`ResourceDiagnosticCode` 自体は `Move` / `Borrow` / `Cell` / `Owner` / `Raw` / `Lower` に分離済みであることを確認した。2026-05-06 時点で drop elaboration authority も checked Resource IR plan に統合済みであり、旧 HIR scope walker を維持する方針は残さない。設計上の未完了点は、raw-memory-boundary capability による stdlib 移行中許可が残っていること、stdlib の owner token / collection storage state が compiler-issued capability に揃い切っていないこと、Stage 4 authority path の full review / regression を継続することである。

2026-05-06 の Stage 5 追記として、host effect operation と raw/host effect count は enum-first の Resource IR 表現へ移行済みである。`ExternalIo` / `Nondet` / `UnsafeMemory` は pure function 境界で Resource IR diagnostic から compiler error へ接続される。残件は、raw-memory-backed stdlib の public API を Stage 6 で internal/public 境界へ分け、raw identity と owner token を safe surface へ漏らさない形へ移行することである。

Resource checker の責務分割 policy も確認し、`initialized_summary_variant_build.rs` が監視対象から漏れていたため `ISS-20260430T062912063Z-RESOURCE-CHECKER-RESPONSIBILITY-POLI-CC55287A` で修正した。2026-05-13 には `ISS-20260512T201359246Z-RESOURCE-CHECKER-RESPONSIBILITY-POLI-E382D3AB` として、後続 Stage 4/5 で追加された Resource IR module も全て行数上限と宣言検査の対象へ入れた。今後 Resource IR module を分割した場合は、実装だけでなく `nodesrc/test_resource_checker_responsibility.js` の責務上限も同時に更新しなければ source policy が失敗する。

したがって、この計画の完了条件は変更しない。旧 checker の special-case や旧 drop walker を戻して現状維持するのではなく、残る raw-memory-backed stdlib public API、owner token、collection storage state を Resource IR / enum / match の設計へ移す。

## 完了条件

この計画は次を満たした時点で完了とする。

1. `typecheck.rs` と `move_check.rs` の主要責務が module 境界へ分離されている。
2. Resource IR が typed HIR 後の正式な検査入力になっている。
3. move / borrow / lifetime / initialized / drop obligation / raw provenance が Resource IR 上で共有状態として検査される。
4. `MemPtr` は non-owning pointer に限定され、owner token と initialized cell state が別表現になっている。
5. raw memory primitive は public pure surface から閉じられ、必要な内部効果だけが surface pure へ fold される。
6. stdlib collection / string / self-host buffer が safe public discipline と compiler Resource IR の責務分割に従う。
7. 旧 HIR 個別 summary を削除しても、既存 memory safety / type safety / effect safety regression が通る。
