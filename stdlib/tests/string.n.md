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
    let ok0 eq len<str> parts 3;
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
    let checks <Vec<Result<(),str>>>:
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
