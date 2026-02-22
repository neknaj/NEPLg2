# stdin.rs 由来の doctest

このファイルは Rust テスト `stdin.rs` を .n.md 形式へ機械的に移植したものです。移植が難しい（複数ファイルや Rust 専用 API を使う）テストは `skip` として残しています。
## stdin_echo_ascii

neplg2:test[normalize_newlines]
stdin: "1 2 +\n"
stdout: "1 2 +\n"
```neplg2
#entry main
#indent 4
#target std
#import "std/stdio" as *

fn main <()*>()> ():
    let s <str> read_all;
    print s;
    ()
```

## stdin_echo_japanese

neplg2:test[normalize_newlines]
stdin: "こんにちは\n"
stdout: "こんにちは\n"
```neplg2
#entry main
#indent 4
#target std
#import "std/stdio" as *

fn main <()*>()> ():
    let s <str> read_all;
    print s;
    ()
```

## stdin_readline_ascii

neplg2:test[normalize_newlines]
stdin: "1 2 +\n"
stdout: "1 2 +"
```neplg2
#entry main
#indent 4
#target std
#import "std/stdio" as *

fn main <()*>()> ():
    let s <str> read_line;
    print s;
    ()
```

## stdin_readline_japanese

neplg2:test[normalize_newlines]
stdin: "こんにちは\n"
stdout: "こんにちは"
```neplg2
#entry main
#indent 4
#target std
#import "std/stdio" as *

fn main <()*>()> ():
    let s <str> read_line;
    print s;
    ()
```

## stdin_kpread_utf8_bom

neplg2:test[normalize_newlines]
stdin: "﻿1 3\n"
stdout: "1\n3\n"
```neplg2
#entry main
#indent 4
#target std

#import "kp/kpread" as *
#import "std/stdio" as *

fn main <()*> ()> ():
    let sc <i32> scanner_new;
    let a <i32> scanner_read_i32 sc;
    let b <i32> scanner_read_i32 sc;
    println_i32 a;
    println_i32 b;
```
