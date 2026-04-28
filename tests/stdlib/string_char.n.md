# string char API

`str` の byte index API と char index API を分け、UTF-8 scalar value としての `char` を扱う回帰テストです。

## string_char_count_access_and_slice

neplg2:test
ret: 0
```neplg2
#entry main
#target std
#indent 4

#import "alloc/string" as *
#import "core/char" as *
#import "core/result" as *
#import "std/test" as *

fn expect_char <(str,Result<char,str>,i32)*>Result<(),str>> (label, got, expected_code):
    match got:
        Result::Err _e:
            Result<(),str>::Err label
        Result::Ok c:
            check_eq_i32 expected_code char_to_i32 c

fn expect_str_ok <(str,Result<str,str>,str)*>Result<(),str>> (label, got, expected):
    match got:
        Result::Err _e:
            Result<(),str>::Err label
        Result::Ok text:
            check_str_eq expected text

fn main <()*>i32> ():
    let s <str> "Aあ💯"
    let checks <Vec<Result<(),str>>>:
        checks_new
        |> checks_push assert_eq_i32 8 str_byte_len s
        |> checks_push assert_eq_i32 3 str_char_count s
        |> checks_push expect_char "char 0" str_char_at_result s 0 'A'
        |> checks_push expect_char "char 1" str_char_at_result s 1 0x3042
        |> checks_push expect_char "char 2" str_char_at_result s 2 0x1F4AF
        |> checks_push expect_str_ok "slice 1..3" str_slice_chars_result s 1 3 "あ💯"
        |> checks_push assert is_err<char,str> str_char_at_result s 3
        |> checks_push assert is_err<str,str> str_slice_chars_result s 2 1
    checks_exit_code checks
```

## string_next_char_and_contains

neplg2:test
ret: 0
```neplg2
#entry main
#target std
#indent 4

#import "alloc/string" as *
#import "core/char" as *
#import "core/field" as *
#import "core/result" as *
#import "std/test" as *

fn expect_next <(str,Result<CharUtf8Step,str>,i32,i32)*>Result<(),str>> (label, got, expected_code, expected_next):
    match got:
        Result::Err _e:
            Result<(),str>::Err label
        Result::Ok item:
            let c <char> get item "value"
            let next <i32> get item "next"
            let code_ok <Result<(),str>> check_eq_i32 expected_code char_to_i32 c
            match code_ok:
                Result::Err e:
                    Result<(),str>::Err e
                Result::Ok _:
                    check_eq_i32 expected_next next

fn main <()*>i32> ():
    let s <str> "Aあ"
    let checks <Vec<Result<(),str>>>:
        checks_new
        |> checks_push expect_next "next A" str_next_char_result s 0 'A' 1
        |> checks_push expect_next "next hira" str_next_char_result s 1 0x3042 4
        |> checks_push assert is_err<CharUtf8Step,str> str_next_char_result s 2
        |> checks_push assert str_starts_with_char s 'A'
        |> checks_push assert not str_starts_with_char s 'あ'
        |> checks_push assert str_contains_char s 'あ'
        |> checks_push assert not str_contains_char s 'Z'
    checks_exit_code checks
```

## string_and_byte_builders_append_char

neplg2:test
ret: 0
```neplg2
#entry main
#target std
#indent 4

#import "alloc/io" as *
#import "alloc/string" as *
#import "core/result" as *
#import "std/test" as *

fn byte_builder_text <()*>Result<str,str>> ():
    match byte_builder_new:
        Result::Err _e:
            Result<str,str>::Err "byte builder alloc"
        Result::Ok b0:
            match byte_builder_push_char_utf8 b0 'A':
                Result::Err _e:
                    Result<str,str>::Err "byte builder push A"
                Result::Ok b1:
                    match byte_builder_push_char_utf8 b1 'あ':
                        Result::Err _e:
                            Result<str,str>::Err "byte builder push hira"
                        Result::Ok b2:
                            match byte_builder_finish b2:
                                Result::Err _e:
                                    Result<str,str>::Err "byte builder finish"
                                Result::Ok bytes:
                                    match io_bytebuf_to_str_result bytes:
                                        Result::Err _e:
                                            Result<str,str>::Err "byte builder decode"
                                        Result::Ok text:
                                            Result<str,str>::Ok text

fn main <()*>i32> ():
    let text <str>:
        string_builder_new
        |> sb_append_char 'A'
        |> sb_append_char 'あ'
        |> sb_append_ascii '!'
        |> sb_build
    let bytes_check <Result<(),str>> match byte_builder_text:
        Result::Err e:
            Result<(),str>::Err e
        Result::Ok out:
            check_str_eq "Aあ" out
    let checks <Vec<Result<(),str>>>:
        checks_new
        |> checks_push assert_str_eq "Aあ!" text
        |> checks_push bytes_check
    checks_exit_code checks
```
