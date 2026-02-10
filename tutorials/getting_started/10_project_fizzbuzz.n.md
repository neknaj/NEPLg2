# ミニプロジェクト: FizzBuzz

ここでは小さな実践として FizzBuzz を実装します。
複数条件の分岐を `if` 式で積み上げる練習です。

## FizzBuzz の核となる関数

neplg2:test
```neplg2
| #entry main
| #indent 4
| #target wasi
|
| #import "core/math" as *
| #import "std/test" as *
|
fn fizzbuzz_code <(i32)->i32> (n):
    if:
        cond eq mod_s n 15 0
        then 3
        else:
            if:
                cond eq mod_s n 3 0
                then 1
                else:
                    if:
                        cond eq mod_s n 5 0
                        then 2
                        else 0

fn main <()*> ()> ():
    assert_eq_i32 1 fizzbuzz_code 6
    assert_eq_i32 2 fizzbuzz_code 10
    assert_eq_i32 3 fizzbuzz_code 30
    assert_eq_i32 0 fizzbuzz_code 7
    test_checked "fizzbuzz core"
```

## 標準出力に結果を表示する

neplg2:test[stdio, normalize_newlines]
stdout: "1 2 0 3\n"
```neplg2
| #entry main
| #indent 4
| #target wasi
|
| #import "core/math" as *
| #import "std/stdio" as *
|
fn fizzbuzz_code <(i32)->i32> (n):
    if:
        cond eq mod_s n 15 0
        then 3
        else:
            if:
                cond eq mod_s n 3 0
                then 1
                else:
                    if:
                        cond eq mod_s n 5 0
                        then 2
                        else 0

fn main <()*> ()> ():
    print_i32 fizzbuzz_code 3;
    print " ";
    print_i32 fizzbuzz_code 5;
    print " ";
    print_i32 fizzbuzz_code 7;
    print " ";
    println_i32 fizzbuzz_code 15;
```
