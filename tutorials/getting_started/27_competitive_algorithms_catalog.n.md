# [競/きょう]プロ[定番/ていばん] 20 サンプル[集/しゅう]

※未完成
この章は、競技プログラミングで頻出のアルゴリズム・データ構造を「どこで使うか」と「最小コード雛形」で素早く参照するためのカタログです。  
ここに載せたコードは、問題ごとに入力部と境界条件を差し替えて使う前提です。

関連ライブラリ:
- `kp/kpread`, `kp/kpwrite`: 高速入出力
- `kp/kpprefix`: 累積和
- `kp/kpsearch`: 二分探索
- `kp/kpgraph`: BFS（密行列表現）
- `kp/kpdsu`: Union-Find
- `kp/kpfenwick`: BIT

## 1. 高速入力（整数 2 個）

```neplg2
#import "kp/kpread" as *
let sc <i32> scanner_new;
let a <i32> scanner_read_i32 sc;
let b <i32> scanner_read_i32 sc;
```

## 2. 高速出力（空白区切り）

```neplg2
#import "kp/kpwrite" as *
let w <i32> writer_new;
writer_write_i32 w 10;
writer_write_space w;
writer_write_i32 w 20;
writer_writeln w;
writer_flush w;
writer_free w;
```

## 3. 累積和（1D）

```neplg2
#import "kp/kpprefix" as *
let pref <i32> prefix_build_i32 data n;
let s <i32> prefix_range_sum_i32 pref l r;
```

## 4. 2D 累積和（雛形）

```neplg2
// pref[y+1][x+1] = a[y][x] + pref[y][x+1] + pref[y+1][x] - pref[y][x]
```

## 5. いもす法（差分配列）

```neplg2
// diff[l] += x; diff[r] -= x; 最後に prefix を取る
```

## 6. lower_bound

```neplg2
#import "kp/kpsearch" as *
let i <i32> lower_bound_i32 data len x;
```

## 7. upper_bound

```neplg2
#import "kp/kpsearch" as *
let j <i32> upper_bound_i32 data len x;
```

## 8. 値が存在するか（binary search）

```neplg2
#import "kp/kpsearch" as *
if contains_i32 data len x then ... else ...
```

## 9. 尺取り法（two pointers）

```neplg2
let mut l <i32> 0;
let mut r <i32> 0;
while lt l n:
    do:
        while and lt r n cond:
            do: set r add r 1;
        // [l, r) を処理
        set l add l 1;
```

## 10. 座標圧縮（雛形）

```neplg2
// 値を配列に集める -> sort -> unique -> lower_bound で圧縮ID化
```

## 11. BFS（単一始点最短距離）

```neplg2
#import "kp/kpgraph" as *
let dist <Vec<i32>> dense_graph_bfs_dist_raw n mat start;
```

## 12. DFS（再帰）

```neplg2
fn dfs <(i32)*>()> (v):
    // 訪問処理
    // 子へ再帰
```

## 13. Union-Find（連結判定）

```neplg2
#import "kp/kpdsu" as *
let d <i32> dsu_new n;
dsu_unite d u v;
if dsu_same d a b then ... else ...
```

## 14. Fenwick Tree（BIT）

```neplg2
#import "kp/kpfenwick" as *
let f <i32> fenwick_new n;
fenwick_add f idx delta;
let ans <i32> fenwick_sum_range f l r;
```

## 15. セグメント木（雛形）

```neplg2
// point update / range query を O(log N)
```

## 16. Dijkstra（雛形）

```neplg2
// dist 配列と優先度付きキューで最短路
```

## 17. トポロジカルソート（雛形）

```neplg2
// 入次数 0 キューを使って DAG を線形順序化
```

## 18. mod べき乗（繰り返し二乗法）

```neplg2
fn mod_pow <(i64,i64,i64)->i64> (a, e, m):
    let mut x <i64> a;
    let mut k <i64> e;
    let mut r <i64> i64_extend_i32_u 1;
    while i64_lt_u i64_extend_i32_u 0 k:
        do:
            if i64_eq i64_and k i64_extend_i32_u 1 i64_extend_i32_u 1:
                then set r i64_rem_u i64_mul r x m
                else ();
            set x i64_rem_u i64_mul x x m;
            set k i64_shr_u k i64_extend_i32_u 1;
    r
```

## 19. 組合せ前計算（雛形）

```neplg2
// fact, inv_fact を前計算して nCk を O(1) で求める
```

## 20. DP（一次元/二次元）

```neplg2
// 状態定義 -> 初期値 -> 遷移 -> ループ順序を固定
```

## 使い方の目安

- まず 1〜3 と 6〜9 を安定化すると、多くの A〜D 問題に対応しやすくなります。
- 11, 13, 14 はグラフ/データ構造問題の基礎セットです。
- 16〜20 は問題依存の実装差が大きいため、雛形を元に逐次調整する運用が安全です。
