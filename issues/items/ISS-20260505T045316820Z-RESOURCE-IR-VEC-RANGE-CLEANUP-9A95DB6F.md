---
id: ISS-20260505T045316820Z-RESOURCE-IR-VEC-RANGE-CLEANUP-9A95DB6F
title: "Resource IR が Vec の range cleanup 完了を証明できない"
area: core
status: open
resolved: false
priority: P1
type: bug
created: 2026-05-05
updated: 2026-05-05
target: "nepl-core/src/resource/**,nepl-core/src/passes/move_check/**,stdlib/alloc/collections/vec.nepl"
---

# ISS-20260505T045316820Z-RESOURCE-IR-VEC-RANGE-CLEANUP-9A95DB6F: Resource IR が Vec の range cleanup 完了を証明できない

## 概要

Vec<NonCopy> の element cleanup を callback loop で行ってから storage dealloc する実装を試すと、各要素が move/drop 済みである事実を range として Resource IR / 旧 move checker が表現できず、dealloc 時に resource.cell.initialized_conflict になる。

## 対象

- `nepl-core/src/resource/**,nepl-core/src/passes/move_check/**,stdlib/alloc/collections/vec.nepl`

## 根拠

- `Vec<DropCounter>` の live 要素を callback loop で 1 個ずつ cleanup し、その後 `dealloc_raw` で storage を解放する実装を `stdlib/alloc/collections/vec.nepl` に試験投入した。
- 同実装を `tests/stdlib/vec_collections.n.md` の positive doctest で確認したところ、`dealloc_raw` 時に `resource.cell.initialized_conflict` が発生した。要素側は loop 内で cleanup callback に渡しているが、checker 側は `[0, len)` の initialized cell がすべて moved/dropped 済みである事実を range summary として保持できていない。
- stdlib 側で関数名を変えるだけ、または shallow `free` に戻すだけでは leak / double-drop の根本原因を隠すため、core Resource IR の表現力不足として扱う。

## 問題

Vec<NonCopy> の element cleanup を callback loop で行ってから storage dealloc する実装を試すと、各要素が move/drop 済みである事実を range として Resource IR / 旧 move checker が表現できず、dealloc 時に resource.cell.initialized_conflict になる。

## 影響

collection-wide owning payload cleanup を stdlib 側だけで正しく提供できず、Copy fast path 以外を許すと leak/double-drop/不正 shallow dealloc の逃げ道になる。Stage 6 collection cleanup と NEPLg2 selfhost の安全な owned storage 実装をブロックする。

## 修正方針

MemPtr の address と storage owner を分けたうえで、InitializedCell/Resource IR に initialized range の move/drop 完了 summary を持たせる。callback 名や stdlib 関数名の特別扱いではなく、owned storage cleanup operation と dealloc obligation の接続を型付き IR と match 網羅性で検査する。

## 検証

Vec<DropCounter> の全要素 cleanup 後に storage dealloc でき、drop counter が要素数と一致する positive test を追加する。free<DropCounter> / clear<DropCounter> の shallow fast path は引き続き compile_fail になることを確認する。

## 関連 issue

- `ISS-20260425T000000Z-RV-STDLIB-004-91534828`: Stage 6 collection API / element cleanup の親 issue。stdlib 側では Copy fast path の境界を先に固定し、この issue の Resource IR 修正後に owned storage cleanup を実装する。

## 2026-05-05 exact raw-load summary 部分進捗

調査中に、range cleanup 以前の exact raw cell move も function summary へ伝播していないことを確認したため、`ISS-20260505T061748334Z-RESOURCE-IR-SUMMARIES-OMIT-EXACT-NON-C8D2F16E` として分離して修正した。

- 修正済み: callee が parameter-derived exact raw address から non-Copy 値を `load<T>` した場合、caller 側の `CellTable` でも該当 raw cell を moved に更新する。
- 修正済み: caller 側で該当 raw cell が未初期化なら、summary 適用時に `RawMemoryLoadCell` として拒否する。
- 未解決: `data + i * size_of<T>` のような unknown/dynamic offset を loop で全 range cleanup した事実は、まだ Resource IR 上で証明できない。unknown offset を exact move summary に混ぜると「1 要素の動的 load」を「全 range moved」と誤認し得るため、この issue では保守的に残す。

## 2026-05-05 unknown-offset raw-load precondition 部分進捗

exact offset 修正後の追加監査で、parameter-derived unknown offset address からの non-Copy raw load が function summary から完全に落ち、caller 側で未証明 cell を読む false negative を確認した。これは range cleanup 完了の証明ではなく caller-side availability precondition の伝播漏れなので、`ISS-20260505T063836309Z-RESOURCE-IR-SUMMARIES-OMIT-UNKNOWN-O-4B002380` として分離して修正した。

- 修正済み: unknown offset の non-Copy raw load は、range cleanup 完了とは扱わず `RawMemoryLoadCell` precondition として caller へ伝播する。
- 修正済み: caller が該当 unknown cell の initialized 状態を証明できない場合、`CellUnavailable` として拒否する。
- 未解決: loop/callback により `[0, len)` の全要素が moved/dropped 済みであることを表す range summary は未実装のままで、この issue の本体として残す。
