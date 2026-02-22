# stdout.rs 由来の doctest

このファイルは Rust テスト `stdout.rs` を .n.md 形式へ機械的に移植したものです。移植が難しい（複数ファイルや Rust 専用 API を使う）テストは `skip` として残しています。
## stdout_concat_is_stable

neplg2:test[normalize_newlines]
stdout: "a:12:b"
```neplg2
#entry main
#indent 4
#target std
#import "std/stdio" as *

fn main <()*>()> ():
    print "a:";
    print_i32 12;
    print ":b";
    ()
```

## println_appends_newline

neplg2:test[normalize_newlines]
stdout: "ab\nc"
```neplg2
#entry main
#indent 4
#target std
#import "std/stdio" as *

fn main <()*>()> ():
    print "a";
    println "b";
    print "c";
    ()
```

## println_i32_appends_newline

neplg2:test[normalize_newlines]
stdout: "12\n3"
```neplg2
#entry main
#indent 4
#target std
#import "std/stdio" as *

fn main <()*>()> ():
    print_i32 1;
    println_i32 2;
    print_i32 3;
    ()
```

## stdout_japanese_utf8

neplg2:test[normalize_newlines]
stdout: "こんにちは世界!\n"
```neplg2
#entry main
#indent 4
#target std

#import "std/stdio" as *

fn main <()*> ()> ():
    println "こんにちは世界!";
```

## stdout_ansi_escape

neplg2:test[normalize_newlines]
stdout: "\u001b[31mred\u001b[0m\n"
```neplg2
#entry main
#indent 4
#target std

#import "std/stdio" as *

fn main <()*> ()> ():
    println "\x1b[31mred\x1b[0m";
```

## stdout_ansi_helpers

neplg2:test[normalize_newlines]
stdout: "\u001b[31mred\u001b[0m \u001b[32mgreen\u001b[0m\n"
```neplg2
#entry main
#indent 4
#target std

#import "std/stdio" as *

fn main <()*> ()> ():
    print_color ansi_red "red";
    print " ";
    println_color ansi_green "green";
```
