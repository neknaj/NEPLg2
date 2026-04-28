# char と ASCII

`char` は Unicode scalar value です。ASCII 判定や UTF-8 encode の補助は `core/char` にあります。`str` の byte index と `char` の値は分けて考えます。

neplg2:test
ret: 0
```neplg2
| #entry main
| #indent 4
| #target std
|
#import "alloc/string" as *
#import "core/char" as *
#import "core/result" as *
#import "std/test" as *

fn expect_char <(Result<char,str>,i32)->Result<(),str>> (got, expected_code):
    match got:
        Result::Ok c:
            check_eq_i32 expected_code char_to_i32 c
        Result::Err msg:
            Result<(),str>::Err msg

fn main <()*>i32> ():
    let a <char> 'A'
    let hira <char> 'あ'
    let checks <Vec<Result<(),str>>>:
        checks_new
        |> checks_push check_eq_i32 65 char_to_i32 a
        |> checks_push check char_is_ascii_alpha a
        |> checks_push check char_is_ascii_digit '7'
        |> checks_push check char_is_ascii_whitespace '\n'
        |> checks_push check_eq_i32 1 char_utf8_len a
        |> checks_push check_eq_i32 3 char_utf8_len hira
        |> checks_push expect_char str_char_at_result "Aあ" 1 0x3042
    checks_exit_code checks
```

ASCII だけを受け付ける処理では `char_is_ascii_*` を使います。任意の text を 1 byte として扱うのは誤りです。
