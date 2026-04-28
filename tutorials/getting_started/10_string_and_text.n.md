# 文字列と text

`str` は UTF-8 text です。`len` や `str_byte_len` は byte 数を返し、`str_char_count` は Unicode scalar value としての `char` 数を数えます。

neplg2:test
ret: 0
```neplg2
| #entry main
| #indent 4
| #target std
|
#import "alloc/string" as *
#import "core/result" as *
#import "std/test" as *

fn expect_str_ok <(Result<str,str>,str)->Result<(),str>> (got, expected):
    match got:
        Result::Ok text:
            check_str_eq expected text
        Result::Err msg:
            Result<(),str>::Err msg

fn main <()*>i32> ():
    let text <str> "Aあ"
    let checks:
        checks_new
        |> checks_push check_eq_i32 4 str_byte_len text
        |> checks_push check_eq_i32 2 str_char_count text
        |> checks_push expect_str_ok str_slice_chars_result text 1 2 "あ"
        |> checks_push check is_err<str,str> str_slice_result text 2 3
    checks_exit_code checks
```

byte offset で切る API は UTF-8 boundary を守る必要があります。利用者向けの text 処理では、可能なら `str_slice_chars_result` のような char index API を使います。
