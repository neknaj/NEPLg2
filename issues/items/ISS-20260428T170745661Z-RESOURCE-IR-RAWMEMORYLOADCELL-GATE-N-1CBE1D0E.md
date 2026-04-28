---
id: ISS-20260428T170745661Z-RESOURCE-IR-RAWMEMORYLOADCELL-GATE-N-1CBE1D0E
title: "Resource IR RawMemoryLoadCell gate needs raw pointer summaries"
area: core
status: open
resolved: false
priority: P1
type: bug
created: 2026-04-28
updated: 2026-04-29
target: "nepl-core/src/resource/initialized.rs, nepl-core/src/compiler.rs"
---

# ISS-20260428T170745661Z-RESOURCE-IR-RAWMEMORYLOADCELL-GATE-N-1CBE1D0E: Resource IR RawMemoryLoadCell gate needs raw pointer summaries

## 概要

RawMemoryLoadCell diagnostics cannot be made authoritative yet because Resource IR CellState still loses raw cell initialization across pointer-returning helpers, MemPtr address wrappers, and projection-based raw field access. Enabling the full load-cell gate produces false D3100 before the intended D3025/raw ownership diagnostics.

## 対象

- `nepl-core/src/resource/initialized.rs, nepl-core/src/compiler.rs`

## 根拠

- `doc/neplg2/static_check_complexity_reduction_plan.md` Stage 4 は CellState による initialized / moved 検査を Resource IR 上へ移す計画である。
- 2026-04-29 の destructive raw cell gate 作業中に `RawMemoryLoadCell` も compiler error 化すると、`tests/compiler/move_effect.n.md` の helper-returned slot、`MemPtr` wrapper、projection field access で false D3100 が出た。
- `RawMemoryStoreCell` / `RawMemoryDeallocCell` / `RawMemoryReallocCell` / `RawMemoryFillCell` / bulk cell は focused regression を通過したが、load cell は raw pointer summary が不足しているため別の完了条件を必要とする。

## 問題

RawMemoryLoadCell diagnostics cannot be made authoritative yet because Resource IR CellState still loses raw cell initialization across pointer-returning helpers, MemPtr address wrappers, and projection-based raw field access. Enabling the full load-cell gate produces false D3100 before the intended D3025/raw ownership diagnostics.

## 影響

Stage 4 cannot fully replace old move_check for raw load before store or repeated non-Copy raw load. Keeping RawMemoryLoadCell shadow-only is necessary until function and projection summaries preserve the pointed cell state without false positives.

## 修正方針

Add Resource IR summaries for raw pointer parameter-to-return transfer, MemPtr/RegionToken address wrappers, and raw aggregate field projection loads, then enable RawMemoryLoadCell diagnostics in the compiler gate after focused move_effect and move_check regressions pass.

## 検証

Add Resource IR unit tests for helper-returned raw slot load, MemPtr wrapper raw slot load, and projection field raw load. Run cargo test -p nepl-core --test resource_ir, trunk build, and node nodesrc/tests.js for tests/compiler/move_effect.n.md and tests/compiler/move_check.n.md.

## 関連ドキュメント

- [NEPLg2 静的検査の複雑化解消計画 Stage 4](../../doc/neplg2/static_check_complexity_reduction_plan.md)
- [RV-CORE-009 Resource IR parent](./ISS-20260425T000000Z-RV-CORE-009-58589A3F.md)

## 2026-04-29 調査追記

`RawMemoryLoadCell` gate を一時的に compiler error 化して `tests/compiler/move_effect.n.md` を実行したところ、最初の大きな false positive 群は Resource IR lowering ではなく CellState checker 側の二重消費だった。direct raw memory helper は Resource IR 上で `ResourceOp::Call` と `ResourceOp::RawMemory` の両方を持つため、generic `Call` が先に store value を moved にし、直後の `RawMemory::Store` が pointed cell を initialized にできていなかった。

この前提不備は `ISS-20260428T173213551Z-RESOURCE-CELLSTATE-CHECKER-CONSUMES--DD20A3D7` として分離し、修正済み。修正後に同じ一時 gate 調査を再実行すると、`move_effect.n.md` の失敗は 18 件から 11 件へ減った。残件は helper-returned slot の direct/indirect return summary、`MemPtr` / `RegionToken` の address wrapper、raw aggregate field load の offset/projection CellState に絞られる。

2026-04-29 追記:

`ISS-20260428T174427199Z-RESOURCE-CELLSTATE-RAW-ADDRESS-ALIAS-45DC270E` として、CellState 側の raw address alias が helper return、known function-value indirect call、aggregate field に渡らない問題を分離し、修正済みにした。direct helper / function-value helper / `PtrBox.ptr` style wrapper の Resource IR regression は `RawMemoryLoadCell` 診断なしで通る。

この修正で helper-returned slot と `MemPtr` / `RegionToken` style wrapper の主要な alias loss は CellState に移った。親 issue には、raw memory へ格納した aggregate の field offset/projection と、compiler field projection load と raw linear memory load の区別不足が残る。

2026-04-29 追記 2:

`ISS-20260428T175617166Z-RESOURCE-CELLSTATE-EXPRESSION-MARKER-26479BD3` として、HIR lowering が semantic `ResourceOp` の直後に出す `ResourceExprKind` marker で raw address alias を消していた問題を修正した。temporary `RawMemoryLoadCell` gate で `move_effect.n.md` は 99/110 から 101/110 へ改善し、doctest#13/#14 の helper-returned slot false D3100 は解消した。

残件は主に `MemPtr` / `RegionToken` address wrapper の raw cell initialization transfer、raw aggregate stored value の field offset/projection、通常 aggregate field projection load と raw linear memory load の分類不足である。

2026-04-29 追記 3:

`ISS-20260428T182913710Z-RESOURCE-IR-LOWERING-MISCLASSIFIES-C-A1508C51` として、通常 aggregate field access の compiler-generated `load` を Resource IR で raw memory load と誤分類していた問題を修正した。Resource IR lowering と coverage comparison の両方に `TypeCtx` 付き分類を追加し、aggregate pseudo-address からの field read は `PlaceProjection::Field` / `TupleField` の `ResourceOp::Read` へ下げる。

一時 `RawMemoryLoadCell` gate では `move_check.n.md` が 51/52 から 52/52 に改善し、`move_effect.n.md` は 101/110 から 104/110 に改善した。通常 Copy aggregate field read の false D3100 / D3101 は解消済みである。残件は 6 件で、`MemPtr` / `RegionToken` address wrapper の raw cell initialization transfer と raw memory に格納した aggregate の field offset/projection に絞られる。

2026-04-29 追記 4:

`ISS-20260428T185304640Z-RESOURCE-IR-LOWERING-FORCES-WHOLE-RA-C9E6E941` として、raw memory に格納した aggregate の field access が whole raw aggregate load に潰れる問題を修正した。typecheck は `get load<Holder> p "ptr"` を `load(add(p, offset))` へ早期変換せず、Resource IR lowering は source-level `get` / `get_field` を `p.*.field` の `ResourceOp::Read` として扱う。old `move_check` も同じ preserved 形を field projection として扱い、Copy field read では raw place 全体を move しない。

一時 `RawMemoryLoadCell` gate では `move_effect.n.md` が 101/110 から 105/110 に改善し、raw aggregate Copy field read 系 `#76` - `#79` は解消した。残件は 5 件で、helper-returned slot / `MemPtr` wrapper 由来の raw pointer summary と load-cell gate 順序に絞られる。

2026-04-29 追記 5:

`ISS-20260428T200446882Z-RESOURCE-CELLSTATE-MERGE-STARTS-FROM-C81C0269` として、`CellTable::merge_paths` が synthetic `Uninit` を畳み込み初期値にしていたため、全 path で `Initialized(T)` のままの local / raw cell まで `MaybeMoved` になる問題を修正した。実 path の最初の `availability_state` から fold するようにし、片方の path にしか存在しない place は従来どおり `Uninit` と合流して `MaybeMoved` になる。

一時 `RawMemoryLoadCell` gate では `move_effect.n.md` が 105/110 から 106/110 に改善し、loop が raw place を触っていない `#80` の false D3100 は解消した。残件は 4 件で、realloc 後の raw slot transfer、`MemPtr` / `RegionToken` wrapper address summary、literal helper address summary に絞られる。

2026-04-29 追記 6:

`ISS-20260428T201631358Z-RESOURCE-CELLSTATE-RAW-CELLS-DO-NOT--72A5D076` として、raw address alias の canonical key が temporary から local へ変わったときに CellTable の raw cell entry が旧 key のまま残る問題を修正した。`realloc_raw` の output temporary へ transfer された `tmp.deref` は、`let grown = tmp` により canonical が `grown` へ変わるため、以後の `load_i32 grown` が `grown.deref` を探して false D3100 になっていた。

`ResourceCheckEngine` の alias transfer を `copy_raw_alias_and_rekey_cells` へ統一し、`CellTable::rekey_raw_cells` で raw cell state を旧 canonical から新 canonical へ移すようにした。一時 `RawMemoryLoadCell` gate では `move_effect.n.md` が 106/110 から 107/110 に改善し、realloc slot 系 `#8` は本来の D3025 へ戻った。残件は 3 件で、`MemPtr` / `RegionToken` wrapper address summary と literal helper address summary に絞られる。

2026-04-29 追記 7:

`ISS-20260428T202704426Z-RESOURCE-IR-LOWERING-DOES-NOT-EXPOSE-0104A160` として、`MemPtr` / `RegionToken` wrapper helper が opaque call として下がり、`MemPtr.raw` / `RegionToken.ptr.raw` の structural alias が CellState に渡らない問題を修正した。`ResourceOp::RawAddressAlias` を追加し、call/effect coverage count を変えずに raw address alias だけを Resource IR に表現する。

一時 `RawMemoryLoadCell` gate では `move_effect.n.md` が 107/110 から 109/110 に改善し、`doctest#23` の `mem_ptr_add` literal disjoint offset と `doctest#38` の RegionToken load-then-dealloc 前 load は通るようになった。残件 `doctest#30` は `ISS-20260428T203931325Z-RESOURCE-IR-RAW-ADDRESS-SUMMARIES-DO-C7473DEA` として、literal arithmetic helper return summary の問題に分離した。

2026-04-29 追記 8:

`ISS-20260428T203931325Z-RESOURCE-IR-RAW-ADDRESS-SUMMARIES-DO-C7473DEA` として、literal argument で確定する raw address helper return と unknown offset overlap の問題を修正した。lowering は `slot_ptr(base, 0)` のような user helper return を call-site で `base + 0` として特殊化し、`ResourceOp::RawAddressAlias` に落とす。literal で確定しない offset は `StorageOffset(None)` として保持し、CellState は unknown offset を offset なしの base raw cell とも重なる prefix として扱う。

一時 `RawMemoryLoadCell` gate では `move_effect.n.md` が 109/110 から 110/110 に改善し、既知の false D3100 は解消した。親 issue はまだ compiler gate 常時有効化を含んでいないため open のままとし、次の確認では `move_check.n.md` など関連範囲を含めて `RawMemoryLoadCell` を正式 gate に入れられるか判断する。
