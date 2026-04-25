# グラフ[探索/たんさく]（BFS）

辺重み 1 の最短距離は BFS で求めます。  
この章は手書きキューではなく `kp/kpgraph` を使って、実装量を減らします。

## 例: 線形グラフ 0-1-2-3

`0` からの距離は `[0, 1, 2, 3]` です。

neplg2:test[stdio, normalize_newlines]
stdout: "0 1 2 3\n"
```neplg2
| #entry main
| #indent 4
| #target std
|
#import "core/math" as *
#import "core/field" as *
#import "alloc/collections/vec" as *
#import "kp/kpgraph" as *
#import "std/stdio" as *

// 距離配列を空白区切りで整形して表示します。
fn print_dist <(Vec<i32>)*>()> (dist):
    let span <VecDataLen<i32>> data_len<i32> dist;
    let n <i32> get span "len";
    let data <i32> mem_ptr_addr get span "data";
    let mut i <i32> 0;
    while lt i n:
        do:
            if lt 0 i:
                then print " "
                else ()
            print_i32 load_i32 add data mul i 4;
            set i add i 1;
    println ""
|
fn main <()*> ()> ():
    // 線形グラフを構築し、0 番からの BFS 距離をまとめて出力します。
    let g <DenseGraph> dense_graph_new 4;
    let g_mem <i32> alloc_raw size_of<DenseGraph>;
    store<DenseGraph> g_mem g;
    let n <i32> get load<DenseGraph> g_mem "n";
    let mat <i32> get load<DenseGraph> g_mem "mat";
    dense_graph_add_undirected DenseGraph n mat 0 1;
    dense_graph_add_undirected DenseGraph n mat 1 2;
    dense_graph_add_undirected DenseGraph n mat 2 3;
    let dist <Vec<i32>> dense_graph_bfs_dist_raw n mat 0;
    print_dist dist;
    dense_graph_free DenseGraph n mat
```

## 実戦での使い分け

- 頂点数が小〜中規模なら `kp/kpgraph` の密行列表現で十分です。
- 大規模入力では隣接リスト実装（`O(N+M)`）へ切り替えるのが基本です。
- まずはこの章の形で BFS の流れを固定し、入力部だけ差し替える運用にすると安定します。
