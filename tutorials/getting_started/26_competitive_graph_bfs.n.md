# グラフ探索（BFS）

競プロでは、無向グラフの最短距離（辺重み 1）が頻出です。
この章では BFS（幅優先探索）で `dist` を計算する最小例を扱います。

## 例: 0-1-2-3 の線形グラフ

`0` からの最短距離は `[0, 1, 2, 3]` になります。

neplg2:test[stdio, normalize_newlines]
stdout: "0 1 2 3\n"
```neplg2
| #entry main
| #indent 4
| #target wasi
|
#import "core/mem" as *
#import "core/math" as *
#import "std/stdio" as *

fn bfs_line4 <(i32)*>()> (dist):
    // dist を -1 で初期化
    store_i32 add dist 0 -1;
    store_i32 add dist 4 -1;
    store_i32 add dist 8 -1;
    store_i32 add dist 12 -1;

    // キュー（最大 4 要素）
    let q <i32> alloc 16;
    let mut head <i32> 0;
    let mut tail <i32> 0;

    // 始点 0
    store_i32 add dist 0 0;
    store_i32 add q mul tail 4 0;
    set tail add tail 1;

    while lt head tail:
        do:
            let v <i32> load_i32 add q mul head 4;
            set head add head 1;

            // 線形グラフなので隣接は v-1, v+1（範囲内）
            if lt 0 v:
                then:
                    let to <i32> sub v 1;
                    let to_ptr <i32> add dist mul to 4;
                    if eq load_i32 to_ptr -1:
                        then:
                            let dv <i32> load_i32 add dist mul v 4;
                            store_i32 to_ptr add dv 1;
                            store_i32 add q mul tail 4 to;
                            set tail add tail 1;
                        else ();
                else ();

            if lt v 3:
                then:
                    let to <i32> add v 1;
                    let to_ptr <i32> add dist mul to 4;
                    if eq load_i32 to_ptr -1:
                        then:
                            let dv <i32> load_i32 add dist mul v 4;
                            store_i32 to_ptr add dv 1;
                            store_i32 add q mul tail 4 to;
                            set tail add tail 1;
                        else ();
                else ();

    dealloc q 16
|
fn main <()*> ()> ():
    let dist <i32> alloc 16;
    bfs_line4 dist;
    print_i32 load_i32 add dist 0;
    print " ";
    print_i32 load_i32 add dist 4;
    print " ";
    print_i32 load_i32 add dist 8;
    print " ";
    println_i32 load_i32 add dist 12;
    dealloc dist 16
```

## BFS 実装時のチェックリスト

- `dist == -1` を未訪問として使うかどうかを最初に決める。
- キュー `head/tail` の更新順を固定する。
- 有向/無向で辺の張り方（追加方向）が変わる点を確認する。

## 実戦での発展

- 頂点数が小〜中規模なら `kp/kpgraph` の補助 API を使うと実装を短くできます。
- 大規模グラフでは密行列ではなく隣接リスト表現へ切り替える設計が必要です。
- まずはこの章の「手書き BFS」を基準実装として持ち、問題ごとに入力形式だけ差し替える運用が安定します。
