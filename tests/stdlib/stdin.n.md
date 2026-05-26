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
#import "core/result" as *

fn main %impure fn unit unit \unit:
    let s %str read_all;
    print s;
    unit
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

fn main %impure fn unit unit \unit:
    let s %str read_all;
    print s;
    unit
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

fn main %impure fn unit unit \unit:
    let s %str read_line;
    print s;
    unit
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

fn main %impure fn unit unit \unit:
    let s %str read_line;
    print s;
    unit
```

## stdin_stream_scanner_utf8_bom

neplg2:test[normalize_newlines]
stdin: "﻿1 3\n"
stdout: "1\n3\n"
```neplg2
#entry main
#indent 4
#target std

#import "std/streamio" as *
#import "std/iotarget" as *
#import "std/stdio" as *
#import "core/result" as *

fn main %impure fn unit unit \unit:
    let sc %StreamScanner unwrap_ok open ReadStream::Stdio;
    let a %i32 read &sc;
    let b %i32 read &sc;
    close sc;
    println_i32 a;
    println_i32 b;
```
