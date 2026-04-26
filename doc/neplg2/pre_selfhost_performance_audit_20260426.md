# NEPLg2.0 pre-self-host performance audit 2026-04-26

最終更新: 2026-04-26

---

## 目的

[pre_selfhost_audit_20260426.md](./pre_selfhost_audit_20260426.md) の追加監査として、計算量、メモリ使用量、断片化、copy cost に関係する問題を Issue 管理へ追加する。
この文書は self-host 実装開始前に、性能劣化を workaround で隠さないための判断材料である。

---

## 追加 Issue

| Issue | 優先度 | 対象 | 性能リスク |
|---|---|---|---|
| [ISS-20260426T021000000Z-HASHCOLLECTION-REHASH-8A1D4C6F](../../issues/items/ISS-20260426T021000000Z-HASHCOLLECTION-REHASH-8A1D4C6F.md) | P1 | `HashMap` / `HashSet` | 固定容量 16、rehash なし、probe 悪化 |
| [ISS-20260426T021001000Z-BTREE-ARRAY-COST-B37E2A91](../../issues/items/ISS-20260426T021001000Z-BTREE-ARRAY-COST-B37E2A91.md) | P2 | `BTreeMap` / `BTreeSet` | 実体が sorted array で更新 O(n) |
| [ISS-20260426T021002000Z-ALLOCATOR-FRAGMENTATION-D0E7A4C3](../../issues/items/ISS-20260426T021002000Z-ALLOCATOR-FRAGMENTATION-D0E7A4C3.md) | P1 | `stdlib/core/mem.nepl` | free block coalescing なし、fragmentation と O(n) scan |
| [ISS-20260426T021003000Z-MEM-BULK-COPY-41F6B8D2](../../issues/items/ISS-20260426T021003000Z-MEM-BULK-COPY-41F6B8D2.md) | P2 | `mem`, `io`, `string` | byte-by-byte copy が hot path に残る |
| [ISS-20260426T021004000Z-IMPORT-VISIBILITY-CLONE-6F92C1A0](../../issues/items/ISS-20260426T021004000Z-IMPORT-VISIBILITY-CLONE-6F92C1A0.md) | P2 | `nepl-core/src/typecheck.rs` | import visibility closure が全体 clone loop |
| [ISS-20260426T021005000Z-MONOMORPHIZE-TRAIT-LOOKUP-93E4A8B5](../../issues/items/ISS-20260426T021005000Z-MONOMORPHIZE-TRAIT-LOOKUP-93E4A8B5.md) | P2 | `nepl-core/src/monomorphize.rs` | trait impl 解決が線形走査へ fallback |

---

## 既存 Issue との関係

| 既存 Issue | 判断 |
|---|---|
| [ISS-20260425T000000Z-RV-CORE-003-CE8DD508](../../issues/items/ISS-20260425T000000Z-RV-CORE-003-CE8DD508.md) | `reduce_calls` の O(n^2) は verified 済み。今回の追加対象ではない |
| [ISS-20260425T000000Z-RV-CORE-004-77FA051C](../../issues/items/ISS-20260425T000000Z-RV-CORE-004-77FA051C.md) | overload 解決の `TypeCtx` clone は verified 済み。今回の import visibility clone とは別箇所 |
| [ISS-20260425T000000Z-RV-CORE-015-1D54B715](../../issues/items/ISS-20260425T000000Z-RV-CORE-015-1D54B715.md) / [ISS-20260425T000000Z-RV-CORE-016-69B16F10](../../issues/items/ISS-20260425T000000Z-RV-CORE-016-69B16F10.md) | 深い HIR traversal の stack overflow は verified 済み。今回の問題は残る allocation / lookup cost |
| [ISS-20260425T000000Z-RV-STDLIB-009-01749CCF](../../issues/items/ISS-20260425T000000Z-RV-STDLIB-009-01749CCF.md) | 巨大 stdlib file 分割は open。分割後に import visibility clone cost が増える可能性がある |
| [ISS-20260426T010005Z-BYTEBUF-BUILDER-WASM-D63A7C11](../../issues/items/ISS-20260426T010005Z-BYTEBUF-BUILDER-WASM-D63A7C11.md) | ByteBuilder API 不足は open。bulk copy 不足は builder 実装後も残る低レベル性能問題 |

---

## self-host 開始条件への反映

S0 の source tree scaffold は今回の性能 Issue を未解決のまま開始できる。
ただし、次の stage では先に方針を固定する。

1. S1 lexer / parser は token table と interning の構造を決める前に、`HashMap` / `HashSet` の grow / rehash 実装を確認する。`ISS-20260426T021000000Z-HASHCOLLECTION-REHASH-8A1D4C6F` は 2026-04-26 に verified 済み。
2. S2 module loader は stdlib 分割と import visibility closure の cost を同時に見る。`ISS-20260426T021004000Z-IMPORT-VISIBILITY-CLONE-6F92C1A0` は 2026-04-26 に worklist 化で verified 済み。
3. S3 typecheck は mutable large table に `BTreeMap` / `BTreeSet` 互換実装を使わない。これらは sorted-array 実装であるため、小さな ordered table では `sorted_array_map_*` / `sorted_array_set_*` alias を使い、安定順序が必要な大きい出力は `HashMap` / `HashSet` と最終段階の key 整列に分ける。`ISS-20260426T021001000Z-BTREE-ARRAY-COST-B37E2A91` は 2026-04-26 に用途契約と focused fixture を verified 済み。
4. S4 monomorphize は trait helper と generic collection の利用が増える前に trait impl lookup の候補表を index 化しておく。`ISS-20260426T021005000Z-MONOMORPHIZE-TRAIT-LOOKUP-93E4A8B5` は 2026-04-26 に `(trait_name, method)` index と lookup cache で verified 済み。
5. S5 WASM emitter は ByteBuilder と同時に bulk copy API の必要性を判断する。
6. 長時間実行の CLI parity test を追加する前に、allocator fragmentation の stress fixture を確認する。`ISS-20260426T021002000Z-ALLOCATOR-FRAGMENTATION-D0E7A4C3` は 2026-04-26 に verified 済み。

---

## 検証メモ

今回の性能監査で追加した `HashMap` / `HashSet` 固定容量問題は、2026-04-26 に grow / rehash と `with_capacity` 追加で解消した。
`BTreeMap` / `BTreeSet` 互換実装の sorted-array 更新コストは、2026-04-26 に `sorted_array_map_*` / `sorted_array_set_*` alias、self-host S3 の利用禁止ルール、`tests/stdlib/btree_array_cost.n.md` の 32 / 128 件 focused fixture で明示した。
Allocator fragmentation 問題も 2026-04-26 に address-order free list と coalescing で解消した。
Import visibility closure の全体 clone loop は、2026-04-26 に `All` edge だけを伝播する worklist へ置き換えた。
Monomorphize の trait impl 解決は、2026-04-26 に `(trait_name, method)` index と正規化済み lookup cache へ置き換え、generic trait impl の dispatch fixture で確認した。
検証は `issues` tool、Markdown link、既存共通 test に加え、`tests/stdlib/hash_collection_rehash.n.md`、`tests/stdlib/btree_array_cost.n.md`、`tests/stdlib/allocator_coalesce.n.md` の focused doctest で行う。
