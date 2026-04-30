# stdlib alloc collections review

対象 commit: `f108cebd`

参照 Actions: `25157230630`

## 概要

collections は selfhost に必要だが、現時点で最も慎重に扱うべき領域である。HashMap / HashSet / Stack / Queue / Deque / RingBuffer / BinaryHeap / BTreeMap / BTreeSet は raw header から typed storage へかなり移行した。一方で、基礎型 `Vec<T>`、raw node `List<T>`、byte/numeric raw storage collection はまだ `MemPtr` owner model に依存する。

## Actions で確認した状態

`stdlib-test` artifact の collection 関連失敗は 73 件。主な内訳は次の通り。

- `Vec` doctest: owner-bearing result / storage owner leak。
- `List` doctest: raw node owner leak と map/filter accumulator may leak。
- `HashMap` / `HashSet` doctest: collection 本体に入る前に `from_f64_result` の `resource.cell.possibly_moved` が表面化。
- `vec/sort`: raw sort helper と Vec storage owner の境界。
- `stdlib/tests/*`: external tests が std/test / collection cleanup 問題を拾う。

## 良い点

- `HashMapBucketState` / `HashSetBucketState` は enum で、0/1/2 sentinel から脱却した。
- HashMap / HashSet は `Vec<HashMapBucketState>` と `Vec<Option<K/V>>` で live slot を型に出している。
- Stack / Queue / Deque / RingBuffer / BinaryHeap は `Vec<Option<T>>` storage に移行し、inactive slot を `None` として扱う。
- `Vec` の read-only observer は `&Vec<T>` に寄せられ、by-value observer から前進している。

## 残る問題

### `Vec<T>` が未完の根

`Vec<T>` は `len/cap/data: MemPtr<T>` であり、empty は `mem_ptr_wrap 0` に依存する。`MemPtr<T>` は non-owning pointer であるべきだが、`Vec.data` では storage owner field になっている。これが derived collection 全体へ波及する。

必要な設計は `OwnedBuffer<T>` + initialized prefix である。

### `List<T>` は raw node chain

`List<T>` は raw node address を持つ。`reverse` / `map` / `filter` の owner flow は改善されているが、node owner wrapper がない限り Resource IR は raw cell owner を追い続ける必要がある。

### fallible update の owner contract

`push` / `insert` / `rehash` / `map` / `filter` などの fallible update は、失敗時に入力 collection と入力 item owner をどう扱うかを型で表す必要がある。`Result<Vec<T>, E>` だけでは non-Copy payload の所有権が不足する。

## selfhost への示唆

短期 selfhost では、`Vec<i32>` / `Vec<char>` / `Vec<TokenId>` のような Copy payload に限定する。`Vec<str>`、`Vec<Diagnostic>`、`HashMap<str, ...>` のような owning payload collection は、`OwnedBuffer` と owner-preserving failure result が入るまで中核データ構造にしない。
