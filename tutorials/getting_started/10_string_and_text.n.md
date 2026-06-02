# 文字列と text

`str` は UTF-8 text です。`len` や `str_byte_len` は byte 数を返し、`str_char_count` は Unicode scalar value としての `char` 数を数えます。

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
#import "alloc/string" as *
#import "core/result" as *
#import "std/test" as *

fn expect_str_ok %fn Result str str fn str Result unit str \got\expected:
    match got:
        Result::Ok text:
            check_str_eq expected text
        Result::Err msg:
            Result::Err msg

fn main %impure fn void i32 \void:
    let text %str "Aあ"
    let checks:
        checks_new
        |> checks_push assert_eq_i32 4 str_byte_len text
        |> checks_push assert_eq_i32 2 str_char_count text
        |> checks_push expect_str_ok str_slice_chars_result text 1 2 "あ"
        |> checks_push assert is_err str_slice_result text 2 3
    let shown checks_print_report checks
    checks_exit_code shown
```

byte offset で切る API は UTF-8 boundary を守る必要があります。利用者向けの text 処理では、可能なら `str_slice_chars_result` のような char index API を使います。
