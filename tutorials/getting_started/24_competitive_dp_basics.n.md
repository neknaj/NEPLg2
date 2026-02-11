# DP の基本パターン

この章では、競プロで頻出の一次元 DP を最小コードで扱います。
ポイントは「状態定義」と「遷移式」を先に固定することです。

## 例: 1 段 or 2 段で階段を登る通り数

- `dp[n]` = `n` 段目に到達する通り数
- 遷移: `dp[n] = dp[n-1] + dp[n-2]`
- 初期値: `dp[0] = 1`, `dp[1] = 1`

neplg2:test[stdio, normalize_newlines]
stdin: "6\n"
stdout: "13\n"
```neplg2
| #entry main
| #indent 4
| #target wasi
|
#import "core/math" as *
#import "kp/kpread" as *
#import "kp/kpwrite" as *

fn ways <(i32)*>i64> (n):
    if le n 1:
        then i64_extend_i32_u 1
        else:
            let mut a <i64> i64_extend_i32_u 1;
            let mut b <i64> i64_extend_i32_u 1;
            let mut i <i32> 2;
            while le i n:
                do:
                    let c <i64> i64_add a b;
                    set a b;
                    set b c;
                    set i add i 1;
            b
|
fn main <()*> ()> ():
    let sc <i32> scanner_new;
    let n <i32> scanner_read_i32 sc;
    let ans <i64> ways n;
    let w <i32> writer_new;
    writer_write_i64 w ans;
    writer_writeln w;
    writer_flush w;
    writer_free w
```

## DP 実装時のチェックリスト

- 状態が何を表すかを 1 行で言えるか。
- 遷移の依存関係がループ順に合っているか。
- 初期値（境界条件）をテストで固定しているか。
