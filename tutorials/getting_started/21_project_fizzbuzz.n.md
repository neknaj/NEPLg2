# Project: FizzBuzz

小さな project では、純粋な判定関数を先に作り、I/O は最後に分けます。ここでは `str` を返す関数として FizzBuzz をテストします。

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: mlstr:
    ##: Checked [ok,ok,ok,ok]
    ##: [0] ok
    ##: [1] ok
    ##: [2] ok
    ##: [3] ok
```neplg2
| #entry main
| #indent 4
| #target std
|
#import "core/result" as *
#import "std/test" as *
#import "core/math" as *

fn fizzbuzz_word <(i32)->str> (n):
    if:
        eq rem_s n 15 0
        then:
            "FizzBuzz"
        else:
            if:
                eq rem_s n 3 0
                then:
                    "Fizz"
                else:
                    if:
                        eq rem_s n 5 0
                        then:
                            "Buzz"
                        else:
                            "Number"

fn main <()*>i32> ():
    let checks:
        checks_new
        |> checks_push assert_str_eq "Number" fizzbuzz_word 1
        |> checks_push assert_str_eq "Fizz" fizzbuzz_word 3
        |> checks_push assert_str_eq "Buzz" fizzbuzz_word 5
        |> checks_push assert_str_eq "FizzBuzz" fizzbuzz_word 15
    let shown checks_print_report checks
    checks_exit_code shown
```

stdout へ出す版は `std/stdio` を import して `println` します。テストしやすい中心ロジックは、まず純粋な関数として残します。
