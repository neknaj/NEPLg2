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
| `core/mem` | `MemPtr<T>` / `RegionToken<T>` / `alloc_ptr` / `dealloc_ptr` が存在する。raw `i32` API と typed wrapper が同じ module に同居している。 | 過渡。Resource IR の防壁は増えたが、設計としては owner token と pointer projection がまだ分離不足。 | internal raw API と public safe API を分離し、compiler-issued owner token へ移行する。 |
| `alloc/io` `ByteBuf` | `io/bytebuf` が `ByteBuf`、RegionToken 経由の確定境界、str との checked conversion を所有する。`ByteBuf.region: RegionToken<u8>` が free obligation owner で、`MemPtr<u8>` は参照から得る非所有 view に限定した。 | 良い方向。短期 self-host の binary I/O buffer として使用可能。root facade に raw capability を戻さない。 | `OwnedBytes` / compiler-issued owner token 設計が入った後に forgeable `RegionToken` から移行する。 |
| `alloc/io` `ByteBuilder` | `io/bytebuilder` が grow/reserve/append/finish を所有する。`region: RegionToken<u8>` と `byte_builder_with_len` で storage owner を field 全体として移し、append は参照から非所有 pointer view を得る。safe API は pure surface とし、raw memory effect は raw-memory-boundary source 内で Resource IR が検査する。 | 良い方向。builder owner leak は回帰テストで監視済み。raw capability は `io/bytebuilder` に限定した。 | fallible append の失敗契約を collection と揃え、将来の compiler-issued owner wrapper に合わせる。 |
| `alloc/string` `str` | `string_alloc_region` / `string_finish` で `RegionToken` を使う経路が増えた。UTF-8 と char API も進み、numeric conversion は `integer/common` / `integer/format` / `integer/parse` / `float/format` / `float/parse` へ分割済み。 | 部分的に良い。`str` 確定境界と module 責務は改善済みだが、raw address helper はまだ内部 discipline に依存する。 | `str` 生成 API を `OwnedStringRegion` に寄せ、unchecked helper を internal boundary に閉じる。 |
| `alloc/string` `StringBuilder` | `bytes: ByteBuilder` を保持する typed text builder wrapper。append は `ByteBuilder` の byte owner boundary へ委譲し、build は `ByteBuilder -> ByteBuf -> str` の確定経路を通す。 | 良い方向。StringBuilder 固有の raw `MemPtr` owner field はなくなった。短期 self-host の text builder として使用可能。 | `ByteBuilder` / `ByteBuf` 側を将来の `OwnedBytes` / `OwnedStringRegion` へ移行する。 |
| `alloc/collections/Vec` | `VecStorageState::Empty/Owned` と `region: RegionToken<T>` を分け、空 storage は enum state と zero-size token で表す。型定義は `vec/types`、storage allocation/free は `vec/storage`、borrowed observer は `vec/access`、raw load/store と scan/fold helper は `vec/raw`、owner-consuming transform は `vec/transform`、borrowed query は `vec/query`、mutation/cleanup は `vec/mutation` が所有する。allocation constructor、raw data observer、raw element helper、`push` / `pop` / `sort` / `clear` / `free` / `vec_free_storage` は、raw storage identity / raw copy / storage-only cleanup / raw swap に依存するため `.T: Copy` に限定済み。`MemPtr<T>` は `RegionToken<T>` 参照から得る non-owning view としてだけ使う。 | 改善済みの過渡。`MemPtr` owner field は消えたが、`RegionToken<T>` が forgeable である点と、non-Copy payload collection の owner-preserving update / move-out / drop traversal は `OwnedBuffer<T>` 化まで残る。 | `OwnedBuffer<T>` + initialized prefix へ移行し、read/copy/move/drop/free を分離する。 |
| `Stack` / `Queue` / `Deque` / `RingBuffer` / `BinaryHeap` | raw header は廃止済みで、`len/cap/head/items: Vec<Option<T>>` 系へ移行済み。live/inactive slot は `Some` / `None` で表す。 | 良い方向。ただし現行 `Vec` は `RegionToken<T>` owner へ移っただけで、compiler-issued `OwnedBuffer<T>` と initialized prefix は未完である。 | `OwnedBuffer<T>` 再実装後、`Vec<Option<T>>` 依存を新 buffer model に移す。 |
| `HashMap` / `HashSet` | `BucketState` enum と typed bucket storage を導入済み。key/value/key-only storage は `Vec<Option<...>>` で初期化状態を表す。 | 良い方向。Copy payload 前提では Resource IR が storage owner と initialized slot を追いやすい。 | source policy で raw header / numeric sentinel への退行を防ぎ、非 Copy payload は別設計で扱う。 |
| `BTreeMap` / `BTreeSet` | sorted-array typed storage を使い、key/value slot は `Vec<Option<T>>` で表す。`key_eq` は by-value `ord_lt` を 2 回呼ぶため `.K: Ord&Copy` / `.T: Ord&Copy` に限定済み。`insert` / grow failure は `BTreeMapInsertError` / `BTreeSetInsertError` に元 owner と `Diag` を入れて返す。 | 良い方向。raw header はなく、non-Copy key の二重消費入口と grow failure で owner を隠す入口を閉じた。ただし名称通りの木構造ではなく小規模 ordered table である。 | borrowed comparison と non-Copy key/value の owner-preserving update は `OwnedBuffer<T>` と initialized cell state が入った後に別 API として設計する。 |
| bitset 系 collection | `BitSet` / `AdjacencyMatrix` / BloomFilter / CountingBloomFilter は `Vec<u8>` storage へ移行済み。 | 良い方向。byte collection 固有の raw owner field は解消済み。 | `Vec` の基礎 owner model を `OwnedBuffer` へ移行し、byte collection もそれに追従する。 |
| `List` | raw node chain を廃止し、`items: Vec<T>` storage へ移行済み。論理先頭は `Vec` 末尾で、`tail` は `Vec.pop` により owner を返す。 | 良い方向。raw node owner は解消したが、Copy payload 前提と `Vec` 基礎 owner model の制約は残る。 | `OwnedBuffer<T>` 化後に borrowed observer と non-Copy payload contract を追加する。 |
| Rust Resource IR | `CellState` / `OwnerState` / `BorrowState` / `StorageOrigin` があり、raw memory op、aggregate projection、enum payload、branch merge を追跡している。 | 方向は正しい。現状は stdlib の曖昧な所有権表現を補うための alias 処理が多い。 | stdlib 側の owner wrapper 化に合わせ、特例的 alias summary を減らす。 |
| self-host stdlib 利用 | ByteBuf / builders / Copy-only Vec / typed slot collection は使えるが、non-Copy payload collection や fallible update は危険。 | 制限付き開始は可能。ただし ResourceIR / typecheck 実装で raw collection discipline を増やしてはいけない。 | token stream / diagnostics / symbol table は安全 subset か専用 typed collection を使う。 |

## 2026-04-30 再レビュー結果

基準: remote main `bbaf2a5` 取り込み後。

今回の再確認では、collection 改善が進み、以前の「raw header が多い」という記述は古くなっている。一方で、`Vec` と `core/mem` の根本設計が未完であるため、最終的な memory safety / type safety 設計はまだ完了していない。

| 分類 | 現状の実装 | 安全性判定 | 理想 |
|---|---|---|---|
| hash collection | `HashMap` / `HashSet` は bucket state enum と `Vec<Option<T>>` storage。 | 良い。数値 status ではなく enum / `match` で検査できる。 | non-Copy payload 用 owner-preserving update / drop traversal を追加する。 |
| derived linear collection | `Queue` / `Deque` / `RingBuffer` / `Stack` / `BinaryHeap` は `Vec<Option<T>>` storage。 | 良い方向。raw header は消えたが `Vec` 依存なので基礎 storage owner は未完。 | `OwnedBuffer<T>` 上の slot-state collection へ移す。 |
| ordered table | `BTreeMap` / `BTreeSet` は sorted-array typed storage。grow failure は owner-preserving insert error で caller に元 owner を返す。 | 良い方向。名前上は BTree だが実装は ordered array table。allocation failure の owner contract は API 型に現れた。 | self-host 用には用途を小規模 ordered table と明記し、大規模 map は別設計にする。 |
| list collection | `List` は `items: Vec<T>` storage。`reverse` / `map` / `filter` / observer は raw node を使わず owner を閉じる。 | 良い方向。raw node は解消済み。Copy payload 前提と `Vec` 基礎 owner model は残る。 | `OwnedBuffer` based storage と borrowed observer / non-Copy payload contract へ追従する。 |
| byte/bit collection | `BitSet` / `AdjacencyMatrix` / BloomFilter / CountingBloomFilter は `Vec<u8>` storage。 | 良い方向。byte/bit collection の payload owner field は `Vec<u8>` に統一済み。 | `Vec` の基礎 owner model を `OwnedBuffer` へ移し、byte collection も `OwnedBytes` 相当の安全境界へ追従する。 |
| numeric array collection | SparseSet / Fenwick / SegmentTree / DisjointSet は `Vec<i32>` storage。 | 良い方向。numeric collection 固有の raw i32 storage は解消済みだが、基礎 `Vec` の owner model は未完。 | `OwnedBuffer<i32>` / typed index API へ移す。 |
| `Vec<T>` | `len/cap/storage/region` を持ち、型/storage/access/raw helper/transform/query/mutation の責務は submodule に分離済み。空 owner state は `VecStorageState::Empty` と zero-size `RegionToken<T>` で表す。 | 最重要残件。`MemPtr` owner field は解消したが、`RegionToken<T>` が forge 可能で、initialized prefix / moved slot / drop traversal は型に出ていない。 | `OwnedBuffer<T>` + `StorageState<T>` + initialized prefix に再実装する。 |
| `core/mem` | raw allocator、`MemPtr<T>`、`RegionToken<T>`、load/store が同居。 | 過渡。token forging と owner/view 混同が残る。 | public safe API と internal raw API を分離し、compiler-issued owner token にする。 |
| `alloc/string` / `alloc/io` | `RegionToken` owner と `ByteBuilder` / `ByteBuf` の typed boundary で owner flow を改善済み。`StringBuilder` 固有 raw owner field と `ByteBuf` / `ByteBuilder` の `Option<MemPtr<u8>>` owner field は削除済み。 | 短期 self-host では使用可能。 | `OwnedBytes` / `OwnedStringRegion` へ移し、unchecked raw conversion を internal boundary に閉じる。 |

現状実装の粗い集計:

- raw memory pattern が残る collection: `Vec`（raw memory boundary を持つ基礎 storage）。
- raw memory pattern が消えた主要 collection: `HashMap`, `HashSet`, `BTreeMap`, `BTreeSet`, `Queue`, `Deque`, `RingBuffer`, `Stack`, `BinaryHeap`, `SparseSet`, `BitSet`, `Fenwick`, `SegmentTree`, `DisjointSet`, `AdjacencyMatrix`, `BloomFilter`, `CountingBloomFilter`。
- enum state が明示されている主要 collection: `HashMap`, `HashSet`。
- slot state を `Option<T>` で明示している collection: `HashMap`, `HashSet`, `BTreeMap`, `BTreeSet`, `Queue`, `Deque`, `RingBuffer`, `Stack`, `BinaryHeap`, 一部 `Vec` 利用 API。

この状態での設計判断:

1. derived collection の raw header 廃止は正しい。Resource IR が raw header 内の疑似 field を復元する必要がなくなり、所有権検査の入力が構造化された。
2. `Vec<Option<T>>` は改善だが完成形ではない。`Vec` 自体は `RegionToken<T>` owner field へ移ったが、最終的な static check authority は `OwnedBuffer<T>` / initialized prefix まで進めて初めて確立する。
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
- `RegionToken<T>` は `region_new` で stdlib code から再構成できるため、compiler-issued capability ではない。
- `dealloc_ptr<T>` / `dealloc_region<T>` は storage-only free と initialized payload destruction を API 上で分けない。
- raw `i32` API と safe-ish typed API が同じ名前空間にあり、self-host 側が discipline を誤って広げやすい。

結論として、現状の `core/mem` 方針は過渡期の安全強化としては妥当だが、最終設計としては不適切である。根本修正は `MemPtr` の強化ではなく、役割分割である。

### `alloc/string`

現状の string は `[len:i32][bytes...]` layout を使い、`str` 自体は UTF-8 保証された immutable value として扱う。

改善済みの点:

- `string_alloc_region` / `string_region_data_ptr` / `string_finish` により、出力 `str` の確定前 owner を `RegionToken<u8>` に寄せている。
- `StringBuilder` は固有の `Option<MemPtr<u8>>` field を持たず、`ByteBuilder` owner を保持する wrapper になった。
- append は `ByteBuilder` の `push_*` / `push_bytes_ref` に委譲し、text builder 側では raw byte 書き込みを行わない。
- UTF-8 lead byte の分類など、一部で enum / match による分岐へ移行している。
- numeric conversion は root facade から分離済みで、integer conversion は `integer/common` が bool/digit/radix/u128 helper、`integer/format` が文字列化、`integer/parse` が解析と範囲検査を所有する形へ整理されている。float conversion も `float/format` が f64/f32 文字列化と raw output allocation、`float/parse` が小数・指数解析を所有する。

残る問題:

- `string_addr` / `string_from_addr_unchecked` は必要だが、internal boundary として compiler / source policy で閉じる必要がある。
- `string_finish_base` は `MemPtr<u8>` から `str` を作れるため、呼び出し側 discipline に依存する。将来は `OwnedStringRegion` だけが `str` へ確定できる形にする。
- `StringBuilder` 固有の raw owner field は解消済みである。残る理想形は `ByteBuilder` / `ByteBuf` 側を `OwnedBytes` / `OwnedRegion<u8>` へ移すことである。

### `alloc/io`

`ByteBuf` と `ByteBuilder` は現時点で stdlib memory model の先行実装になっている。module 責務は `alloc/io` root facade、`io/bytebuf`、`io/bytebuilder`、`io/traits` に分離済みで、raw memory capability は `io/bytebuf` と `io/bytebuilder` の exact path に限定する。

良い点:

- `ByteBuf` / `ByteBuilder` の free obligation は `RegionToken<u8>` field に集約され、`MemPtr<u8>` は参照から得る non-owning view としてだけ扱う。
- 空 storage は `size=0` の sentinel `RegionToken` で表し、解放時には allocator へ返さない。null pointer を所有 pointer field として公開しない。
- `ByteBuf` の free / to-str 変換は `RegionToken` owner を centralized cleanup へ渡す。
- `ByteBuilder` は append 成功時に payload pointer を取り出して包み直さず、owner field 全体を移す。
- stream trait 群は `io/traits` に分離され、raw memory operation を持たない抽象境界として監視される。

残る問題:

- `RegionToken<T>` はまだ stdlib code から `region_new` で構築できるため、compiler-issued capability ではない。
- empty / allocated と capacity / initialized prefix の一部は `ByteBuf` / `ByteBuilder` の struct discipline に残る。

### `alloc/collections`

collections は self-host に必要な基礎構造だが、現状は安全設計として最も再設計が必要な領域である。

現状:

- `HashMap` / `HashSet` は `Empty` / `Full` / `Tombstone` の enum と typed storage へ移行済みで、raw bucket status に戻さない source policy を持つ。
- `Queue` / `Deque` / `RingBuffer` / `Stack` / `BinaryHeap` は raw header を廃止し、`Vec<Option<T>>` storage へ移行済みである。live slot と inactive slot は `Some` / `None` で表す。
- `BTreeMap` / `BTreeSet` は sorted-array 形式の typed `Vec<Option<T>>` storage へ移行済みであり、raw key/value pointer layout ではない。
- `List` は raw node chain を廃止し、`items: Vec<T>` storage へ移行済みである。論理先頭を `Vec` 末尾に置くことで先頭追加と `tail` を owner-preserving に実装している。
- CountingBloomFilter / BitSet / AdjacencyMatrix / BloomFilter / SparseSet / Fenwick / SegmentTree / DisjointSet は `Vec<u8>` / `Vec<i32>` storage へ移行済みである。payload は主に Copy だが、基礎 `Vec` 自体はまだ raw memory owner field を持つ。
- `Vec<T>` は `len/cap/storage/region` を持ち、空 Vec は `VecStorageState::Empty` と zero-size `RegionToken<T>` で表す。型/storage/access/raw helper/transform/query/mutation の責務は submodule に分離済みで、`MemPtr<T>` は `data_mem_ptr<T>(&Vec<T>)` が参照から返す raw pointer view に限定される。ただし基礎型 `Vec` はまだ forgeable `RegionToken<T>` と Copy-only raw element helper に依存するため、owner model の完成には `OwnedBuffer<T>` が必要である。
- `get_ref<T: Copy>` のように Copy 読み取りへ制限した API はあるが、`get(Vec<T>) -> Option<T>` や `pop` などは move-out と owner state の扱いが明確でない。
- `free<T>(Vec<T>)` などの storage free は、要素の Drop / consume と storage-only dealloc を完全には分けていない。

問題:

- `MemPtr<T>` field が owner なのか borrowed view なのか、型から分からない。
- 空/非空、初期化済み prefix、未初期化 capacity、tombstone、moved-out cell が構造化されていない。
- `push(Vec<T>, T) -> Result<Vec<T>, E>` のような fallible update は、allocation failure 時に入力 collection と入力 item の owner をどう扱うかが API に現れない。
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

現在の `Vec<T> { len, cap, storage, region: RegionToken<T> }` は `MemPtr<T>` owner field を消した点では前進だが、initialized prefix / moved slot / drop obligation を型と compiler-issued token で表せないため、根本的には `OwnedBuffer<T>` へ置き換える。

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
- `VecDataLen<T>` は `Vec.data: MemPtr<T>` と `len` を public struct field として再包装するだけの raw storage view carrier だったため削除した。呼び出し側は `data_mem_ptr<T>(&Vec<T>)` と `len<T>(&Vec<T>)` を明示的に別々に使う。残件 baseline は 6 field ではなく 5 field であり、`VecDataLen.data` を transitional allowlist から外した。
- `StringBuilder` は `ByteBuilder` と重複して `Option<MemPtr<u8>>` / len / cap を持つ設計をやめ、`ByteBuilder` owner を保持する typed wrapper へ移した。残件 baseline は 5 field ではなく 4 field であり、`StringBuilder.data` を transitional allowlist から外した。
- `ByteBuf` / `ByteBuilder` は `Option<MemPtr<u8>>` owner field をやめ、storage owner を `RegionToken<u8>` field に集約した。`region_ptr` / `io_bytebuf_data_ptr_ref` / `byte_builder_data_ptr_ref` は参照から non-owning `MemPtr` view だけを返す。残件 baseline は 4 field ではなく 2 field であり、`ByteBuf.ptr` と `ByteBuilder.ptr` を transitional allowlist から外した。
- `Vec<T>` は `data: MemPtr<T>` owner field をやめ、storage owner を `region: RegionToken<T>` field に集約した。`data_mem_ptr<T>` は参照から non-owning `MemPtr<T>` view だけを返す。残件 baseline は 2 field ではなく 1 field であり、`Vec.data` を transitional allowlist から外した。

2026-05-15 追記:

- `Vec.data_ptr<T>(&Vec<T>) -> i32` は public raw address observer として残さず削除した。`data_mem_ptr<T>(&Vec<T>) -> MemPtr<T>` は typed non-owning view として残すが、raw `i32` address への変換は raw-memory-boundary 実装箇所に限定する。
- `kpsearch` の lower/upper bound / unique 内部 raw helper は private とし、公開面は `Vec<i32>` owner wrapper だけに揃えた。ordinary source の利用例は raw buffer 構築ではなく `Vec<i32>` による doctest で表す。
- `vec_storage_mem_ptr<T>(VecStorageState, &RegionToken<T>)` は public helper として残さず削除した。storage state から data view への projection は `data_mem_ptr<T>(&Vec<T>)` が直接 match して所有する。
- `Vec` の in-place sort family は storage を書き換える API なので、raw write helper、quick / heap / simple sort、raw slice sort adapter、owner-returning sort wrapper、default `sort` を impure `*>` signature へ揃えた。observer の `sort_is_sorted` と比較 helper は pure のまま残し、effect contract でも観察と破壊的更新を分離する。
- `alloc/collections/vec/sort` root facade は raw `MemPtr` helper と raw slice adapter を再公開しない。raw traversal は `sort/raw/*` に閉じ、ordinary caller は `Vec` の sort API と `sort_is_sorted` observer だけを使う。
- root `alloc/collections/vec` facade は `vec/raw` を再公開しない。unchecked `vec_read_at` / `vec_write_at` は `alloc/collections/vec/raw` を明示 import した実装境界だけに置き、通常の `Vec` import は safe public surface に限定する。
- `RegionToken<T>` の realloc は `core/mem/pointer/region.nepl` の `realloc_region_bytes_keep<T>` へ集約した。`Vec` / `ByteBuilder` は token の `ptr` / `size` を直接分解して `realloc_ptr` を呼ばず、成功時は新 owner、失敗時は旧 owner を `RegionReallocError<T>` 経由で受け取る。Resource IR owner checker は `region_ptr(&region)` から得た非所有 `MemPtr` の raw-address alias を集約値コピー越しに保持するため、`realloc_region_bytes_keep` の Ok / Err 両 payload が free obligation owner を返すことを compiler 側で証明する。
- `Vec.push` の grow capacity は `vec_next_capacity<T>` で `.T` の element size と allocator payload 上限から証明してから決定する。unchecked `cap * 2` は push hot path から外し、上限超過は `CapacityExceeded` として `VecPushError` に元 `Vec` owner を戻す。
- root `std/fs` / `std/stdio` facade は raw ABI submodule を再公開しない。WASI / LLVM syscall helper と raw scratch helper は `std/fs/raw` / `std/stdio/raw` を明示 import した implementation boundary だけに置き、通常の filesystem / standard I/O import は safe public surface に限定する。

2026-05-16 追記:

- `RegionToken<T>` は `ptr: MemPtr<T>` を持たず、`raw: i32, size: i32` の owner token layout に移行した。`MemPtr<T>` は `region_ptr<T>(&RegionToken<T>)` / `region_ptr_at<T,U>` の checked non-owning projection としてだけ使う。
- `region_new<T>` は internal boundary で `MemPtr<T>` から raw owner identity を取り出して token を構築する。通常 source は `RegionToken` constructor や raw field projectionを直接使えず、compiler owner aggregate boundary と source capability が検査する。
- Resource IR owner summary は direct raw memory op だけでなく、callee summary が消費する raw owner alias も seed する。これにより `dealloc_region<T>` が `RegionToken.raw` から一時 `MemPtr.raw` を作り `dealloc_ptr<T>` へ渡す場合も、caller では `RegionToken<T>` owner が消費されたことが証明される。
- `nodesrc/test_stdlib_memptr_owner_field_policy.js` の transitional allowlist は 0 件になった。今後 stdlib の struct field に `MemPtr` / `Option<MemPtr>` を owner-like field として戻すことは禁止する。
- ただし `RegionToken<T>` はまだ compiler-issued `OwnedRegion<T>` ではない。Stage B/D/F の最終目標は、`RegionToken` を過渡 owner token として閉じ、`OwnedBuffer<T>` / `StorageState<T>` / initialized prefix / compiler-issued free obligation owner へ進めることである。
- 追加調査で、struct field としての `MemPtr` owner は 0 件になった一方、`alloc_ptr<T> -> Result<MemPtr<T>, str>` / `realloc_ptr<T> -> Result<MemPtr<T>, str>` / `dealloc_ptr<T>(MemPtr<T>, i32)` が public API として free obligation を `MemPtr<T>` に残していることを確認した。これは [ISS-20260515T163252091Z-PUBLIC-ALLOC-PTR-APIS-STILL-ENCODE-F-EECDD686](../../issues/items/ISS-20260515T163252091Z-PUBLIC-ALLOC-PTR-APIS-STILL-ENCODE-F-EECDD686.md) として分離し、Stage B/F の実装対象にする。

### Stage B: `core/mem` の internal/public 分離

- 前提として、typecheck の import visibility が `pub` / private item boundary を binding authority として扱う必要がある。現状の blocker は [ISS-20260512T235355207Z-IMPORT-VISIBILITY-DOES-NOT-ENFORCE-P-30FB5573](../../issues/items/ISS-20260512T235355207Z-IMPORT-VISIBILITY-DOES-NOT-ENFORCE-P-30FB5573.md) で追跡する。
- raw `i32` allocator / load/store は internal raw module に移す。
- public API は `MemPtr` view、`OwnedRegion` owner、initialized cell 操作を分ける。
- `region_new` のような token forging API を廃止する。
- `dealloc_region` は storage-only region にだけ許可する。

### Stage C: string / byte buffer を owner wrapper へ移行

- `ByteBuf` / `ByteBuilder` の `Option<MemPtr<u8>>` owner field は削除済みである。次は forgeable `RegionToken<u8>` を `StorageState<u8>` または compiler-issued `OwnedBytes` に移す。
- `string_finish_base` を internal に閉じ、`OwnedStringRegion` だけが `str` を作れるようにする。
- fallback API と Result API の責務を整理し、allocation failure を空値 success に潰さない入口を標準にする。

### Stage D: `OwnedBuffer<T>` と Vec 再実装

- `Vec<T>` を `OwnedBuffer<T>` 上に再実装する。
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
| `ISS-20260427T204839136Z-STDLIB-RAW-MEMORY-BACKED-APIS-REQUIR-E503CD84` | raw-memory-backed stdlib API の段階移行親 issue。 |
| `ISS-20260514T055830236Z-VECDATALEN-CARRIES-RAW-VEC-STORAGE-V-B662D7DF` | `VecDataLen` raw storage view carrier の削除。 |
| `ISS-20260514T063755030Z-STRINGBUILDER-DUPLICATES-BYTEBUILDER-F90DFA2F` | `StringBuilder` 固有 raw owner field を `ByteBuilder` owner boundary へ集約。 |
| `ISS-20260514T071955576Z-BYTEBUF-STORES-OWNED-BYTES-AS-OPTION-FA165159` | `ByteBuf` / `ByteBuilder` の `Option<MemPtr<u8>>` owner field を `RegionToken` owner boundary へ集約。 |
| `ISS-20260514T085248522Z-VEC-STORES-BACKING-STORAGE-AS-MEMPTR-FFC9775A` | `Vec.data` raw `MemPtr` owner field を `Vec.region: RegionToken<T>` owner boundary へ集約。 |
| `ISS-20260515T141747916Z-VEC-GROW-REIMPLEMENTS-REGIONTOKEN-RE-255A043F` | `Vec` / `ByteBuilder` の `RegionToken` realloc を core/mem に集約し、Vec grow capacity overflow proof を追加。 |
| `ISS-20260515T155626605Z-REGIONTOKEN-STILL-STORES-MEMPTR-AS-O-C0F9E4D1` | `RegionToken<T>` の `MemPtr<T>` owner-like field を direct raw owner identity へ置き換え、MemPtr owner-field transitional baseline を 0 件にする。 |
| `ISS-20260515T163252091Z-PUBLIC-ALLOC-PTR-APIS-STILL-ENCODE-F-EECDD686` | public `alloc_ptr` / `realloc_ptr` / `dealloc_ptr` が `MemPtr<T>` を free obligation carrier として公開している残件。 |
| `ISS-20260515T170146857Z-CORE-MEM-POINTER-FACADE-RE-EXPORTS-L-4724AF44` | `core/mem` / `mem/pointer` safe facade から低レベル `alloc_ptr` owner wrapper の re-export を削除し、`mem_ptr_add` を non-owning view helper へ分離。 |
| `ISS-20260514T102108865Z-VEC-SORT-MERGE-RET-ERR-PATH-LOSES-CO-98B83660` | `sort_merge_ret<T>` の失敗 payload に `Vec<T>` owner を返し、merge sort scratch buffer を `RegionToken<T>` owner へ移行。 |
| `ISS-20260429T120339805Z-FALLIBLE-OWNING-COLLECTION-UPDATES-L-21EF56CB` | fallible collection update の owner loss。 |
| `ISS-20260429T131646897Z-BYTEBUF-EMPTY-NON-EMPTY-CONDITIONAL--34FBA0C2` | ByteBuf の空/所有 storage 構造化。 |
| `ISS-20260429T142213822Z-BYTEBUILDER-AND-STRINGBUILDER-RESULT-4EB1D1EB` | builder owner boundary の修正済み回帰。 |
| `ISS-20260429T155343006Z-COLLECTION-STORAGE-STATES-USE-NUMERI-E4B3A749` | collection の数値/null storage state を enum owner state へ移す新規 issue。 |
| `ISS-20260428T223953830Z-VEC-ELEMENT-LOADS-LOSE-BACKING-STORA-E811458B` | Vec backing storage と Resource IR raw cell alias の修正済み回帰。 |
| `ISS-20260429T083822053Z-SELF-HOST-DIAGNOSTICS-USE-STRING-COD-1040C21E` | self-host diagnostics も enum-first diagnostic id に合わせる。 |
| `ISS-20260513T212118060Z-MEMPTR-OWNER-FIELD-MIGRATION-LACKS-G-7E2612E2` | Stage A の `MemPtr` owner-like field 増殖禁止 source policy。 |

## 判定

現状の方向性は「過渡期としては正しいが、最終設計としては未完」である。

`ByteBuf` / `ByteBuilder` / `StringBuilder` / `Vec` は、空/所有 storage を型に出す方向へ進んだため短期利用に耐える。`StringBuilder` 固有の raw owner field と `Vec.data` raw owner field は削除済みである。一方で `core/mem` と collections は、forgeable `RegionToken` と Copy-only raw element helper にまだ依存しており、Resource IR が後追いで alias と initialized cell を復元する構造が残っている。この複雑さは設計上の警告であり、さらに特例を足して維持すべきではない。

2026-05-16 時点で、`core/mem` root と `mem/pointer` facade は低レベル `alloc_ptr` / `realloc_ptr` / `dealloc_ptr` を再公開しない。safe caller の標準経路は `alloc_region` / `region_ptr` / `dealloc_region` であり、低レベル scratch 実装は `core/mem/pointer/alloc` を明示 import する。これは最終設計ではなく、direct low-level import を `OwnedBytes` / `OwnedBuffer` / compiler-issued owner token へ置き換える前段である。

理想は、stdlib が owner state を型で表し、Resource IR がその型構造をそのまま検査できる状態である。self-host の型検査・メモリ検査を妥協しないためには、collection 再設計を避けず、`OwnedBuffer` / owner token / initialized prefix / enum state / match exhaustiveness を中核に置く。
