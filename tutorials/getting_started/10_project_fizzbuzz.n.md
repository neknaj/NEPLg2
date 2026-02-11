# ミニプロジェクト: FizzBuzz

ここでは小さな実践として FizzBuzz を実装します。
複数条件の分岐を `if` 式で積み上げる練習です。

## FizzBuzz の核となる関数

neplg2:test[stdio, normalize_newlines]
stdout: "6 -> Fizz\n10 -> Buzz\n30 -> FizzBuzz\n7 -> 7\n"
```neplg2
| #entry main
| #indent 4
| #target wasi
|
#import "core/math" as *
#import "std/stdio" as *

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

fn show_line <(i32)*>()> (n):
    let code <i32> fizzbuzz_code n;
    print_i32 n;
    print " -> ";
    if:
        cond eq code 1
        then println "Fizz"
        else:
            if:
                cond eq code 2
                then println "Buzz"
                else:
                    if:
                        cond eq code 3
                        then println "FizzBuzz"
                        else println_i32 n

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
| #target wasi
|
#import "core/math" as *
#import "std/stdio" as *

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

fn print_fizzbuzz_1_to_n <(i32)*>()> (n):
    let mut i <i32> 1;
    while le i n:
        do:
            let code <i32> fizzbuzz_code i;
            if:
                cond eq code 1
                then println "Fizz"
                else:
                    if:
                        cond eq code 2
                        then println "Buzz"
                        else:
                            if:
                                cond eq code 3
                                then println "FizzBuzz"
                                else println_i32 i;
            set i add i 1;

fn main <()*> ()> ():
    print_fizzbuzz_1_to_n 15
```
