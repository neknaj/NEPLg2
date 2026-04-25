# テスト[駆動/くどう]で[関数/かんすう]を[固/かた]める

実装を進めるときは、最初に「壊したくない挙動」を `std/test` で固定してから実装を調整すると安全です。

この章では小さな関数を例に、入力ケースを増やして仕様を固定する流れを示します。

## 仕様を先にテストで固定する

neplg2:test
ret: 0
```neplg2
| #entry main
| #indent 4
| #target std
|
#import "core/math" as *
#import "core/result" as *
#import "std/test" as *

// 負の値だけ符号を反転し、絶対値を返します。
fn abs_i32 <(i32)->i32> (x):
    if lt x 0 then sub 0 x else x

fn main <()*>i32> ():
    // 典型値と境界値を先に固定し、仕様を崩さないようにします。
    let checks <Vec<Result<(),str>>>:
        checks_new
        |> checks_push assert_eq_i32 0 abs_i32 0
        |> checks_push assert_eq_i32 8 abs_i32 8
        |> checks_push assert_eq_i32 8 abs_i32 -8
    let shown <Vec<Result<(),str>>> checks_print_report checks
    checks_exit_code shown
```

## 失敗時の読みやすい出力

`check_*` や `finish_checks` は `Result<(),str>` を返すので、`checks_exit_code` や `result_exit_code` で `main` の戻り値へ落とします。`Vec<Result<(),str>>` の[表示/ひょうじ]は[自動/じどう]ではなく、test [末尾/まつび]で `checks_print_report` を[明示的/めいじてき]に[呼/よ]びます。

neplg2:test[stdio, normalize_newlines, strip_ansi]
ret: 0
stdout: "Checked [ok,ok]\n[0] ok\n[1] ok\n"
```neplg2
| #entry main
| #indent 4
| #target std
|
#import "std/test" as *
#import "core/result" as *

fn main <()*>i32> ():
    // 判定結果を一覧化してから終了コードへ落とします。
    let checks <Vec<Result<(),str>>>:
        checks_new
        |> checks_push Result<(),str>::Ok ()
        |> checks_push Result<(),str>::Ok ()
    let shown <Vec<Result<(),str>>> checks_print_report checks
    checks_exit_code shown
```

## テスト追加の実務手順

1. 先に最小ケースを 1 つ追加する（赤になることを確認）。
2. 実装を最小変更で通す。
3. 境界ケースを 1 つずつ増やし、失敗時に原因が切り分けられる粒度を保つ。

一度に大量ケースを追加すると、失敗原因の特定コストが急に上がるため、差分を小さく保つのが重要です。
