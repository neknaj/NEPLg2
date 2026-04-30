# stdlib/string.n.md

## string_len_and_concat

neplg2:test
ret: 1
```neplg2
#entry main
#indent 4
#target std

#import "alloc/string" as *
#import "core/math" as *

fn main <()*>i32> ():
    let s:
        "hello"
        |> concat "world"
    let s1234 from_i32 1234;
    let ok0 eq len s 10;
    let ok1 eq len s1234 4;
    if and ok0 ok1 1 0
```

## string_trim_and_slice

neplg2:test
ret: 1
```neplg2
#entry main
#indent 4
#target std

#import "alloc/string" as *
#import "core/math" as *

fn main <()*>i32> ():
    let src "  fn main(a: i32)  ";
    let trimmed str_trim src;
    let part str_slice trimmed 3 7;
    let ok0 eq len trimmed 15;
    let ok1 and str_starts_with trimmed "fn" str_ends_with trimmed ")";
    let ok2 and eq len part 4 and str_starts_with part "ma" str_ends_with part "in";
    if and ok0 and ok1 ok2 1 0
```

## string_split_and_builder

neplg2:test
ret: 1
```neplg2
#entry main
#indent 4
#target std

#import "alloc/string" as *
#import "alloc/collections/vec" as *
#import "core/math" as *

fn main <()*>i32> ():
    let parts str_split "a--b--c" "--";
    let msg <str>:
        string_builder_new
        |> sb_append "Error: "
        |> sb_append_i32 404
        |> sb_append " Not Found"
        |> sb_build
    let ok0 eq len<str> &parts 3;
    let ok1 eq len msg 20;
    if and ok0 ok1 1 0
```

## string_byte_at

neplg2:test
ret: 1
```neplg2
#entry main
#indent 4
#target std

#import "alloc/string" as *
#import "core/math" as *
#import "core/option" as *

fn main <()*>i32> ():
    let ok0 <bool> match byte_at "AZ" 0:
        Option::Some b:
            eq b 65
        Option::None:
            false
    let ok1 <bool> match byte_at "AZ" 1:
        Option::Some b:
            eq b 90
        Option::None:
            false
    let ok2 <bool> is_none<i32> byte_at "AZ" 2;
    if and ok0 and ok1 ok2 1 0
```

## string_find_byte_index

`find` が最初に一致した byte index を返し、空 pattern、未検出、source より長い pattern を区別できることを確認します。
NEPLg3 self-host lexer が改行区切りや delimiter を探す用途を想定し、改行を含む探索も固定します。

neplg2:test
ret: 0
```neplg2
#entry main
#indent 4
#target std

#import "alloc/string" as *
#import "core/option" as *
#import "std/test" as *

fn expect_find_some <(str,Option<i32>,i32)*>Result<(),str>> (label, got, expected):
    match got:
        Option::Some actual:
            assert_eq_i32 expected actual
        Option::None:
            Result<(),str>::Err label

fn main <()*>i32> ():
    let checks:
        checks_new
        |> checks_push expect_find_some "empty" find "abc" "" 0
        |> checks_push expect_find_some "prefix" find "abc" "ab" 0
        |> checks_push expect_find_some "middle" find "abcabc" "ca" 2
        |> checks_push expect_find_some "suffix" find "abc" "bc" 1
        |> checks_push expect_find_some "delimiter" find "#target std\n#entry main" "\n#entry" 11
        |> checks_push assert is_none<i32> find "abc" "z"
        |> checks_push assert is_none<i32> find "ab" "abc"
    checks_exit_code checks
```

## string_result_allocation_apis

`concat_result` / `str_slice_result` / `str_split_result` / `StringBuilder` の Result API が、既存の互換 facade と同じ内容を返せることを確認します。
allocator failure を trap へ寄せない self-host 用入口を固定するための回帰テストです。

neplg2:test
ret: 0
```neplg2
#entry main
#indent 4
#target std

#import "alloc/string" as *
#import "alloc/collections/vec" as *
#import "core/option" as *
#import "core/result" as *
#import "std/test" as *

fn expect_str_ok <(str,Result<str,str>,str)*>Result<(),str>> (label, got, expected):
    match got:
        Result::Ok actual:
            check_str_eq expected actual
        Result::Err e:
            Result<(),str>::Err concat label e

fn expect_vec_middle <(Result<Vec<str>,str>)*>Result<(),str>> (got):
    match got:
        Result::Err e:
            Result<(),str>::Err e
        Result::Ok parts:
            let part_len <i32> len<str> &parts
            match get<str> &parts 1:
                Option::Some mid:
                    if:
                        and eq part_len 3 str_eq mid "b"
                        then:
                            Result<(),str>::Ok ()
                        else:
                            Result<(),str>::Err "split content mismatch"
                Option::None:
                    Result<(),str>::Err "split middle missing"

fn main <()*>i32> ():
    let builder_result <Result<str,str>>:
        match string_builder_new_result:
            Result::Err e:
                Result<str,str>::Err e
            Result::Ok sb0:
                match sb_append_result sb0 "Error: ":
                    Result::Err e:
                        Result<str,str>::Err e
                    Result::Ok sb1:
                        match sb_append_i32_result sb1 404:
                            Result::Err e:
                                Result<str,str>::Err e
                            Result::Ok sb2:
                                match sb_append_result sb2 " Not Found":
                                    Result::Err e:
                                        Result<str,str>::Err e
                                    Result::Ok sb3:
                                        sb_build_result sb3
    let checks:
        checks_new
        |> checks_push expect_str_ok "concat: " concat_result "ab" "cd" "abcd"
        |> checks_push expect_str_ok "slice: " str_slice_result "abcdef" 2 5 "cde"
        |> checks_push expect_vec_middle str_split_result "a--b--c" "--"
        |> checks_push expect_str_ok "builder: " builder_result "Error: 404 Not Found"
    checks_exit_code checks
```

## string_utf8_mem_result

`string_from_utf8_mem_result` が有効な UTF-8 を `str` へ複製し、invalid leading byte を拒否することを確認します。
`alloc/string` 自体の境界で UTF-8 不変条件を固定するための回帰テストです。

neplg2:test
ret: 0
```neplg2
#entry main
#indent 4
#target std

#import "alloc/string" as *
#import "core/mem" as *
#import "core/result" as *
#import "std/test" as *

fn main <()*>i32> ():
    let mut checks checks_new;
    let src <str> "こんにちは";
    match string_from_utf8_mem_result string_data_ptr src len src:
        Result::Ok copied:
            set checks checks_push checks check_str_eq src copied
        Result::Err e:
            set checks checks_push checks Result<(),str>::Err e;
    match alloc_ptr<u8> 1:
        Result::Err _e:
            set checks checks_push checks Result<(),str>::Err "alloc failed"
        Result::Ok data:
            let raw <i32> mem_ptr_addr data;
            store_u8 raw 128;
            match string_from_utf8_mem_result data 1:
                Result::Ok _text:
                    set checks checks_push checks Result<(),str>::Err "invalid UTF-8 was accepted"
                Result::Err _e:
                    set checks checks_push checks Result<(),str>::Ok ();
            dealloc_raw raw 1;
    checks_exit_code checks
```

## string_to_f64_parser

`to_f64` が整数部だけ、符号付き小数、先頭が小数点の値、指数表記を clean end-of-input で正しく受理し、
不正な末尾や digit 不足を `Result::Err` として返すことを確認します。

neplg2:test
ret: 0
```neplg2
#entry main
#indent 4
#target std

#import "alloc/string" as *
#import "alloc/collections/vec" as *
#import "core/math" as *
#import "core/result" as *
#import "std/test" as *

fn expect_f64_ok <(str,Result<f64,i32>,f64)*>Result<(),str>> (label, got, expected):
    match got:
        Result::Ok actual:
            if:
                eq actual expected
                then:
                    Result<(),str>::Ok ()
                else:
                    Result<(),str>::Err concat label " value mismatch"
        Result::Err _e:
            Result<(),str>::Err concat label " rejected"

fn expect_f64_err <(str,Result<f64,i32>)*>Result<(),str>> (label, got):
    match got:
        Result::Ok _actual:
            Result<(),str>::Err concat label " accepted"
        Result::Err _e:
            Result<(),str>::Ok ()

fn main <()*>i32> ():
    let mut checks checks_new;
    let expected_integer <f64> <f64> cast 123;
    let expected_signed_fraction <f64> div <f64> cast -3 <f64> cast 2;
    let expected_leading_fraction <f64> div <f64> cast 1 <f64> cast 2;
    let expected_integer_exp <f64> <f64> cast 100;
    let expected_fraction_exp <f64> <f64> cast 125;
    set checks checks_push checks expect_f64_ok "integer" (to_f64 "123") expected_integer;
    set checks checks_push checks expect_f64_ok "signed fraction" (to_f64 "-1.5") expected_signed_fraction;
    set checks checks_push checks expect_f64_ok "leading fraction" (to_f64 ".5") expected_leading_fraction;
    set checks checks_push checks expect_f64_ok "integer exponent" (to_f64 "1e2") expected_integer_exp;
    set checks checks_push checks expect_f64_ok "fraction exponent" (to_f64 "1.25e2") expected_fraction_exp;
    set checks checks_push checks expect_f64_err "empty" (to_f64 "");
    set checks checks_push checks expect_f64_err "sign only" (to_f64 "-");
    set checks checks_push checks expect_f64_err "dot only" (to_f64 ".");
    set checks checks_push checks expect_f64_err "missing exponent" (to_f64 "1e");
    set checks checks_push checks expect_f64_err "trailing byte" (to_f64 "1x");
    set checks checks_push checks expect_f64_err "trailing fraction byte" (to_f64 "1.2x");
    checks_exit_code checks
```

## string_slice_utf8_boundary

`str_slice_result` が UTF-8 の文字境界に揃った範囲だけを `str` として返し、
multi-byte 文字の途中で切る範囲を `Result::Err` にすることを確認します。

neplg2:test
ret: 0
```neplg2
#entry main
#indent 4
#target std

#import "alloc/string" as *
#import "alloc/collections/vec" as *
#import "core/result" as *
#import "std/test" as *

fn expect_slice_ok <(str,Result<str,str>,str)*>Result<(),str>> (label, got, expected):
    match got:
        Result::Ok actual:
            check_str_eq expected actual
        Result::Err e:
            Result<(),str>::Err concat label e

fn expect_slice_err <(str,Result<str,str>)*>Result<(),str>> (label, got):
    match got:
        Result::Ok _actual:
            Result<(),str>::Err concat label " accepted invalid boundary"
        Result::Err _e:
            Result<(),str>::Ok ()

fn main <()*>i32> ():
    let mut checks checks_new;
    set checks checks_push checks expect_slice_ok "full: " (str_slice_result "あ" 0 3) "あ";
    set checks checks_push checks expect_slice_ok "second: " (str_slice_result "あい" 3 6) "い";
    set checks checks_push checks expect_slice_err "cut end: " (str_slice_result "あ" 0 1);
    set checks checks_push checks expect_slice_err "cut start: " (str_slice_result "あ" 1 3);
    set checks checks_push checks check_str_eq "" (str_slice "あ" 0 1);
    checks_exit_code checks
```
