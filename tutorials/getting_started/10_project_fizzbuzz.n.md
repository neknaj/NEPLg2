# ミニプロジェクト: FizzBuzz

ここでは小さな実践として FizzBuzz を実装します。
複数条件の分岐を `if` 式で積み上げる練習です。

## FizzBuzz の核となる関数

neplg2:test[stdio, normalize_newlines]
stdout: "6 -> Fizz\n10 -> Buzz\n30 -> FizzBuzz\n7 -> 7\n"
```neplg2
| #entry main
| #indent 4
| #target std
|
#import "core/math" as *
#import "std/stdio" as *
#import "alloc/string" as *

// 1 つの値を FizzBuzz 表記へ変換して表示します。
fn show_line <(i32)*>()> (n):
    print_i32 n;
    print " -> ";
    println <str> if:
        cond eq mod_s n 15 0
        then "FizzBuzz"
        else:
            if:
                cond eq mod_s n 3 0
                then "Fizz"
                else:
                    if:
                        cond eq mod_s n 5 0
                        then "Buzz"
                        else from_i32 n

fn main <()*> ()> ():
    show_line 6;
    show_line 10;
    show_line 30;
    show_line 7
```

## 標準出力に結果を表示する

neplg2:test[stdio, normalize_newlines]
stdout: "1\n2\nFizz\n4\nBuzz\nFizz\n7\n8\nFizz\nBuzz\n11\nFizz\n13\n14\nFizzBuzz\n"
```neplg2
| #entry main
| #indent 4
| #target std
|
#import "core/math" as *
#import "std/stdio" as *
#import "alloc/string" as *

// 1 から n までを順に判定し、その場で標準出力へ流します。
fn print_fizzbuzz_1_to_n <(i32)*>()> (n):
    let mut i <i32> 1;
    while le i n:
        do:
            println <str> if:
                cond eq mod_s i 15 0
                then "FizzBuzz"
                else:
                    if:
                        cond eq mod_s i 3 0
                        then "Fizz"
                        else:
                            if:
                                cond eq mod_s i 5 0
                                then "Buzz"
                                else from_i32 i
            set i add i 1;

fn main <()*> ()> ():
    print_fizzbuzz_1_to_n 15
```
