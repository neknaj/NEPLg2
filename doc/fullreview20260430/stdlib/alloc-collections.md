# stdlib alloc collections review

確認対象 commit: `b350213c docs(review): add selfhost compiler review`

## 確認対象

- `stdlib/alloc/collections/vec.nepl`
- `stdlib/alloc/collections/vec/**`
- `stdlib/alloc/collections/{hashmap,hashset}.nepl`
- `stdlib/alloc/collections/{hashmap,hashset}/**`
- `stdlib/alloc/collections/{list,stack,queue,deque,ringbuffer,binary_heap}.nepl`
- `stdlib/alloc/collections/{btreemap,btreeset}.nepl`
- `stdlib/alloc/collections/{bitset,bloom_filter,counting_bloom_filter,adjacency_matrix,fenwick,segment_tree,disjoint_set,sparse_set}.nepl`

## 良い点

`Vec` は facade と `types/storage/access/raw/mutation/query/transform/sort` に分割されている。`VecStorageState` は `Empty` / `Owned` enum で、null pointer sentinel を owner state に使わない。

HashMap/HashSet は bucket state を enum (`Empty` / `Full` / `Tombstone`) として持ち、storage/probe/rehash/api に分かれている。numeric sentinel ではなく match で扱える設計になっている。

Stack/Queue/Deque/RingBuffer/BinaryHeap/BTreeMap/BTreeSet/List は raw header や raw node pointer から、`Vec<Option<T>>` や `Vec<T>` などの typed storage へかなり移行している。List も raw node chain ではなく Vec storage を持つ。

DisjointSet/Fenwick/SegmentTree/BitSet/AdjacencyMatrix/BloomFilter は Copy payload 中心だが、update error owner を返す API や borrowed observers が増えている。

## 問題とリスク

P1 open issue の通り、collection free/drop contract は未完である。`Vec<T>` や `HashMap<K,V>` に owning/non-Copy payload を入れたとき、container free が要素 Drop をどう呼ぶか、remove/pop が owner をどう返すか、storage-only dealloc と element cleanup をどう分けるかが未確定である。

多くの collection が `.T: Copy` を要求して問題を回避している。これは現時点では正直な制約だが、selfhost AST/HIR/diagnostic では non-Copy payload が必要になるため、長期的な解決にはならない。

BTreeMap/BTreeSet は名前に反して sorted array 実装で、doc では明記済みである。大きい mutable table や compiler symbol table に使うと O(n) update が問題になる。既存 issue は resolved だが、selfhost で誤用しないよう review で再確認する。

List には by-value observer が残る。現実には owner を閉じる terminal API として設計されているが、borrowed observer へ統一する余地がある。現時点では collection free/drop parent issue の一部として扱う。

## 進捗状況

| 領域 | 状態 | 判定 |
|---|---|---|
| `Vec` | module split + storage enum。 | 良い。element drop contractは未完。 |
| `HashMap` / `HashSet` | bucket state enum + storage分割。 | 良い。non-Copy value/dropは未完。 |
| `BTreeMap` / `BTreeSet` | sorted-array implementation。 | 小規模/安定順用。compiler大規模表には不向き。 |
| `List` | Vec-backed storage。 | raw node廃止は良い。observer/drop contractは残る。 |
| Stack/Queue/Deque/RingBuffer/BinaryHeap | typed slot storage。 | Copy payload中心なら有用。 |
| graph/numeric collections | Vec/Copy storage中心。 | selfhost symbol tableにはHashMap優先。 |

## 推奨対応

- collection API を `Copy read`、`borrowed read`、`owned remove/pop`、`container drop`、`storage-only dealloc` に分ける。
- `Drop<T>` bound と ResourceIR drop obligation を前提に、non-Copy payload の container free を設計する。
- selfhost compiler の table は BTree sorted-array ではなく HashMap/HashSet と最終 sort の組み合わせを基本にする。
- source policy は raw header/raw pointer sentinel 再導入だけでなく、by-value observer の再拡大も監視する。
