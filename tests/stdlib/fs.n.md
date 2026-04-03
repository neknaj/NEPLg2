# fs facade

## fs_read_to_string_missing_file

このケースは、存在しないファイルを読んだときに `fs_read_to_string` が `Err` を返すことを確認します。
ファイルシステム依存の失敗を成功扱いしないことが目的です。

neplg2:test
```neplg2
#entry main
#indent 4
#target std

#import "std/fs" as *
#import "std/test" as *
#import "core/result" as *

fn main <()*>i32> ():
    let mut checks <Vec<Result<(),str>>> checks_new;
    match fs_read_to_string "__definitely_missing_file__.txt":
        Result::Ok _s:
            set checks checks_push checks Result<(),str>::Err "fs_read_to_string unexpectedly succeeded"
        Result::Err _e:
            set checks checks_push checks Result<(),str>::Ok ();
    let shown <Vec<Result<(),str>>> checks_print_report checks;
    checks_exit_code shown
```

## fs_bytes_to_string_roundtrip

このケースは、`ByteBuf` を `fs_bytes_to_string` で `str` に戻せることを確認します。
host filesystem の preopen に依存しない形で、`std/fs` の binary helper が `ByteBuf` 前提で保たれていることを確認するのが目的です。

neplg2:test
ret: 0
```neplg2
#entry main
#indent 4
#target std

#import "std/fs" as *
#import "std/test" as *
#import "alloc/io" as *
#import "core/result" as *

fn main <()*>i32> ():
    let mut checks <Vec<Result<(),str>>> checks_new;
    let bytes <ByteBuf> io_bytebuf_from_str "fs helper";
    let text <str> fs_bytes_to_string bytes;
    set checks checks_push checks check_str_eq "fs helper" text;
    let shown <Vec<Result<(),str>>> checks_print_report checks;
    checks_exit_code shown
```
