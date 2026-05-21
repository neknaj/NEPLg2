# stdlib collection/mem/string と静的検査の安全設計

作成日: 2026-04-30

## 目的

`stdlib/core/mem.nepl`、`stdlib/alloc/string.nepl`、`stdlib/alloc/io.nepl`、`stdlib/alloc/collections/**` と Rust compiler 側 Resource IR の関係を整理し、self-host に向けて必要な memory safety / type safety の設計を定める。

この文書は「現状の実装を通すために静的検査を弱める」ためのものではない。メモリ安全と型安全は必達であり、stdlib 側の型表現と Resource IR 側の検査が同じ不変条件を共有できる設計へ移行する。

## 前提

- 後方互換は安全性より優先しない。
- `MemPtr<T>` は non-owning pointer / projection として扱い、storage owner として拡張し続けない。
- storage owner、initialized cell、borrow projection、drop/free obligation は別概念として扱う。
- 状態は数値や null pointer ではなく enum / Option / typed wrapper で表し、分岐は `match` による網羅性検査が効く形にする。
- Resource IR の owner / cell / borrow gate は弱めない。既存 stdlib が拒否される場合は、stdlib の所有権契約か Resource IR lowering のどちらが不十分かを切り分ける。

## 進捗状況

| 領域 | 現状 | 判定 | 次の作業 |
|---|---|---|---|
| `core/mem` | `MemPtr<T>` は non-owning projection、`RegionToken<T>` は free obligation owner として分離済み。public / direct import 可能な `alloc_ptr` / `realloc_ptr` / `dealloc_ptr` は削除済みで、safe caller の標準経路は `alloc_region` / `region_ptr` / `dealloc_region` である。 | 改善済みの過渡。`MemPtr` owner API は閉じたが、`RegionToken<T>` はまだ compiler-issued owner token ではなく、initialized payload destruction と storage-only free の最終分離も残る。 | `RegionToken` を過渡 owner token として閉じ、compiler-issued `OwnedRegion` / `OwnedBuffer` / initialized cell state へ移行する。 |
| `alloc/io` `ByteBuf` | `io/bytebuf` が `ByteBuf`、RegionToken 経由の確定境界、str との checked conversion を所有する。`ByteBuf.storage: ByteBufStorage` が `Empty | Owned(RegionToken<u8>)` として空 storage と free obligation owner を構造的に分け、`MemPtr<u8>` は参照から得る非所有 view に限定した。 | 良い方向。短期 self-host の binary I/O buffer として使用可能。root facade に raw capability を戻さない。empty storage sentinel は削除済み。 | `OwnedBytes` / compiler-issued owner token 設計が入った後に forgeable `RegionToken` から移行する。 |
| `alloc/io` `ByteBuilder` | `io/bytebuilder` が grow/reserve/append/finish を所有する。`ByteBuilder.storage: ByteBuilderStorage` が `Empty | Owned(RegionToken<u8>)` として空 storage と free obligation owner を構造的に分け、append / finish / free は `match` の網羅性で empty branch と owned branch を分ける。reserve / append / finish failure は `ByteBuilderError` に builder owner を戻し、`byte_builder_push_bytebuf` は `ByteBuilderByteBufError` に builder と input `ByteBuf` の両 owner を戻す。safe API は pure surface とし、raw memory effect は raw-memory-boundary source 内で Resource IR が検査する。 | 良い方向。builder owner leak、bare error failure contract、zero-size sentinel 回帰は source policy と doctest で監視済み。raw capability は `io/bytebuilder` に限定した。 | 将来の compiler-issued `OwnedBytes` / owner wrapper に合わせる。 |
| `alloc/string` `str` | `string_alloc_region` / `string_finish` で `RegionToken` を使う経路が増えた。UTF-8 と char API も進み、numeric conversion は `integer/common` / `integer/format` / `integer/parse` / `float/format` / `float/parse` へ分割済み。`string_addr` / `string_from_addr_unchecked` は `storage.nepl` 内の private helper に閉じ、`string_finish_base` は削除済み。`access.nepl` / `scanner.nepl` の `str_addr` helper と scanner unchecked byte reader も private にした。`string_byte_at_unchecked` は `access.nepl` と root `alloc/string` facade から外し、過渡的な `unchecked_access.nepl` も削除した。現在の low-level byte reader は `byte_index.nepl` の private `StringByteIndex` witness を要求する。 | 改善済みの過渡。`str` 確定境界は `RegionToken<u8>` を消費する `string_finish` に集約し、通常 byte access は checked `byte_at` に戻した。unchecked public raw reader はなくなり、hot path 用 helper も checked factory を必ず通る。ただし `RegionToken` 自体はまだ compiler-issued owner token ではない。 | `str` 生成 API を `OwnedStringRegion` に寄せ、forgeable `RegionToken` から移行する。byte index witness の方針を維持し、raw layout authority を public surface へ戻さない。 |
| `alloc/string` `StringBuilder` | `bytes: ByteBuilder` を保持する typed text builder wrapper。append は `ByteBuilder` の byte owner boundary へ委譲し、build は `ByteBuilder -> ByteBuf -> str` の確定経路を通す。 | 良い方向。StringBuilder 固有の raw `MemPtr` owner field はなくなった。短期 self-host の text builder として使用可能。 | `ByteBuilder` / `ByteBuf` 側を将来の `OwnedBytes` / `OwnedStringRegion` へ移行する。 |
| `alloc/collections/Vec` | `Vec<T>` は public facade として `buffer: OwnedBuffer<T>` だけを保持する。`OwnedBuffer<T>` が `len/initialized_len/cap/storage` を持ち、`VecStorage<T>::Empty/Owned(RegionToken<T>)` で storage state と free obligation owner を同じ enum に束ねる。`len` は public API の live length、`initialized_len` は drop / move state の対象になる initialized prefix である。現行 Copy-only API では両者を同じ値に保つが、field は分離済みなので後続の moved slot / drop traversal が接続できる。型定義は `vec/types`、storage allocation/free は `vec/storage`、borrowed observer は `vec/access`、owner-consuming transform は `vec/transform`、borrowed query は `vec/query`、mutation/cleanup は `vec/mutation` が所有する。raw load/store は `vec/raw` のような直接 import 可能な helper module に置かず、len・initialized_len・storage variant・capacity を検査する操作の同じ source file 内に閉じる。allocation constructor、typed data view observer、raw element operation、`push` / `pop` / `sort` / `clear` / `free` / `vec_free_storage` は、raw storage identity / raw copy / storage-only cleanup / raw swap に依存するため `.T: Copy` に限定済み。`data_mem_view<T>(&Vec<T>) -> VecDataView<T>` は `Empty | Data(MemPtr<T>) | Invalid(VecCopyInvariantInvalid)` を返し、`MemPtr<T>` は `VecDataView::Data` payload としてだけ raw boundary に渡す。 | 改善済みの過渡。collection facade と backing storage owner の責務分離は入った。`MemPtr` owner field、split `RegionToken<T>` field、`Vec<T>` 直下の `len/initialized_len/cap/storage` は消え、`Empty` と allocated token の相関は型で表せる。public `vec/raw` bypass も削除済み。ただし `RegionToken<T>` が forgeable である点と、non-Copy payload collection の owner-preserving update / move-out / drop traversal は moved slot / initialized cell state まで残る。 | `OwnedBuffer<T>` の `initialized_len` に moved slot / drop traversal を接続し、read/copy/move/drop/free を分離する。 |
| `Stack` / `Queue` / `Deque` / `RingBuffer` / `BinaryHeap` | raw header は廃止済みで、`len/cap/head/items: Vec<Option<T>>` 系へ移行済み。live/inactive slot は `Some` / `None` で表す。`Stack` / `Queue` / `Deque` / `RingBuffer` / `BinaryHeap` の pop は、更新後 owner と item を同時に返す owner-preserving result を持つ。`Deque.push_front` / `push_back` と `BinaryHeap.push` の grow failure も専用 error payload に元 owner と `Diag` を入れて返す。 | 良い方向。ただし現行 `Vec` は `OwnedBuffer<T>` facade へ移った段階で、compiler-issued owner token と initialized prefix / moved slot / drop traversal は未完である。 | `OwnedBuffer<T>` の initialized cell state 実装後、`Vec<Option<T>>` 依存を新 buffer model に移す。 |
| `HashMap` / `HashSet` | `BucketState` enum と typed bucket storage を導入済み。key/value/key-only storage は `Vec<Option<...>>` で初期化状態を表す。 | 良い方向。Copy payload 前提では Resource IR が storage owner と initialized slot を追いやすい。 | source policy で raw header / numeric sentinel への退行を防ぎ、非 Copy payload は別設計で扱う。 |
| `BTreeMap` / `BTreeSet` | sorted-array typed storage を使い、key/value slot は `Vec<Option<T>>` で表す。`key_eq` は by-value `ord_lt` を 2 回呼ぶため `.K: Ord&Copy` / `.T: Ord&Copy` に限定済み。`insert` / grow failure は `BTreeMapInsertError` / `BTreeSetInsertError` に元 owner と `Diag` を入れて返す。 | 良い方向。raw header はなく、non-Copy key の二重消費入口と grow failure で owner を隠す入口を閉じた。ただし名称通りの木構造ではなく小規模 ordered table である。 | borrowed comparison と non-Copy key/value の owner-preserving update は `OwnedBuffer<T>` と initialized cell state が入った後に別 API として設計する。 |
| bitset 系 collection | `BitSet` / `AdjacencyMatrix` / BloomFilter / CountingBloomFilter は `Vec<u8>` storage へ移行済み。 | 良い方向。byte collection 固有の raw owner field は解消済み。 | `Vec` の基礎 owner model を `OwnedBuffer` へ移行し、byte collection もそれに追従する。 |
| `List` | raw node chain を廃止し、`items: Vec<T>` storage へ移行済み。論理先頭は `Vec` 末尾で、`tail` は `Vec.pop` により owner を返す。`cons` / `push` の grow failure は `ListPushError<T>`、`map` / `filter` の transform failure は `ListTransformError<T>` に入力 owner を戻す。 | 良い方向。raw node owner と bare `Diag` failure に owner を隠す入口は解消したが、Copy payload 前提と `Vec` 基礎 owner model の制約は残る。 | `OwnedBuffer<T>` 化後に borrowed observer と non-Copy payload contract を追加する。 |
| Rust Resource IR | `CellState` / `OwnerState` / `BorrowState` / `StorageOrigin` があり、raw memory op、aggregate projection、enum payload、branch merge を追跡している。 | 方向は正しい。現状は stdlib の曖昧な所有権表現を補うための alias 処理が多い。 | stdlib 側の owner wrapper 化に合わせ、特例的 alias summary を減らす。 |
| self-host stdlib 利用 | ByteBuf / builders / Copy-only Vec / typed slot collection は使えるが、non-Copy payload collection や fallible update は危険。 | 制限付き開始は可能。ただし ResourceIR / typecheck 実装で raw collection discipline を増やしてはいけない。 | token stream / diagnostics / symbol table は安全 subset か専用 typed collection を使う。 |

## 2026-05-20 Vec raw access invariant 追記

`Vec<T>` の current Copy-only raw element access は、`OwnedBuffer<T>` の metadata を個別に読むだけでは不十分である。raw load/store、range copy、sort traversal、grow path の直前には、少なくとも次の相関を 1 つの invariant として確認する。

- `0 <= len == initialized_len <= cap`
- `VecStorage::Empty` なら `len == 0` かつ `cap == 0`
- `VecStorage::Owned(region)` なら `cap > 0` かつ `region_size(region) == cap * size_of<T>`

このため `stdlib/alloc/collections/vec/invariant.nepl` を追加し、`vec_buffer_current_copy_invariant<T>` / `vec_current_copy_invariant<T>` を raw access boundary の共通入口にした。`get` / `replace` / `push` / `pop` / transform / sort family は raw data view の導出や raw load/store より前にこの helper を通す。これにより、Copy-only 期間の `Vec` は「各 API が len/cap を読む」状態ではなく、「raw memory operation の前提となる `OwnedBuffer<T>` 全体の相関を検査する」状態になる。

これは final non-Copy collection design ではない。`initialized_len` が moved slot / drop traversal / compiler-issued owner token と接続されるまでは、raw element operation は `.T: Copy` に限定する。今回の invariant helper は、その後続段階で `InitializedCell` / Resource IR が接続される位置を明示するための過渡境界である。

2026-05-20 追記: aggregate / predicate query も同じ invariant boundary に含める。`count` / `fold` / `reduce` / `find` / `any` / `all` は直接 raw load しない場合でも、`len` を loop bound として使うため traversal decision の前に invariant を確認する。invalid owner aggregate は empty Vec と同一視せず、走査に入らない neutral non-success result へ落とす。これにより、raw access だけでなく計算量と制御フローも invalid metadata へ依存しない。

2026-05-20 追記 2: typed raw data view observer である `data_mem_view<T>(&Vec<T>)` も invariant boundary に含める。raw load/store caller が guard を持つだけでは不十分で、`RegionToken<T>` から non-owning raw view を導出する public API 自体が `OwnedBuffer<T>` invariant を確認する必要がある。結果は `VecDataView<T>::Empty | Data(MemPtr<T>) | Invalid(VecCopyInvariantInvalid)` とし、invalid owner aggregate は actual storage view や null pointer sentinel に潰さず typed refutation として返す。

2026-05-20 追記 3: `Vec` invariant の結果は bool ではなく `VecCopyInvariant::Valid | Invalid(VecCopyInvariantInvalid)` として扱う。`len` / `initialized_len` / `cap` / `VecStorage` の相関が壊れた理由を enum payload に残し、raw boundary caller は `match` で `Valid` と `Invalid` を分岐する。これにより、future invariant case を追加した時に source policy と compiler の網羅性検査が効き、静的検査プログラム自体の誤りを見つけやすくする。raw access は引き続き `.T: Copy` の過渡境界に限定し、non-Copy payload の move/drop state は `InitializedCell` / Resource IR 接続後に扱う。

2026-05-20 追記 4: `VecStorage::Owned(region)` の invariant は tag と `cap > 0` だけでは不十分である。raw offset は `idx * size_of<T>` で計算されるため、`cap` metadata と backing `RegionToken` の byte extent が一致していることも同じ proof に含める。`vec_buffer_current_copy_invariant<T>` は `size_of<T>` と allocator payload 上限で `cap * size_of<T>` を checked に計算し、`region_size(region)` と一致しない場合は `OwnedStorageExtentMismatch` として拒否する。これは `RegionToken` を信用するのではなく、source object の capacity と owner extent の相関を typed invariant として証明する境界である。

2026-05-20 追記 5: `data_mem_ptr<T>(&Vec<T>) -> MemPtr<T>` は互換 alias を残さず削除した。`MemPtr<T>` を直接返す API は、valid empty storage と invalid owner aggregate を同じ null pointer sentinel に畳み、proof/refutation evidence を caller の `match` から隠すためである。raw boundary caller は `data_mem_view<T>` を `match` し、`VecDataView::Data(data)` branch の中だけで `mem_ptr_addr` / `load` / `store` へ進む。`VecDataView::Empty` は valid empty state、`VecDataView::Invalid(reason)` は malformed `OwnedBuffer<T>` として扱い、両者を値で同一視しない。

2026-05-20 追記 6: `region_new` は `RegionToken<T>` owner token construction として、`mem_ptr_wrap` の generic raw-address alias boundary から分離した。Rust compiler 側では `OwnerTokenConstructBoundary` と `RawAddressAliasKind::OwnerTokenConstruct` を追加し、`region_new` lowering は owner-token construction 専用の effect boundary proof を要求する。これにより `MemPtr<T>` の non-owning projection と `RegionToken<T>` の free obligation owner issuance を同じ source capability に混ぜない。これはまだ compiler-issued `OwnedRegion<T>` の完成ではないが、non-Copy collection payload support に必要な owner token / initialized cell / drop traversal proof を generic Resource IR 境界へ載せるための前提である。関連 issue は [ISS-20260520T153639309Z-REGIONTOKEN-CONSTRUCTION-SHARES-GENE-C5BF72D0](../../issues/items/ISS-20260520T153639309Z-REGIONTOKEN-CONSTRUCTION-SHARES-GENE-C5BF72D0.md)。

2026-05-21 追記: non-Copy payload collection は raw `mem_move` の意味を ownership move に読み替えて実装しない。`core/mem` の `mem_copy<T>` / `mem_move<T>` は byte copy / memmove であり、public typed API は引き続き `.T: Copy` を要求する。non-Copy payload の安全な移動は、collection slot の `InitializeEmpty`、`BorrowRead`、`MoveOut`、`ReplaceInitialized`、`DropInitialized`、`StorageDealloc` を typed lifecycle event として Resource IR 側へ載せる。これにより、slot の live / moved / dropped / released state を stdlib module 名の allowlist ではなく、compiler-core の enum / match による汎用 proof boundary で検査する。関連 issue は [ISS-20260520T183033547Z-NON-COPY-COLLECTION-PAYLOADS-NEED-TY-674BA21D](../../issues/items/ISS-20260520T183033547Z-NON-COPY-COLLECTION-PAYLOADS-NEED-TY-674BA21D.md)。

2026-05-21 追記 2: typed lifecycle event は単体の状態遷移だけでなく、Resource IR の `Place` ごとの `CollectionSlotStateTable` として保持する。storage release は配下の live initialized slot を拒否し、moved / dropped slot だけを released storage として閉じる。これにより、Vec/Stack/Queue などの個別 stdlib module ごとに proof table を作らず、compiler-core の同じ slot state table を再利用する。関連 issue は [ISS-20260520T184420541Z-COLLECTION-SLOT-LIFECYCLE-NEEDS-PLAC-2B2D025C](../../issues/items/ISS-20260520T184420541Z-COLLECTION-SLOT-LIFECYCLE-NEEDS-PLAC-2B2D025C.md)。

2026-05-21 追記 3: collection slot state は制御フロー合流時の不確実性を `MaybeInitialized` / `MaybeReleased` として保持する。片方の分岐だけで move/drop/release された slot を definite な `Moved` や `Released` に潰すと、後続の move/drop/storage release が安全であるかを証明できなくなるため、合流後は typed refutation により reinit / move / drop / release を拒否する。関連 issue は [ISS-20260520T190336025Z-COLLECTION-SLOT-STATE-LACKS-PATH-MER-3E8FEBA9](../../issues/items/ISS-20260520T190336025Z-COLLECTION-SLOT-STATE-LACKS-PATH-MER-3E8FEBA9.md)。

2026-05-21 追記 4: collection storage の grow / realloc / owner replacement は、Vec 固有の proof や stdlib 関数名 allowlist では扱わない。Rust compiler 側では `CollectionSlotLifecyclePrimitive::StorageRelocate` と `ResourceOp::CollectionStorageRelocate` を追加し、old storage prefix 配下の initialized / moved / dropped slot state を new storage prefix へ rekey する。new storage に既存 slot state がある場合は typed refutation とし、old storage は relocation 後に released として閉じる。call summary も `Relocate { old_storage, new_storage }` を parameter-relative target として保持し、caller actual storage へ replay する。これにより `OwnedBuffer<T>` / `Vec<T>` の grow path は、個別 module proof ではなく generic Resource IR operation として non-Copy payload state を保持できる。関連 issue は [ISS-20260520T214013832Z-COLLECTION-SLOT-LIFECYCLE-STATE-DOES-FA4DE5B2](../../issues/items/ISS-20260520T214013832Z-COLLECTION-SLOT-LIFECYCLE-STATE-DOES-FA4DE5B2.md)。

2026-05-21 追記 5: collection slot state は storage relocate だけでなく、Resource IR の通常の value transfer にも追従する必要がある。`RegionToken<T>` / `OwnedBuffer<T>` 相当の owner が local へ束縛される、aggregate field に入る、branch / match の結果になる、callee から return される、といった普通の所有権移動で slot state が old place に残ると、移動先 owner の storage dealloc が live non-Copy payload を見落とす。Rust compiler 側では `CollectionSlotStateTable::transfer_storage_prefix` を追加し、`Move`、`Construct`、assignment/initializer、branch/match output、call return summary に接続した。これは Vec 固有の proof ではなく、`Place` prefix replacement に基づく generic Resource IR proof である。関連 issue は [ISS-20260520T223249968Z-COLLECTION-SLOT-STATE-DOES-NOT-FOLLO-A808C521](../../issues/items/ISS-20260520T223249968Z-COLLECTION-SLOT-STATE-DOES-NOT-FOLLO-A808C521.md)。

## 2026-04-30 再レビュー結果

基準: remote main `bbaf2a5` 取り込み後。

今回の再確認では、collection 改善が進み、以前の「raw header が多い」という記述は古くなっている。一方で、`Vec` と `core/mem` の根本設計が未完であるため、最終的な memory safety / type safety 設計はまだ完了していない。

| 分類 | 現状の実装 | 安全性判定 | 理想 |
|---|---|---|---|
| hash collection | `HashMap` / `HashSet` は bucket state enum と `Vec<Option<T>>` storage。 | 良い。数値 status ではなく enum / `match` で検査できる。 | non-Copy payload 用 owner-preserving update / drop traversal を追加する。 |
| derived linear collection | `Queue` / `Deque` / `RingBuffer` / `Stack` / `BinaryHeap` は `Vec<Option<T>>` storage。主要 pop API は owner-preserving result で更新後 owner を返し、container drop と remove/pop を分ける。`Deque.push_front` / `push_back` と `BinaryHeap.push` の grow failure も owner-preserving error payload で元 owner を返す。 | 良い方向。raw header は消えたが `Vec` 依存なので基礎 storage owner は未完。 | `OwnedBuffer<T>` 上の slot-state collection へ移す。 |
| ordered table | `BTreeMap` / `BTreeSet` は sorted-array typed storage。grow failure は owner-preserving insert error で caller に元 owner を返す。 | 良い方向。名前上は BTree だが実装は ordered array table。allocation failure の owner contract は API 型に現れた。 | self-host 用には用途を小規模 ordered table と明記し、大規模 map は別設計にする。 |
| list collection | `List` は `items: Vec<T>` storage。`reverse` / `map` / `filter` / observer は raw node を使わず owner を閉じる。 | 良い方向。raw node は解消済み。Copy payload 前提と `Vec` 基礎 owner model は残る。 | `OwnedBuffer` based storage と borrowed observer / non-Copy payload contract へ追従する。 |
| byte/bit collection | `BitSet` / `AdjacencyMatrix` / BloomFilter / CountingBloomFilter は `Vec<u8>` storage。 | 良い方向。byte/bit collection の payload owner field は `Vec<u8>` に統一済み。 | `Vec` の基礎 owner model を `OwnedBuffer` へ移し、byte collection も `OwnedBytes` 相当の安全境界へ追従する。 |
| numeric array collection | SparseSet / Fenwick / SegmentTree / DisjointSet は `Vec<i32>` storage。 | 良い方向。numeric collection 固有の raw i32 storage は解消済みだが、基礎 `Vec` の owner model は未完。 | `OwnedBuffer<i32>` / typed index API へ移す。 |
| `Vec<T>` | `buffer: OwnedBuffer<T>` を持つ public facade で、`OwnedBuffer<T>` が `len/cap/storage` を保持する。`VecStorage<T>::Empty | Owned(RegionToken<T>)` が storage state と free obligation owner を同時に表す。型/storage/access/raw helper/transform/query/mutation の責務は submodule に分離済み。 | 重要残件は縮小した。facade と backing storage owner は分離済みだが、`RegionToken<T>` が forge 可能で、initialized prefix / moved slot / drop traversal はまだ型に出ていない。 | `OwnedBuffer<T>` に compiler-issued owner token と initialized cell state を接続する。 |
| `core/mem` | public `MemPtr<T>` owner API は削除済みで、raw allocator / load / store は compiler-owned raw boundary 内に閉じる方向へ進んだ。`RegionToken<T>` は `raw,size` の owner token、`MemPtr<T>` は参照から得る projection である。 | 改善済みの過渡。owner/view 混同は大きく減ったが、`RegionToken<T>` はまだ compiler-issued token ではない。 | public safe API と internal raw API の分離を維持し、compiler-issued owner token にする。 |
| `alloc/string` / `alloc/io` | `RegionToken` owner と `ByteBuilder` / `ByteBuf` の typed boundary で owner flow を改善済み。`StringBuilder` 固有 raw owner field と `ByteBuf` / `ByteBuilder` の `Option<MemPtr<u8>>` owner field は削除済み。 | 短期 self-host では使用可能。 | `OwnedBytes` / `OwnedStringRegion` へ移し、unchecked raw conversion を internal boundary に閉じる。 |

現状実装の粗い集計:

- raw memory pattern が残る collection: `Vec`（public facade は `OwnedBuffer<T>` 化済みだが、基礎 storage boundary はまだ raw memory を持つ）。
- raw memory pattern が消えた主要 collection: `HashMap`, `HashSet`, `BTreeMap`, `BTreeSet`, `Queue`, `Deque`, `RingBuffer`, `Stack`, `BinaryHeap`, `SparseSet`, `BitSet`, `Fenwick`, `SegmentTree`, `DisjointSet`, `AdjacencyMatrix`, `BloomFilter`, `CountingBloomFilter`。
- enum state が明示されている主要 collection: `HashMap`, `HashSet`。
- slot state を `Option<T>` で明示している collection: `HashMap`, `HashSet`, `BTreeMap`, `BTreeSet`, `Queue`, `Deque`, `RingBuffer`, `Stack`, `BinaryHeap`, 一部 `Vec` 利用 API。

この状態での設計判断:

1. derived collection の raw header 廃止は正しい。Resource IR が raw header 内の疑似 field を復元する必要がなくなり、所有権検査の入力が構造化された。
2. `Vec<Option<T>>` は改善だが完成形ではない。`Vec` 自体は `OwnedBuffer<T>` facade へ移ったが、最終的な static check authority は compiler-issued owner token / initialized prefix / moved slot / drop traversal まで進めて初めて確立する。
3. `RegionToken<T>` は builder / string / Vec で raw address owner を局所化する効果があるが、stdlib から forge 可能な struct である限り compiler capability ではない。self-host の memory model へそのまま移植してはいけない。
4. type safety の観点では、state を enum / Option へ出した領域は良い。逆に raw `i32` address、numeric length/state、string diagnostic code へ依存する領域は再設計対象である。
5. memory safety の観点では、allocation failure 時の owner contract が API 型に出ていない `Result<Collection<T>, E>` は non-Copy payload では不十分である。Err で collection / item owner を返す専用 enum が必要である。

## 現状の設計と実装

### `core/mem`

現状の `core/mem` は byte allocator、raw load/store、bulk copy/move、typed `MemPtr` wrapper、`RegionToken` を 1 module に持つ。

良い点:

- `mem_copy<T>` / `mem_move<T>` は `T: Copy` に制約され、non-Copy の浅い byte copy を public helper から行いにくくなっている。
- `RegionToken<T>` を使う string / ByteBuf の確定経路が増え、確定前 owner を local raw address ではなく token に置く方向へ進んでいる。
- Resource IR は raw allocation の free obligation、raw cell initialized state、borrow state を追跡し始めている。

問題:

- `MemPtr<T>` は stdlib コメント上は Copy な non-owning pointer であり、ByteBuf / ByteBuilder / StringBuilder / Vec の public owner field からは外れた。残る問題は、`RegionToken<T>` 自体が forge 可能な owner token である点である。
- `RegionToken<T>` は `region_new` で stdlib code から allocator-issued raw owner identity を束ね直せるため、compiler-issued capability ではない。
- `dealloc_region<T>` は storage-only free と initialized payload destruction を API 上で分けない。non-Copy payload collection では、`OwnedBuffer<T>` / initialized cell state / drop traversal が入るまで `.T: Copy` 境界を維持する。
- raw `i32` API と safe-ish typed API が同じ名前空間にあり、self-host 側が discipline を誤って広げやすい。

結論として、現状の `core/mem` 方針は過渡期の安全強化としては妥当だが、最終設計としては不適切である。根本修正は `MemPtr` の強化ではなく、役割分割である。

### `alloc/string`

現状の string は `[len:i32][bytes...]` layout を使い、`str` 自体は UTF-8 保証された immutable value として扱う。

改善済みの点:

- `string_alloc_region` / `string_region_data_ptr` / `string_finish` により、出力 `str` の確定前 owner を `RegionToken<u8>` に寄せている。
- `StringBuilder` は固有の `Option<MemPtr<u8>>` field を持たず、`ByteBuilder` owner を保持する wrapper になった。
- append は `ByteBuilder` の typed `push_*` / `push_str` / `push_str_slice` / `push_bytebuf` に委譲し、text builder 側では raw byte 書き込みを行わない。
- UTF-8 lead byte の分類など、一部で enum / match による分岐へ移行している。
- numeric conversion は root facade から分離済みで、integer conversion は `integer/common` が bool/digit/radix/u128 helper、`integer/format` が文字列化、`integer/parse` が解析と範囲検査を所有する形へ整理されている。float conversion も `float/format` が f64/f32 文字列化と raw output allocation、`float/parse` が小数・指数解析を所有する。

残る問題:

- `string_addr` / `string_from_addr_unchecked` は `storage.nepl` 内の private helper に閉じた。`access.nepl` / `scanner.nepl` が必要とする `str_addr` 由来 observer も各 module の private helper にし、通常 source が direct import して raw `i32` observer や raw address finalizer として使う経路は残さない。
- `string_finish_base` は `MemPtr<u8>` から `str` を作れる不要な確定 helper だったため削除した。`str` への所有権移行は `RegionToken<u8>` を消費する `string_finish` だけが担当する。
- `scanner_string_byte_at_unchecked` は scanner 内部の範囲正規化後だけに使う private helper にした。public scanner API は range helper / byte classification helper に限定する。
- `alloc/string/access` は safe public API に寄せ、`byte_at` が `len` で範囲確認した後に private raw helper を呼ぶ。旧 `string_byte_at_unchecked` は root facade から再公開せず、過渡的な `alloc/string/unchecked_access` も削除した。既存 hot path は `alloc/string/byte_index` の private `StringByteIndex` witness を使う checked helper へ移行済みで、任意 `i32` が raw read へ直接届かない。
- `StringBuilder` 固有の raw owner field は解消済みである。残る理想形は `ByteBuilder` / `ByteBuf` 側を `OwnedBytes` / `OwnedRegion<u8>` へ移すことである。

### `alloc/io`

`ByteBuf` と `ByteBuilder` は現時点で stdlib memory model の先行実装になっている。module 責務は `alloc/io` root facade、`io/bytebuf`、`io/bytebuilder`、`io/traits` に分離済みで、raw memory capability は `io/bytebuf` と `io/bytebuilder` の exact path に限定する。

良い点:

- `ByteBuf` / `ByteBuilder` の free obligation は `RegionToken<u8>` field に集約され、`MemPtr<u8>` は参照から得る non-owning view としてだけ扱う。
- 空 storage は `ByteBufStorage::Empty` / `ByteBuilderStorage::Empty` で表し、zero-size sentinel `RegionToken` は持たない。所有 storage だけが `Owned(RegionToken<u8>)` payload を持つ。
- `ByteBuf` の free / to-str 変換は `RegionToken` owner を centralized cleanup へ渡す。
- `ByteBuilder` は append 成功時に payload pointer を取り出して包み直さず、owner field 全体を移す。
- `ByteBuilder` は reserve / append / finish 失敗時にも internal free で owner transfer を隠さず、`ByteBuilderError` / `ByteBuilderByteBufError` へ owner を戻す。
- stream trait 群は `io/traits` に分離され、raw memory operation を持たない抽象境界として監視される。

残る問題:

- `RegionToken<T>` はまだ stdlib code から `region_new` で allocator-issued raw owner identity を束ねて構築できるため、compiler-issued capability ではない。
- empty / allocated と capacity / initialized prefix の一部は `ByteBuf` / `ByteBuilder` の struct discipline に残る。

### `alloc/collections`

collections は self-host に必要な基礎構造だが、現状は安全設計として最も再設計が必要な領域である。

現状:

- `HashMap` / `HashSet` は `Empty` / `Full` / `Tombstone` の enum と typed storage へ移行済みで、raw bucket status に戻さない source policy を持つ。
- `Queue` / `Deque` / `RingBuffer` / `Stack` / `BinaryHeap` は raw header を廃止し、`Vec<Option<T>>` storage へ移行済みである。live slot と inactive slot は `Some` / `None` で表す。`Stack` / `Queue` / `Deque` / `RingBuffer` / `BinaryHeap` の pop は更新後 owner と item を同時に返し、`Deque.push_front` / `push_back` と `BinaryHeap.push` の grow failure は元 owner と diagnostic を専用 error payload で返す。
- `BTreeMap` / `BTreeSet` は sorted-array 形式の typed `Vec<Option<T>>` storage へ移行済みであり、raw key/value pointer layout ではない。
- `List` は raw node chain を廃止し、`items: Vec<T>` storage へ移行済みである。論理先頭を `Vec` 末尾に置くことで先頭追加と `tail` を owner-preserving に実装している。`map` / `filter` の failure も `ListTransformError<T>` に入力 owner を戻すため、fallible transform が `Diag` だけへ owner を隠す入口は閉じた。
- CountingBloomFilter / BitSet / AdjacencyMatrix / BloomFilter / SparseSet / Fenwick / SegmentTree / DisjointSet は `Vec<u8>` / `Vec<i32>` storage へ移行済みである。payload は主に Copy だが、基礎 `Vec` 自体はまだ raw memory backed storage boundary を持つ。
- `Vec<T>` は `buffer: OwnedBuffer<T>` だけを持ち、`OwnedBuffer<T>` が `len/initialized_len/cap/storage` を束ねる。空 Vec は `VecStorage<T>::Empty`、allocated storage は `VecStorage<T>::Owned(RegionToken<T>)` で表す。型/storage/access/transform/query/mutation/sort の責務は submodule に分離済みで、`MemPtr<T>` は `data_mem_view<T>(&Vec<T>)` の `VecDataView::Data` payload からだけ得る。`VecDataView::Empty` と `VecDataView::Invalid(reason)` は enum branch として分離し、null pointer sentinel で同一視しない。ただし基礎型 `OwnedBuffer<T>` はまだ forgeable `RegionToken<T>` と Copy-only raw element helper に依存するため、owner model の完成には compiler-issued owner token と initialized cell state が必要である。
- `get_ref<T: Copy>` のように Copy 読み取りへ制限した API はあるが、`get(Vec<T>) -> Option<T>` や `pop` などは move-out と owner state の扱いが明確でない。
- `free<T>(Vec<T>)` などの storage free は、要素の Drop / consume と storage-only dealloc を完全には分けていない。

問題:

- `MemPtr<T>` field が owner なのか borrowed view なのか、型から分からない。
- 空/非空、初期化済み prefix、未初期化 capacity、tombstone、moved-out cell が構造化されていない。
- `push(Vec<T>, T) -> Result<Vec<T>, E>` のような fallible update は major collection では owner-preserving error payload へ移行済みだが、新しい owner-consuming fallible API が bare `Diag` / `StdErrorKind` を返さないことを source policy で継続監視する必要がある。
- `replace` / `clear` / `remove` は non-Copy payload の旧値を drop/return する責務を API で表せていない。
- raw header layout は Resource IR にとって追跡対象が増え、静的検査が複雑化する。

## 理想設計

### 1. memory role の分離

最終的な概念分離は次の形にする。

| 役割 | 型 / IR | Copy | free obligation | 説明 |
|---|---|---:|---:|---|
| pointer projection | `MemPtr<T>` | 可 | なし | storage 内の位置を指す non-owning view。 |
| storage owner | `OwnedRegion<T>` / `OwnedBytes` / `Storage<T>` | 不可 | あり | allocator が発行した owner token。stdlib code から forge できない。 |
| initialized cell | Resource IR `CellState` / stdlib wrapper | 不可 | payload に依存 | cell が `Uninit` / `Initialized` / `Moved` / `Dropped` のどれかを表す。 |
| borrowed view | `&T` / slice view | 条件付き | なし | owner を動かさず読むための projection。 |
| finalized value | `str` / `ByteBuf` / collection | 型ごと | 型ごと | public API が扱う所有値。 |

`MemPtr<T>` は最後まで non-owning に固定する。`Option<MemPtr<T>>` は過渡的には許容するが、長期的には `Option<OwnedRegion<T>>` または `StorageState<T>` へ移す。

### 2. storage state は enum で表す

null pointer、数値 status、`len == 0` による owner 有無の暗黙表現はやめる。

概念例:

```neplg2
enum StorageState<.T>:
    Empty
    Allocated OwnedRegion<.T>

struct OwnedBuffer<.T>:
    storage <StorageState<.T>>
    len <i32>
    cap <i32>
```

hash table の bucket も数値ではなく enum にする。

```neplg2
enum BucketState<.K,.V>:
    Empty
    Full Bucket<.K,.V>
    Tombstone
```

これにより `match` が網羅性検査の対象になり、status 値の typo や未知状態を静的に扱える。

### 3. collection API を owner discipline ごとに分ける

collection は少なくとも次の API class に分ける。

| API class | 例 | 制約 |
|---|---|---|
| Copy read | `get_ref<T: Copy>(&Vec<T>, i32) -> Option<T>` | storage から copy するだけ。 |
| borrow read | `get_borrow(&Vec<T>, i32) -> Option<&T>` | owner を動かさず borrow lifetime を返す。 |
| move-out | `remove(Vec<T>, i32) -> RemoveResult<Vec<T>, T>` | cell を Moved にし、collection owner と item owner を同時に返す。 |
| replace | `replace_owned(Vec<T>, i32, T) -> ReplaceResult<Vec<T>, T>` | 旧 item を返すか drop 済みであることを型に出す。 |
| drop/free | `free(Vec<T>) -> ()` | initialized prefix を順に consume/drop してから storage-only dealloc。 |
| fallible update | `push_owned(Vec<T>, T) -> PushResult<Vec<T>, T, E>` | Err で collection / item owner を失わない。 |

`Result<Vec<T>, E>` だけでは、失敗時に入力 owner を返すのか、free するのか、item を保持するのかが分からない。non-Copy payload を扱う API は、失敗時の owner を型に含める。

概念例:

```neplg2
enum PushResult<.C,.T,.E>:
    Ok .C
    Err PushError<.C,.T,.E>

struct PushError<.C,.T,.E>:
    collection <.C>
    item <.T>
    error <.E>
```

短期的に `Result<Vec<T>, E>` を維持する場合は、対象を `T: Copy` に限定するか、Err path で collection/item を完全に consume/drop することを API 名と doc に出す。ただし self-host の compiler data structure では、owner を暗黙に破棄する API を標準にしない。

### 4. `Vec` を collection の基礎型として再設計する

`Vec<T>` は次の不変条件を持つ typed collection にする。

- `0 <= len <= cap`
- `storage = Empty` なら `len = 0` かつ `cap = 0`
- `storage = Allocated(region)` なら region size は `cap * size_of<T>`
- `0..len` は initialized
- `len..cap` は uninitialized storage
- `T: Copy` でない値を byte copy / byte move しない
- grow では initialized prefix を move し、旧 storage は storage-only 状態にしてから free する
- free では initialized prefix を drop/consume し、storage-only になってから dealloc する

現在の `Vec<T> { len, cap, storage: VecStorage<T> }` は `MemPtr<T>` owner field と split `RegionToken<T>` field を消し、`Empty` と allocated token の相関を型で表せるようにした点では前進だが、initialized prefix / moved slot / drop obligation を型と compiler-issued token で表せないため、根本的には `OwnedBuffer<T>` へ置き換える。

### 5. string / byte buffer は collection 設計の先行実装として扱う

`ByteBuf` / `ByteBuilder` は `RegionToken<u8>` owner に移行し、`MemPtr<u8>` を owner field として公開しない形になった。ただしこれは完成形ではない。`RegionToken` は compiler-issued token ではなく、initialized prefix / capacity の証明も `OwnedBytes` ほど明示的ではない。`StringBuilder` は `ByteBuilder` wrapper へ移したため、独立した raw owner field は持たない。

理想:

- `OwnedBytes` は storage owner と len/cap を持つ。
- `ByteBuilder` は `OwnedBytesBuilder` を持ち、finish で `ByteBuf` へ owner を移す。
- `StringBuilder` は UTF-8 生成専用 builder で、`ByteBuilder` / `ByteBuf` owner boundary を通して `str` へ確定する。将来は finish で `OwnedStringRegion` から `str` へ確定する。
- `str` は immutable finalized value であり、通常 API から raw address へ戻せない。
- unchecked conversion は compiler/internal boundary に閉じ、source policy で public stdlib からの使用を監視する。

## 静的検査の要求

Resource IR / typecheck / match check は次を必須にする。

### type safety

- `str` と `i32`、`MemPtr<T>` と `i32`、owner token と pointer projection を型で混同させない。
- bucket status、diagnostic code、token kind、resource state は enum で表し、文字列や数値で管理しない。
- `match` は enum variant の網羅性を検査する。safety-critical enum では wildcard arm に逃げない。
- `T: Copy` が必要な raw copy / read API は trait bound と Resource IR の両方で検査する。

### memory safety

- allocation は owner token を生成し、owner token は exactly once で return / move / free される。
- `MemPtr<T>` の copy は free obligation を複製しない。
- raw `load<T>` は `T: Copy` の copy read と non-Copy の move-out を区別する。
- raw `store<T>` は uninitialized cell の initialize と initialized cell の overwrite を区別する。non-Copy overwrite は旧値の drop/consume が証明される場合だけ許可する。
- dealloc / realloc は initialized non-Copy cell が残る storage を拒否する。
- branch / loop / match merge は owner / cell / borrow state を `MaybeMoved` / `MaybeFreed` として保守的に合流し、曖昧な場合は拒否する。
- function summary は aggregate field、enum payload、raw storage cell、borrow token を projection 単位で保持する。

### diagnostics

- Resource IR diagnostic は `resource.owner.*`、`resource.cell.*`、`resource.borrow.*`、`resource.raw.*`、`resource.lower.*` のように意味分類を失わない。
- self-host の diagnostic id も Rust 側の enum-first 設計に合わせる。内部主キーに文字列や数値を使わない。

## 移行計画

### Stage A: safety policy の固定

- `MemPtr` を owner field として使う新規 public API を禁止する source policy を追加する。
- null pointer owner、direct `Result::Ok Vec ... mem_ptr_wrap 0`、bucket status magic number の監査テストを追加する。
- 既存の `ByteBuf` / builder owner boundary 回帰テストは維持する。

2026-05-13 追記:

- `nodesrc/test_stdlib_memptr_owner_field_policy.js` を追加し、stdlib 全体の `struct` field に直接現れる `MemPtr` / `Option<MemPtr>` を集約して監視する。
- この policy は `MemPtr` field を安全証明として認めるものではない。現時点で残る `RegionToken.ptr` を Stage B/F の移行対象として固定し、それ以外の新規 `MemPtr` owner-like field を禁止する。
- source policy runner に組み込み、module 個別 policy の外で raw-memory-backed owner field が増える退行を検出する。

2026-05-14 追記:

- `StreamWriter.buf` は `StreamWriter` が buffer owner を `ByteBuilder` に集約したことで移行済みになった。残件 baseline は 8 field ではなく 7 field であり、`nodesrc/test_stdlib_memptr_owner_field_policy.js` の transitional allowlist と一致させる。
- `StreamScanner.header` は `StreamScanner` が input owner を `ByteBuf` field、cursor position を typed `Vec<i32>` storage へ分けたことで移行済みになった。残件 baseline は 7 field ではなく 6 field であり、`nodesrc/test_stdlib_memptr_owner_field_policy.js` の transitional allowlist と一致させる。
- compiler core 側では、`RegionToken<T>` の direct constructor restriction と同じ `StructConstructorPolicy::RawMemoryBoundaryOnly(OwnerToken)` を Copy capability impl target validation にも接続した。これにより stdlib public API 移行中でも、owner token を構造的 Copy 型として trait 層から複製可能にする経路を閉じる。
- `VecDataLen<T>` は `Vec.data: MemPtr<T>` と `len` を public struct field として再包装するだけの raw storage view carrier だったため削除した。当時の呼び出し側は raw view と `len<T>(&Vec<T>)` を明示的に別々に扱う方針へ移行し、その後 2026-05-20 に raw view observer は `data_mem_view<T>(&Vec<T>) -> VecDataView<T>` へ置き換えた。残件 baseline は 6 field ではなく 5 field であり、`VecDataLen.data` を transitional allowlist から外した。
- `StringBuilder` は `ByteBuilder` と重複して `Option<MemPtr<u8>>` / len / cap を持つ設計をやめ、`ByteBuilder` owner を保持する typed wrapper へ移した。残件 baseline は 5 field ではなく 4 field であり、`StringBuilder.data` を transitional allowlist から外した。
- `ByteBuf` / `ByteBuilder` は `Option<MemPtr<u8>>` owner field をやめ、storage owner を `RegionToken<u8>` 由来の typed storage enum に集約した。`ByteBufStorage::Empty | Owned(RegionToken<u8>)` と `ByteBuilderStorage::Empty | Owned(RegionToken<u8>)` により、空 storage は zero-size `RegionToken` sentinel を持たず、所有 storage だけが free obligation owner payload を持つ。`region_ptr` / `io_bytebuf_data_ptr_ref` / `byte_builder_data_ptr_ref` は参照から non-owning `MemPtr` view だけを返す。残件 baseline は 4 field ではなく 2 field であり、`ByteBuf.ptr` と `ByteBuilder.ptr` を transitional allowlist から外した。
- `Vec<T>` は `data: MemPtr<T>` owner field をやめ、storage owner を `region: RegionToken<T>` field に集約した。その後 `data_mem_ptr<T>` も削除し、現在は `data_mem_view<T>` が `VecDataView<T>` として valid empty / valid data / invalid invariant を分ける。残件 baseline は 2 field ではなく 1 field であり、`Vec.data` を transitional allowlist から外した。

2026-05-15 追記:

- `Vec.data_ptr<T>(&Vec<T>) -> i32` は public raw address observer として残さず削除した。さらに 2026-05-20 に `data_mem_ptr<T>(&Vec<T>) -> MemPtr<T>` も削除し、raw `i32` address への変換は `data_mem_view<T>` の `VecDataView::Data` branch を持つ raw-memory-boundary 実装箇所に限定する。
- `kpsearch` の lower/upper bound / unique 内部 raw helper は private とし、公開面は `Vec<i32>` owner wrapper だけに揃えた。ordinary source の利用例は raw buffer 構築ではなく `Vec<i32>` による doctest で表す。
- `vec_storage_mem_ptr<T>(VecStorageState, &RegionToken<T>)` は public helper として残さず削除した。storage state から data view への projection は `data_mem_view<T>(&Vec<T>)` が直接 match して所有し、結果を `VecDataView<T>` の enum branch として返す。
- `VecStorageState` と split `region: RegionToken<T>` field は廃止し、`VecStorage<T>::Empty | Owned(RegionToken<T>)` へ移行した。`vec_free_storage<T>` は owner-carrying enum を消費し、`Empty` branch では free obligation が存在しないこと、`Owned` branch では `RegionToken<T>` を消費することを source type と `match` で証明する。borrowed observer は `&VecStorage<T>` match により payload を参照として束縛し、owner token を move/copy しない。
- `Vec` の in-place sort family は storage を書き換える API なので、raw write helper、quick / heap / simple sort、raw slice sort adapter、owner-returning sort wrapper、default `sort` を impure `*>` signature へ揃えた。observer の `sort_is_sorted` と比較 helper は pure のまま残し、effect contract でも観察と破壊的更新を分離する。
- `alloc/collections/vec/sort` root facade は raw `MemPtr` helper と raw slice adapter を再公開しない。さらに direct-importable な `sort/raw/*` helper module、`sort/merge/buffer`、`sort/merge/range` も残さず、raw traversal は `sort_quick` / `sort_heap` / simple sort / `merge/api` など、len・storage view・scratch owner を扱う検査済み implementation file 内の private boundary に閉じる。ordinary caller は `Vec` の sort API と `sort_is_sorted` observer だけを使う。
- root `alloc/collections/vec` facade は unchecked raw element helper を再公開しないだけでなく、`alloc/collections/vec/raw` という直接 import 可能な bypass module 自体を残さない。Copy element の raw load/store は、`get`/`push`/`replace`/`pop`/`map`/`filter`/`prefix` など、len・storage variant・capacity を検査する public 操作と同じ source file 内に置き、検査条件と raw operation を分離しない。
- `RegionToken<T>` の realloc は `core/mem/pointer/region.nepl` の `realloc_region_bytes_keep<T>` へ集約した。`Vec` / `ByteBuilder` は token の `ptr` / `size` を直接分解して `realloc_ptr` を呼ばず、成功時は新 owner、失敗時は旧 owner を `RegionReallocError<T>` 経由で受け取る。Resource IR owner checker は `region_ptr(&region)` から得た非所有 `MemPtr` の raw-address alias を集約値コピー越しに保持するため、`realloc_region_bytes_keep` の Ok / Err 両 payload が free obligation owner を返すことを compiler 側で証明する。
- `Vec.push` の grow capacity は `vec_next_capacity<T>` で `.T` の element size と allocator payload 上限から証明してから決定する。unchecked `cap * 2` は push hot path から外し、上限超過は `CapacityExceeded` として `VecPushError` に元 `Vec` owner を戻す。
- root `std/fs` / `std/stdio` facade は raw ABI submodule を再公開しない。WASI / LLVM syscall helper と raw scratch helper は `std/fs/raw` / `std/stdio/raw` を明示 import した implementation boundary だけに置き、通常の filesystem / standard I/O import は safe public surface に限定する。

2026-05-16 追記:

- `RegionToken<T>` は `ptr: MemPtr<T>` を持たず、`raw: i32, size: i32` の owner token layout に移行した。`MemPtr<T>` は `region_ptr<T>(&RegionToken<T>)` / `region_ptr_at<T,U>` の checked non-owning projection としてだけ使う。
- `region_new<T>` は internal boundary で allocator / realloc helper が返した raw owner identity と size を束ねて token を構築する。`MemPtr<T>` は non-owning view なので `region_new` の入力にはしない。通常 source は `RegionToken` constructor、raw field projection、`region_token_raw_ref` のような raw identity accessor を直接使えず、compiler owner aggregate boundary と source capability が検査する。
- Resource IR owner summary は direct raw memory op だけでなく、callee summary が消費する raw owner alias も seed する。これにより `dealloc_region<T>` が `RegionToken.raw` から一時 `MemPtr.raw` を作り `dealloc_ptr<T>` へ渡す場合も、caller では `RegionToken<T>` owner が消費されたことが証明される。
- `nodesrc/test_stdlib_memptr_owner_field_policy.js` の transitional allowlist は 0 件になった。今後 stdlib の struct field に `MemPtr` / `Option<MemPtr>` を owner-like field として戻すことは禁止する。
- ただし `RegionToken<T>` はまだ compiler-issued `OwnedRegion<T>` ではない。Stage B/D/F の最終目標は、`RegionToken` を過渡 owner token として閉じ、`OwnedBuffer<T>` / `StorageState<T>` / initialized prefix / compiler-issued free obligation owner へ進めることである。
- 追加調査で確認した public `alloc_ptr<T>` / `realloc_ptr<T>` / `dealloc_ptr<T>` は [ISS-20260515T163252091Z-PUBLIC-ALLOC-PTR-APIS-STILL-ENCODE-F-EECDD686](../../issues/items/ISS-20260515T163252091Z-PUBLIC-ALLOC-PTR-APIS-STILL-ENCODE-F-EECDD686.md) で削除済みである。2026-05-16 時点で、safe facade だけでなく direct import 可能な `core/mem/pointer/alloc` module 自体も残さない。
- `std/stdio/write/byte.nepl` の `print_byte` は 1 byte scratch buffer を `RegionToken<u8>` owner と non-owning `MemPtr<u8>` view に分けた。続く stdio fd write/read、std/fs fd write/read/open/stat、LLVM fallback C string、std/env/cliarg raw scratch も同じ方針で RegionToken owner と raw ABI view を分離済みである。
- `std/stdio/write/fd.nepl` も `iov` / `nwritten` scratch owner を `RegionToken<u8>` に移し、raw iovec layout は `std/stdio/raw.nepl` の `stdio_fd_write_from_result` へ閉じた。stdout/stderr write path では `MemPtr<u8>` を free owner として扱わない。
- `Vec<T>` は `buffer: OwnedBuffer<T>` だけを持つ public facade へ移行した。`OwnedBuffer<T>` が `len/cap/storage` と storage owner enum を保持し、observer / mutation / transform / sort wrapper はすべて `OwnedBuffer<T>` 経由で storage を参照または消費する。source policy は旧 `Vec<T> len cap storage` constructor と `Vec<T>` 直下の `len/cap/storage` field の復活を拒否する。残件は initialized prefix、move-out / replace / drop traversal、compiler-issued owner token の接続である。

2026-05-19 追記:

- Resource IR owner summary は host memory span だけでなく raw memory operation の readable / writable span も caller requirement として要約する。`string_from_mem_unchecked_result` / `string_from_utf8_mem_result` のような stdlib callee 内 raw span helper は、caller 側の `RegionToken` extent と length の照合を要求する。
- これは `MemPtr<T>` に owner 責務を戻す変更ではない。`MemPtr<T>` は non-owning view のままで、proof は backing owner と span requirement の対応から導出する。
- `cstr_to_str_bounded_result` のような loop-guarded scanner では、`while i < max_len` と `load_u8(mem_ptr_add p i)` から `p[0..max_len]` を要約する汎用 path-conditioned span proof がまだ不足している。これは [ISS-20260519T144811685Z-RESOURCE-IR-RAW-SPAN-SUMMARIES-MISS--FA49E19D](../../issues/items/ISS-20260519T144811685Z-RESOURCE-IR-RAW-SPAN-SUMMARIES-MISS--FA49E19D.md) で扱う。

2026-05-20 追記:

- raw-memory-backed API の staged effect migration parent は、[ISS-20260427T204839136Z-STDLIB-RAW-MEMORY-BACKED-APIS-REQUIR-E503CD84](../../issues/items/ISS-20260427T204839136Z-STDLIB-RAW-MEMORY-BACKED-APIS-REQUIR-E503CD84.md) として closure audit 済みである。raw memory primitive は compiler の internal effect / Resource IR gate / source capability proof に接続され、public raw `MemPtr` surface は source policy で監視されている。
- collection free が non-Copy payload を受け入れて要素 Drop を呼ばない旧バグは、[ISS-20260425T000000Z-RV-STDLIB-004-91534828](../../issues/items/ISS-20260425T000000Z-RV-STDLIB-004-91534828.md) の closure audit で fixed とした。現行 public surface は Copy-only 境界と source policy で guarded である。
- これは non-Copy collection の完成を意味しない。compiler-issued owner token、moved slot、drop traversal、`InitializedCell` / Resource IR 接続による final support は [ISS-20260520T152218366Z-NON-COPY-COLLECTION-PAYLOAD-SUPPORT--A6A88543](../../issues/items/ISS-20260520T152218366Z-NON-COPY-COLLECTION-PAYLOAD-SUPPORT--A6A88543.md) で扱い、self-host 実装はそれまで safe subset を使う。
- 2026-05-21 に [ISS-20260521T020307778Z-COLLECTION-SLOT-OWNER-TRANSFER-NEEDS-403A919A](../../issues/items/ISS-20260521T020307778Z-COLLECTION-SLOT-OWNER-TRANSFER-NEEDS-403A919A.md) で、同一関数内の non-Copy slot owner-transfer は raw `StoreValue` / `MoveOutLoadedCell` fact を消費する generic proof として通せるようになった。summary replay の branch / loop / indirect call でも `CellTable` を path merge するため、片方の summary path だけの raw value-flow fact は後続 proof に使えない。これは positive support の最初の段階であり、callee summary certified proof と drop traversal はまだ parent issue の残件である。
- 2026-05-21 に [ISS-20260521T025115696Z-COLLECTION-SLOT-SUMMARIES-NEED-CERTI-92379E7C](../../issues/items/ISS-20260521T025115696Z-COLLECTION-SLOT-SUMMARIES-NEED-CERTI-92379E7C.md) で、callee 内で証明済みの raw value-flow proof を `CollectionSlotLifecycleSummaryEventProof::OwnerTransferValueFlow` として summary event に載せるようになった。caller replay はその certified proof を obligation と照合し、caller-local raw fact の再要求や stdlib helper allowlist なしに non-Copy slot initialize / move-out を通す。

2026-05-21 追記:

- `CollectionSlotLifecycleEvent` / `CollectionSlotStateTable` は、`ResourceOp::CollectionSlotLifecycle` として Resource IR の通常命令に接続した。これにより non-Copy collection slot の Initialize / BorrowRead / MoveOut / Replace / Drop / StorageDealloc は、stdlib module ごとの個別許可ではなく compiler core の generic typed proof boundary として検査される。
- initialized checker は `CollectionSlotStateTable` を関数状態に含め、branch / loop / match の各 path を `CollectionSlotStateTable::merge_paths` で合流する。片側だけ initialized な slot や片側だけ storage release 済みの状態は `MaybeInitialized` / `MaybeReleased` として残り、合流後の reinit / move / drop / storage dealloc を typed refutation で拒否する。
- `ResourceCheckDiagnostic::CollectionSlotRefuted` と `ResourceCollectionSlotDiagnosticCode` を追加し、slot lifecycle の失敗理由を enum のまま保持する。diagnostic id は serialization 用の dotted string であり、checker 内の authority にはしない。
- 今回の接続で Resource IR 上の手書き operation は検査対象になったが、stdlib collection lowering が実際の `Vec<T>` / `OwnedBuffer<T>` operation から `ResourceOp::CollectionSlotLifecycle` を発行する段階は未完である。次段階では source policy や module allowlist に逃がさず、collection API の source semantics から generic slot lifecycle event を生成する。
- callee 内の slot lifecycle effect は `CollectionSlotLifecycleFunctionSummary` として caller へ伝播する。summary は parameter projection 上の `Event` / `Merge` / `Loop` であり、direct call は actual arg へ suffix を置換して再適用し、function alias indirect call は候補関数ごとに state path を merge する。これにより helper 関数を跨いだ live slot / moved slot / partial branch slot state は caller の `CollectionSlotStateTable` に残る。関連 issue は [ISS-20260520T200325099Z-COLLECTION-SLOT-LIFECYCLE-EFFECTS-DO-6E6407FC](../../issues/items/ISS-20260520T200325099Z-COLLECTION-SLOT-LIFECYCLE-EFFECTS-DO-6E6407FC.md)。
- callee が owner parameter をそのまま返す owner-preserving helper では、callee 内に新しい slot lifecycle event がなくても caller actual の slot state を call output へ transfer する必要がある。`CollectionSlotLifecycleFunctionSummary` は `return_transfers` を持ち、return value と parameter prefix の関係から generic summary fact を作る。call site では raw alias canonicalization を通して actual arg prefix から output prefix へ slot state を移すため、stdlib module allowlist なしに owner-preserving API を跨いだ live slot を検出できる。関連 issue は [ISS-20260521T004808159Z-COLLECTION-SLOT-SUMMARIES-DO-NOT-TRA-2AA84347](../../issues/items/ISS-20260521T004808159Z-COLLECTION-SLOT-SUMMARIES-DO-NOT-TRA-2AA84347.md)。
- `InitializeEmpty` / `MoveOut` / `ReplaceReturnOld` / `ReplaceDropOld` は payload value-flow proof と結び付いていなければならない。slot state transition だけを annotation として許すと、実際の payload consume / materialize が欠落しても Resource IR が所有権移動を信じてしまうため、2026-05-21 時点では non-Copy payload の owner-transfer lifecycle event を `OwnerTransferRequiresValueProof` として拒否する。Copy payload の state marker は維持するが、non-Copy collection API の positive support は payload place proof と compiler-owned slot-drop lowering を実装してから開く。collection slot の place 正規化も raw value origin ではなく owner-cell canonicalization を使い、value move 後の storage dealloc が移動先 live slot を見落とさないようにした。関連 issue は [ISS-20260521T010410090Z-COLLECTION-SLOT-OWNER-TRANSFER-LIFEC-3C1056B2](../../issues/items/ISS-20260521T010410090Z-COLLECTION-SLOT-OWNER-TRANSFER-LIFEC-3C1056B2.md)。
- production lowering / annotation path は [ISS-20260520T200531197Z-COLLECTION-SLOT-LIFECYCLE-HAS-NO-PRO-298A1B25](../../issues/items/ISS-20260520T200531197Z-COLLECTION-SLOT-LIFECYCLE-HAS-NO-PRO-298A1B25.md) で対応済みである。`CollectionSlotLifecyclePrimitive`、`SourceCapabilityUseSite::CollectionSlotLifecycleBoundary`、typecheck gate、Resource IR lowering producer を接続し、compiler-proven source evidence を持つ exact span からだけ `ResourceOp::CollectionSlotLifecycle` を生成する。
- collection slot lifecycle producer は stdlib function name allowlist ではない。compiler-owned intrinsic を typed enum で分類し、source capability proof と Resource IR checker が同じ lifecycle event を見る。anchor は `MemPtr<T>` / `RegionToken<T>` の element type と intrinsic type args を照合し、storage dealloc は owner token anchor に限定する。これにより、Vec/Stack/Queue など module ごとの proof machine を増やさず、generic `CollectionSlotStateTable` へ集約する。
- collection slot lifecycle intrinsic の public stdlib wrapper は追加しない。通常 import 可能な wrapper は user source に lifecycle event 偽造の入口を与えるため、stdlib collection 実装が必要な箇所で compiler-proven source evidence を持つ境界として使う。現行 Copy-only `Vec` に無理に注釈を付けるのではなく、non-Copy API と drop traversal を導入する段階でこの producer を使う。
- 関連 issue: [ISS-20260520T192939566Z-RESOURCE-IR-DOES-NOT-CARRY-COLLECTIO-5585A1D7](../../issues/items/ISS-20260520T192939566Z-RESOURCE-IR-DOES-NOT-CARRY-COLLECTIO-5585A1D7.md)。

### Stage B: `core/mem` の internal/public 分離

- 前提として、typecheck の import visibility が `pub` / private item boundary を binding authority として扱う必要がある。現状の blocker は [ISS-20260512T235355207Z-IMPORT-VISIBILITY-DOES-NOT-ENFORCE-P-30FB5573](../../issues/items/ISS-20260512T235355207Z-IMPORT-VISIBILITY-DOES-NOT-ENFORCE-P-30FB5573.md) で追跡する。
- raw `i32` allocator / load/store は internal raw module に移す。
- public API は `MemPtr` view、`OwnedRegion` owner、initialized cell 操作を分ける。
- `region_new` のような token forging API を廃止する。
- `dealloc_region` は storage-only region にだけ許可する。

### Stage C: string / byte buffer を owner wrapper へ移行

- `ByteBuf` / `ByteBuilder` の `Option<MemPtr<u8>>` owner field は削除済みである。`ByteBuf` は `ByteBufStorage::Empty | Owned(RegionToken<u8>)`、`ByteBuilder` は `ByteBuilderStorage::Empty | Owned(RegionToken<u8>)` に移し、empty sentinel も削除済みである。`ByteBuilder` の fallible reserve / append / finish は owner-preserving error payload に揃えた。次は forgeable `RegionToken<u8>` を `StorageState<u8>` または compiler-issued `OwnedBytes` に移す。
- `string_finish_base` は削除済みであり、raw address から `str` への変換も private `string_from_addr_unchecked` に閉じた。次は `RegionToken<u8>` を `OwnedStringRegion` / compiler-issued owner token へ移す。
- fallback API と Result API の責務を整理し、allocation failure を空値 success に潰さない入口を標準にする。

### Stage D: `OwnedBuffer<T>` と Vec 再実装

- `Vec<T>` を `OwnedBuffer<T>` 上に再実装する。2026-05-16 時点で public facade と backing storage owner の分離は完了し、`Vec<T>` 直下の `len/cap/storage` は削除済みである。
- Copy read、borrow read、move-out、replace、clear、free を別 API に分ける。
- fallible push/grow は Err path で collection / item owner を返す設計にする。
- `Vec<T>` の doctest は `T: Copy` と non-Copy payload の両方を持つ。

### Stage E: derived collections の raw header 廃止

- `Stack` / `Queue` / `Deque` / `BinaryHeap` は raw header ではなく `Vec` / `OwnedBuffer` wrapper へ戻す。
- bitset / bloom filter / adjacency matrix は `OwnedBytes` を使う。
- `HashMap` / `HashSet` は移行済みの `BucketState` enum と typed bucket storage 契約を維持する。
- `BTreeMap` / `BTreeSet` の sorted-array 実装は小規模 ordered table 用であることを名前と doc に出す。

### Stage F: Resource IR special-case の削減

- stdlib 側が owner wrapper を持った後、raw alias summary / external raw root special-case を減らす。
- 残すべきものは unsafe/internal boundary の検査に限定する。
- self-host ResourceIR は最初からこの設計に合わせ、過去の HIR 直走査 special-case を移植しない。

## self-host 開始可否

開始は可能だが、使用可能な stdlib subset を制限する必要がある。

使用してよいもの:

- `ByteBuf` / `ByteBuilder` / `StringBuilder` の current Result API。
- `str` の UTF-8 / char / prefix / slice 系 API。ただし unchecked raw conversion を compiler core に持ち込まない。
- `Vec<T>` は `T: Copy` の token id / type id / span id などに限定して使う。
- 小規模の ordered table は `sorted_array_*` と明示した用途に限定する。

使用を避けるもの:

- non-Copy payload を入れる `Vec` / `Stack` / `HashMap`。
- raw header / raw node / raw byte storage discipline を ResourceIR / typecheck の中核データ構造に使うこと。
- `MemPtr` / raw address / `RegionToken` を self-host compiler core の public data structure に持つこと。
- failure path で owner を返さない fallible collection update。

したがって self-host の lexer / parser は進められるが、typecheck / ResourceIR / diagnostic 集約へ入る前に `OwnedBuffer` と collection redesign を進める必要がある。

## 関連 issue

| Issue | 関係 |
|---|---|
| `ISS-20260427T152958303Z-MEMPTR-AND-REGIONTOKEN-LACK-COMPILER-0BC8ECDF` | `MemPtr` / `RegionToken` の owner/provenance 分離。 |
| `ISS-20260514T054314434Z-COPY-IMPL-CAN-MARK-COMPILER-OWNER-TO-D6C08048` | compiler owner token の線形性を Copy capability impl で崩せる経路を typecheck boundary で拒否。 |
| `ISS-20260427T164432612Z-CORE-MEM-DEALLOC-APIS-DO-NOT-ENCODE--204F1F47` | initialized payload と storage-only dealloc の分離。 |
| `ISS-20260427T204839136Z-STDLIB-RAW-MEMORY-BACKED-APIS-REQUIR-E503CD84` | raw-memory-backed stdlib API の段階移行親 issue。2026-05-20 に closure audit 済み。 |
| `ISS-20260425T000000Z-RV-STDLIB-004-91534828` | 旧 collection free/drop bug。2026-05-20 に Copy-only public surface と policy coverage の closure audit で fixed。 |
| `ISS-20260520T152218366Z-NON-COPY-COLLECTION-PAYLOAD-SUPPORT--A6A88543` | non-Copy payload collection の final support。compiler-issued owner token、initialized/moved/drop state、Resource IR proof boundary へ接続する後続 issue。 |
| `ISS-20260521T020307778Z-COLLECTION-SLOT-OWNER-TRANSFER-NEEDS-403A919A` | raw `StoreValue` / `MoveOutLoadedCell` から local non-Copy slot owner-transfer proof を作り、collection slot lifecycle が消費できるようにした issue。 |
| `ISS-20260520T214013832Z-COLLECTION-SLOT-LIFECYCLE-STATE-DOES-FA4DE5B2` | collection storage relocation で slot lifecycle state を old storage prefix から new storage prefix へ rekey する Resource IR operation。 |
| `ISS-20260514T055830236Z-VECDATALEN-CARRIES-RAW-VEC-STORAGE-V-B662D7DF` | `VecDataLen` raw storage view carrier の削除。 |
| `ISS-20260514T063755030Z-STRINGBUILDER-DUPLICATES-BYTEBUILDER-F90DFA2F` | `StringBuilder` 固有 raw owner field を `ByteBuilder` owner boundary へ集約。 |
| `ISS-20260514T071955576Z-BYTEBUF-STORES-OWNED-BYTES-AS-OPTION-FA165159` | `ByteBuf` / `ByteBuilder` の `Option<MemPtr<u8>>` owner field を `RegionToken` owner boundary へ集約。 |
| `ISS-20260519T181131422Z-BYTEBUILDER-FALLIBLE-OWNER-APIS-DISC-DBFDE7BB` | `ByteBuilder` の reserve / append / finish failure を owner-preserving `ByteBuilderError` / `ByteBuilderByteBufError` へ移行。 |
| `ISS-20260514T085248522Z-VEC-STORES-BACKING-STORAGE-AS-MEMPTR-FFC9775A` | `Vec.data` raw `MemPtr` owner field を `Vec.region: RegionToken<T>` owner boundary へ集約。 |
| `ISS-20260515T223330574Z-VEC-STORAGE-TAG-AND-REGIONTOKEN-OWNE-DDDAD134` | split `VecStorageState` / `RegionToken<T>` field を `VecStorage<T>::Owned(RegionToken<T>)` へ統合し、Empty と owned token の相関を型で証明。 |
| `ISS-20260516T005642964Z-VEC-STORES-BACKING-STORAGE-DIRECTLY--2407B1D0` | `Vec<T>` を `buffer: OwnedBuffer<T>` facade にし、`len/cap/storage` を backing storage owner 側へ移す。 |
| `ISS-20260515T141747916Z-VEC-GROW-REIMPLEMENTS-REGIONTOKEN-RE-255A043F` | `Vec` / `ByteBuilder` の `RegionToken` realloc を core/mem に集約し、Vec grow capacity overflow proof を追加。 |
| `ISS-20260515T155626605Z-REGIONTOKEN-STILL-STORES-MEMPTR-AS-O-C0F9E4D1` | `RegionToken<T>` の `MemPtr<T>` owner-like field を direct raw owner identity へ置き換え、MemPtr owner-field transitional baseline を 0 件にする。 |
| `ISS-20260515T163252091Z-PUBLIC-ALLOC-PTR-APIS-STILL-ENCODE-F-EECDD686` | public `alloc_ptr` / `realloc_ptr` / `dealloc_ptr` が `MemPtr<T>` を free obligation carrier として公開していた問題。2026-05-16 に削除済み。 |
| `ISS-20260515T170146857Z-CORE-MEM-POINTER-FACADE-RE-EXPORTS-L-4724AF44` | `core/mem` / `mem/pointer` safe facade から低レベル `alloc_ptr` owner wrapper の re-export を削除し、`mem_ptr_add` を non-owning view helper へ分離。 |
| `ISS-20260516T041311764Z-STAGE-6-MEMORY-MODEL-DOC-STILL-DESCR-AEC8348B` | Stage 6 文書に残った削除済み `alloc_ptr` owner path / `core/mem/pointer/alloc` import 前提を現在の設計へ同期。 |
| `ISS-20260518T072713737Z-ALLOC-STRING-PUBLIC-UNCHECKED-BYTE-A-896205D1` | root `alloc/string` と `access.nepl` から unchecked byte reader を外し、残る明示 `unchecked_access` 境界を削除して private `StringByteIndex` witness に移した。 |
| `ISS-20260514T102108865Z-VEC-SORT-MERGE-RET-ERR-PATH-LOSES-CO-98B83660` | `sort_merge_ret<T>` の失敗 payload に `Vec<T>` owner を返し、merge sort scratch buffer を `RegionToken<T>` owner へ移行。 |
| `ISS-20260429T120339805Z-FALLIBLE-OWNING-COLLECTION-UPDATES-L-21EF56CB` | fallible collection update の owner loss。 |
| `ISS-20260518T134512597Z-DEQUE-PUSH-GROW-FAILURE-DESTROYS-THE-081F1BD4` | `Deque.push_front` / `push_back` の grow failure で元 deque owner を `DequePushError<T>` に戻す Stage 6 子 issue。 |
| `ISS-20260429T131646897Z-BYTEBUF-EMPTY-NON-EMPTY-CONDITIONAL--34FBA0C2` | ByteBuf の空/所有 storage 構造化。 |
| `ISS-20260429T142213822Z-BYTEBUILDER-AND-STRINGBUILDER-RESULT-4EB1D1EB` | builder owner boundary の修正済み回帰。 |
| `ISS-20260429T155343006Z-COLLECTION-STORAGE-STATES-USE-NUMERI-E4B3A749` | collection の数値/null storage state を enum owner state へ移す新規 issue。 |
| `ISS-20260428T223953830Z-VEC-ELEMENT-LOADS-LOSE-BACKING-STORA-E811458B` | Vec backing storage と Resource IR raw cell alias の修正済み回帰。 |
| `ISS-20260429T083822053Z-SELF-HOST-DIAGNOSTICS-USE-STRING-COD-1040C21E` | self-host diagnostics も enum-first diagnostic id に合わせる。 |
| `ISS-20260513T212118060Z-MEMPTR-OWNER-FIELD-MIGRATION-LACKS-G-7E2612E2` | Stage A の `MemPtr` owner-like field 増殖禁止 source policy。 |

## 判定

現状の方向性は「過渡期としては正しいが、最終設計としては未完」である。

`ByteBuf` / `ByteBuilder` / `StringBuilder` / `Vec` は、空/所有 storage を型に出す方向へ進んだため短期利用に耐える。`StringBuilder` 固有の raw owner field、`Vec.data` raw owner field、`Vec<T>` 直下の `len/cap/storage` は削除済みである。一方で `core/mem` と collections は、forgeable `RegionToken` と Copy-only raw element helper にまだ依存しており、Resource IR が後追いで alias と initialized cell を復元する構造が残っている。この複雑さは設計上の警告であり、さらに特例を足して維持すべきではない。

2026-05-16 時点で、`core/mem` root と `mem/pointer` facade は低レベル `alloc_ptr` / `realloc_ptr` / `dealloc_ptr` を再公開しない。さらに direct import 可能な `core/mem/pointer/alloc` module 自体も削除済みである。safe caller の標準経路は `alloc_region` / `region_ptr` / `dealloc_region` であり、低レベル scratch 実装は `RegionToken` owner と `region_ptr` 由来の non-owning ABI view に分ける。これは最終設計ではなく、forgeable `RegionToken` を `OwnedBytes` / `OwnedBuffer` / compiler-issued owner token へ置き換える前段である。

理想は、stdlib が owner state を型で表し、Resource IR がその型構造をそのまま検査できる状態である。self-host の型検査・メモリ検査を妥協しないためには、collection 再設計を避けず、`OwnedBuffer` / owner token / initialized prefix / enum state / match exhaustiveness を中核に置く。
