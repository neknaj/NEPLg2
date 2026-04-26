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

## fs_write_to_string_creates_and_truncates

このケースは、`fs_write_to_string` が存在しないファイルを作成し、既存ファイルを truncate してから内容を書き直すことを確認します。
self-host compiler が同じ output path へ artifact を再生成するとき、古い内容が末尾に残らないことが目的です。

neplg2:test
ret: 0
```neplg2
#entry main
#indent 4
#target std

#import "std/fs" as *
#import "std/test" as *
#import "core/result" as *

fn main <()*>i32> ():
    let path <str> "tmp/fs_write_to_string_case.txt"
    let mut checks <Vec<Result<(),str>>> checks_new;
    match fs_write_to_string path "first-longer":
        Result::Err _e:
            set checks checks_push checks Result<(),str>::Err "first write failed"
        Result::Ok _:
            match fs_write_to_string path "second":
                Result::Err _e:
                    set checks checks_push checks Result<(),str>::Err "second write failed"
                Result::Ok _:
                    match fs_read_to_string path:
                        Result::Err _e:
                            set checks checks_push checks Result<(),str>::Err "read after write failed"
                        Result::Ok text:
                            set checks checks_push checks check_str_eq "second" text;
    let shown <Vec<Result<(),str>>> checks_print_report checks;
    checks_exit_code shown
```

## fs_write_to_bytes_preserves_nul

このケースは、`fs_write_to_bytes` が text ではなく binary buffer として NUL byte を含む内容をそのまま保存できることを確認します。
`.wasm` artifact は NUL を含むため、文字列 roundtrip だけでは write API の十分な回帰テストになりません。

neplg2:test
ret: 0
```neplg2
#entry main
#indent 4
#target std

#import "std/fs" as *
#import "std/test" as *
#import "alloc/io" as *
#import "core/mem" as *
#import "core/result" as *

fn main <()*>i32> ():
    let path <str> "tmp/fs_write_to_bytes_case.bin"
    let mut checks <Vec<Result<(),str>>> checks_new;
    match alloc_ptr<u8> 3:
        Result::Err _e:
            set checks checks_push checks Result<(),str>::Err "alloc failed"
        Result::Ok data:
            let raw <i32> mem_ptr_addr data
            store_u8 raw 65;
            store_u8 add raw 1 0;
            store_u8 add raw 2 66;
            match fs_write_to_bytes path ByteBuf data 3:
                Result::Err _e:
                    set checks checks_push checks Result<(),str>::Err "binary write failed"
                Result::Ok _:
                    match fs_read_to_bytes path:
                        Result::Err _e:
                            set checks checks_push checks Result<(),str>::Err "binary read failed"
                        Result::Ok read_buf:
                            let read_ptr <MemPtr<u8>> get read_buf "ptr"
                            let read_raw <i32> mem_ptr_addr read_ptr
                            set checks checks_push checks check_eq_i32 3 get read_buf "len";
                            set checks checks_push checks check_eq_i32 65 load_u8 read_raw;
                            set checks checks_push checks check_eq_i32 0 load_u8 add read_raw 1;
                            set checks checks_push checks check_eq_i32 66 load_u8 add read_raw 2;
                            io_bytebuf_free read_buf;
    let shown <Vec<Result<(),str>>> checks_print_report checks;
    checks_exit_code shown
```
