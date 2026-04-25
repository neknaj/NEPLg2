# DP の[基本/きほん]パターン

この章は「状態定義 -> 初期値 -> 遷移」の順で一次元 DP を固定する練習です。

## 例: 1 段 or 2 段で階段を登る通り数

- `dp[n]`: `n` 段目に到達する通り数
- 遷移: `dp[n] = dp[n-1] + dp[n-2]`
- 初期値: `dp[0] = 1`, `dp[1] = 1`

neplg2:test[stdio, normalize_newlines]
stdin: "6\n"
stdout: "13\n"
```neplg2
| #entry main
| #indent 4
| #target std
|
#import "core/math" as *
#import "core/result" as *
#import "core/cast" as *
#import "std/streamio" as *
#import "std/iotarget" as *

// 直前 2 状態だけを持つ rolling DP で通り数を更新します。
fn ways <(i32)*>i64> (n):
    if le n 1:
        then <i64> cast 1
        else:
            let mut a <i64> cast 1;
            let mut b <i64> cast 1;
            let mut i <i32> 2;
            while le i n:
                do:
                    let c <i64> add a b;
                    set a b;
                    set b c;
                    set i add i 1;
            b
|
fn main <()*> ()> ():
    let sc <StreamScanner> unwrap_ok open ReadStream::Stdio;
    let ans <i64> ways read sc;
    close sc;
    unwrap_ok open WriteStream::Stdio
    |> writeln ans
    |> flush
    |> close
```

## DP 実装時のチェックリスト

- 状態を 1 行で説明できるか。
- ループ順が遷移依存（`n-1`,`n-2`）と一致しているか。
- 境界条件（`n=0`,`n=1`）をテストで固定しているか。
