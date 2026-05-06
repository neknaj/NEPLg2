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
| `CORE-MEM-RAW-MEMORY-OPERATIONS-BYPAS` | Stage 5 の親 issue。raw memory effect / ownership boundary を追跡する。 | raw memory primitive が public pure surface から閉じ、Resource IR で ownership event として扱われる。 |
| `MEMPTR-AND-REGIONTOKEN` | Stage 3/6 の設計 issue。`MemPtr` / owner token / initialized cell の分離を追跡する。 | `MemPtr` が non-owning pointer に限定され、free obligation は compiler-issued owner token へ移る。 |
| `CORE-MEM-EXPOSES-RAW-ADDRESS-ESCAPE` | Stage 5/6 の stdlib public API issue。 | safe import から raw address escape を呼べない。 |
| `CORE-MEM-DEALLOC-APIS-DO-NOT-ENCODE` | Stage 4/6 の drop obligation issue。 | initialized payload を残した storage-only free が拒否される。 |
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
- ただし、現行 pipeline はまだ `passes::insert_drops` を Resource IR check より前に HIR 上で実行している。旧 `passes::move_check::run` fallback は 2026-05-06 に削除済みだが、drop elaboration が HIR 側に残る限り、drop obligation の最終設計は完了していない。
- `ResourceCheckDiagnostic::CellUnavailable` と `ResourceOwnerDiagnostic::*` は compiler diagnostic で `resource.cell.*` と `resource.owner.*` に分離済みである。今後も D3100 相当の粗い raw bucket に戻さず、原因分類を enum-first で維持する。
- `UnsafeMemoryInPureFunction` は 2026-05-06 時点で Resource IR gate から `effect.pure.calls_impure` へ error 化済みである。ただし、`stdlib/core/mem.nepl` など compiler-owned raw-memory-boundary capability を持つ source では、Stage 6 の stdlib migration が完了するまで移行中許可を維持する。
- self-host の S1/S2 は進められるが、S3 以降の typecheck / Resource IR / diagnostic aggregation では raw header collection や `MemPtr` owner discipline を中核に持ち込まない。

追加精査で、`ResourceDiagnosticCode` 自体は `Move` / `Borrow` / `Cell` / `Owner` / `Raw` / `Lower` に分離済みであることを確認した。設計上の未完了点は、Cell / Owner category の追加ではなく、HIR `insert_drops` がまだ drop elaboration authority として残っていること、raw-memory-boundary capability による stdlib 移行中許可が残っていること、stdlib の owner token / collection storage state が compiler-issued capability に揃い切っていないことである。

2026-05-06 の Stage 5 追記として、host effect operation と raw/host effect count は enum-first の Resource IR 表現へ移行済みである。`ExternalIo` / `Nondet` / `UnsafeMemory` は pure function 境界で Resource IR diagnostic から compiler error へ接続される。残件は、raw-memory-backed stdlib の public API を Stage 6 で internal/public 境界へ分け、raw identity と owner token を safe surface へ漏らさない形へ移行することである。

Resource checker の責務分割 policy も確認し、`initialized_summary_variant_build.rs` が監視対象から漏れていたため `ISS-20260430T062912063Z-RESOURCE-CHECKER-RESPONSIBILITY-POLI-CC55287A` で修正した。今後 Resource IR module を分割した場合は、実装だけでなく `nodesrc/test_resource_checker_responsibility.js` の必須 module 一覧と行数上限も同時に更新する。

したがって、この計画の完了条件は変更しない。旧 checker の special-case を増やして現状維持するのではなく、drop elaboration、owner/cell state authority、stdlib collection owner state を Resource IR / enum / match の設計へ移す。

## 完了条件

この計画は次を満たした時点で完了とする。

1. `typecheck.rs` と `move_check.rs` の主要責務が module 境界へ分離されている。
2. Resource IR が typed HIR 後の正式な検査入力になっている。
3. move / borrow / lifetime / initialized / drop obligation / raw provenance が Resource IR 上で共有状態として検査される。
4. `MemPtr` は non-owning pointer に限定され、owner token と initialized cell state が別表現になっている。
5. raw memory primitive は public pure surface から閉じられ、必要な内部効果だけが surface pure へ fold される。
6. stdlib collection / string / self-host buffer が safe public discipline と compiler Resource IR の責務分割に従う。
7. 旧 HIR 個別 summary を削除しても、既存 memory safety / type safety / effect safety regression が通る。
