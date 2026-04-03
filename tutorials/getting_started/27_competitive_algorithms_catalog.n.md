# [競/きょう]プロ[定番/ていばん]カタログ（Part 6 [総まとめ/そうまとめ]）

この章は、Part 6（22〜26）で使った実装パターンを、問題投入前に見返せる形で整理した実戦メモです。  
ポイントは「短いが安全」「標準ライブラリ優先」「`std/streamio` で入出力の雛形を固定」の 3 つです。

## まず使うテンプレート

neplg2:test[stdio, normalize_newlines]
stdin: "3 10 20 30\n"
stdout: "60\n"
```neplg2
| #entry main
| #indent 4
| #target std
|
#import "core/math" as *
#import "core/result" as *
#import "std/streamio" as *
#import "std/iotarget" as *
#import "alloc/collections/vec" as *
#import "core/mem" as *

fn main <()*> ()> ():
    let sc <StreamScanner> unwrap_ok open ReadStream::Stdio;
    let n <i32> read sc;
    let mut a <Vec<i32>> unwrap_ok new<i32>;
    let mut i <i32> 0;
    while lt i n:
        do:
            set a unwrap_ok push a read sc;
            set i add i 1;
    let mut sum <i32> 0;
    let s <VecDataLen<i32>> data_len<i32> a;
    let p <i32> mem_ptr_addr get s "data";
    let len <i32> get s "len";
    let mut j <i32> 0;
    while lt j len:
        do:
            set sum add sum load_i32 add p mul j 4;
            set j add j 1;
    close sc;
    unwrap_ok open WriteStream::Stdio
    |> writeln sum
    |> flush
    |> close
```

## 典型パターンと対応ライブラリ

### 入出力

- 入力: `std/streamio` の `open`, `read`, `close`
- 出力: `std/streamio` の `open`, `write`, `writeln`, `flush`, `close`

### 配列・ソート・探索

- 配列構築: `alloc/collections/vec` の `new`, `push`, `len`, `get`
- ソート: `alloc/collections/vec/sort` の `sort_quick_ret`
- 二分探索: `kp/kpsearch` の `lower_bound_vec_i32`, `upper_bound_vec_i32`, `count_equal_range_vec_i32`

### 区間和・2 ポインタ

- 累積和: `kp/kpprefix` の `prefix_build_vec_i32`, `prefix_sum_i32`, `prefix_free_i32`
- 2 ポインタ: `Vec` + `while` で `l,r` を単調増加させる

### グラフ

- 密行列 BFS: `kp/kpgraph` の `dense_graph_new`, `dense_graph_add_undirected`, `dense_graph_bfs_dist_raw`
- 入力形式付き構築: `dense_graph_read_undirected_1indexed`

### データ構造

- Union-Find: `kp/kpdsu`
- Fenwick Tree: `kp/kpfenwick`

## 解法フロー（提出前チェック）

1. 入力を `StreamScanner` で読み切る（型は i32 / i64 を先に確定）。
2. 状態を `Vec` や専用構造へ入れる（手書きヒープ操作を避ける）。
3. 本体ロジックを `sort` / `search` / `prefix` / `graph` 補助で短く保つ。
4. 出力を `StreamWriter` でまとめて flush する。
5. 境界ケース（空配列、1要素、同値多数、最大値付近）を `neplg2:test` で固定する。

## 20 テーマの使い分けメモ

1. 高速入力: `read`
2. 高速出力: `write` / `writeln`
3. 1D 累積和: `prefix_build_vec_i32`
4. 2D 累積和: 問題ごとに実装（行列サイズ注意）
5. いもす法: 差分配列 + 累積和
6. lower_bound: `lower_bound_vec_i32`
7. upper_bound: `upper_bound_vec_i32`
8. 存在判定: `contains_vec_i32`
9. 尺取り法: `l,r` 単調増加
10. 座標圧縮: sort + unique + lower_bound
11. BFS: `dense_graph_bfs_dist_raw`（小〜中規模）
12. DFS: 再帰深さに注意
13. Union-Find: `kp/kpdsu`
14. BIT: `kp/kpfenwick`
15. セグ木: 問題別モノイド設計
16. Dijkstra: 優先度付きキュー
17. トポソ: 入次数管理
18. mod べき乗: 繰り返し二乗法
19. 組合せ前計算: `fact/inv_fact`
20. DP: 状態・遷移・初期値を固定

Part 6 の目的は「全部を暗記すること」ではなく、  
「問題文を見たらどの型・どの補助 API を使うかをすぐ選べる状態」にすることです。
