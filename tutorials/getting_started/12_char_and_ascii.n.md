# char と ASCII

`char` は Unicode scalar value です。ASCII 判定や UTF-8 encode の補助は `core/char` にあります。`str` の byte index と `char` の値は分けて考えます。

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: mlstr:
    ##: Checked [ok,ok,ok,ok,ok,ok,ok]
    ##: [0] ok
    ##: [1] ok
    ##: [2] ok
    ##: [3] ok
    ##: [4] ok
    ##: [5] ok
    ##: [6] ok
```neplg2
| #entry main
| #indent 4
| #target std
|
#import "alloc/string" as *
#import "core/char" as *
#import "core/result" as *
#import "std/test" as *

fn expect_char %fn Result char str fn i32 Result () str \got\expected_code:
    match got:
        Result::Ok c:
            check_eq_i32 expected_code char_to_i32 c
        Result::Err msg:
            Result::Err msg

fn main %impure fn () i32 \():
    let a %char 'A'
    let hira %char 'あ'
    let checks:
        checks_new
        |> checks_push assert_eq_i32 65 char_to_i32 a
        |> checks_push assert char_is_ascii_alpha a
        |> checks_push assert char_is_ascii_digit '7'
        |> checks_push assert char_is_ascii_whitespace '\n'
        |> checks_push assert_eq_i32 1 char_utf8_len a
        |> checks_push assert_eq_i32 3 char_utf8_len hira
        |> checks_push expect_char str_char_at_result "Aあ" 1 0x3042
    let shown checks_print_report checks
    checks_exit_code shown
```

ASCII だけを受け付ける処理では `char_is_ascii_*` を使います。任意の text を 1 byte として扱うのは誤りです。
