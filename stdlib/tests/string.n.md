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
