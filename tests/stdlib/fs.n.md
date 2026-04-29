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
    let mut checks checks_new;
    match fs_read_to_string "__definitely_missing_file__.txt":
        Result::Ok _s:
            set checks checks_push checks Result<(),str>::Err "fs_read_to_string unexpectedly succeeded"
        Result::Err _e:
            set checks checks_push checks Result<(),str>::Ok ();
    let shown checks_print_report checks;
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
    let mut checks checks_new;
    let bytes <ByteBuf> io_bytebuf_from_str "fs helper";
    let text <str> fs_bytes_to_string bytes;
    set checks checks_push checks check_str_eq "fs helper" text;
    let shown checks_print_report checks;
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
    let mut checks checks_new;
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
    let shown checks_print_report checks;
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
#import "core/option" as *
#import "core/result" as *

fn main <()*>i32> ():
    let path <str> "tmp/fs_write_to_bytes_case.bin"
    let mut checks checks_new;
    match alloc_ptr<u8> 3:
        Result::Err _e:
            set checks checks_push checks Result<(),str>::Err "alloc failed"
        Result::Ok data:
            let raw <i32> mem_ptr_addr data
            store_u8 raw 65;
            store_u8 add raw 1 0;
            store_u8 add raw 2 66;
            match fs_write_to_bytes path io_bytebuf_from_owned_ptr data 3:
                Result::Err _e:
                    set checks checks_push checks Result<(),str>::Err "binary write failed"
                Result::Ok _:
                    match fs_read_to_bytes path:
                        Result::Err _e:
                            set checks checks_push checks Result<(),str>::Err "binary read failed"
                        Result::Ok read_buf:
                            set checks checks_push checks check_eq_i32 3 io_bytebuf_len_ref &read_buf;
                            match io_bytebuf_byte_at &read_buf 0:
                                Option::Some b0:
                                    set checks checks_push checks check_eq_i32 65 b0;
                                Option::None:
                                    set checks checks_push checks Result<(),str>::Err "missing byte 0";
                            match io_bytebuf_byte_at &read_buf 1:
                                Option::Some b1:
                                    set checks checks_push checks check_eq_i32 0 b1;
                                Option::None:
                                    set checks checks_push checks Result<(),str>::Err "missing byte 1";
                            match io_bytebuf_byte_at &read_buf 2:
                                Option::Some b2:
                                    set checks checks_push checks check_eq_i32 66 b2;
                                Option::None:
                                    set checks checks_push checks Result<(),str>::Err "missing byte 2";
                            io_bytebuf_free read_buf;
    let shown checks_print_report checks;
    checks_exit_code shown
```

## fs_file_kind_helpers

このケースは、`fs_exists` / `fs_is_file` / `fs_is_dir` が読み込み副作用なしで file kind を判定できることを確認します。
stdlib discovery が候補 path を read 失敗で分類しないための回帰テストです。

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
    let mut checks checks_new;
    set checks checks_push checks check fs_exists "tests/fixtures/fs/read_sample.txt";
    set checks checks_push checks check fs_is_file "tests/fixtures/fs/read_sample.txt";
    set checks checks_push checks check fs_is_dir "tests/fixtures/fs/dirlist";
    if:
        fs_exists "tests/fixtures/fs/__missing__.txt"
        then:
            set checks checks_push checks Result<(),str>::Err "missing path unexpectedly exists"
        else:
            set checks checks_push checks Result<(),str>::Ok ();
    let shown checks_print_report checks;
    checks_exit_code shown
```

## fs_normalize_relative_rejects_escape

このケースは、相対 path 正規化が `.` と内部 `..` を畳みつつ、preopen root 外へ出る `..` を拒否することを確認します。
host path 文字列を caller が手作業で扱う経路を増やさないための回帰テストです。

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
    let mut checks checks_new;
    match fs_normalize_relative "tests/fixtures/fs/dirlist/../read_sample.txt":
        Result::Ok path:
            set checks checks_push checks check_str_eq "tests/fixtures/fs/read_sample.txt" path
        Result::Err _e:
            set checks checks_push checks Result<(),str>::Err "normalizing internal parent failed";
    match fs_normalize_relative "a/./b//c":
        Result::Ok path:
            set checks checks_push checks check_str_eq "a/b/c" path
        Result::Err _e:
            set checks checks_push checks Result<(),str>::Err "normalizing dot and empty components failed";
    match fs_normalize_relative "../outside":
        Result::Ok _path:
            set checks checks_push checks Result<(),str>::Err "parent escape normalized successfully"
        Result::Err e:
            set checks checks_push checks check_eq_i32 76 e;
    let shown checks_print_report checks;
    checks_exit_code shown
```

## fs_read_dir_returns_sorted_entries

このケースは、`fs_read_dir` が directory entry 名だけを安定した byte 順で返すことを確認します。
host filesystem の列挙順が違っても self-host compiler の discovery 結果がぶれないようにするための回帰テストです。

neplg2:test[skip]
ret: 0
```neplg2
#entry main
#indent 4
#target std

#import "std/fs" as *
#import "std/test" as *
#import "core/result" as *
#import "core/option" as *
#import "core/mem" as *
#import "alloc/collections/vec" as v
#import "alloc/string" as *

fn main <()*>i32> ():
    let mut checks checks_new;
    match fs_read_dir "tests/fixtures/fs/dirlist":
        Result::Err e:
            set checks checks_push checks Result<(),str>::Err concat "fs_read_dir failed errno=" from_i32 e
        Result::Ok entries:
            set checks checks_push checks check_eq_i32 3 get entries "len";
            let entries_data <i32> mem_ptr_addr get entries "data"
            set checks checks_push checks check_str_eq "alpha.nepl" load<str> entries_data;
            set checks checks_push checks check_str_eq "beta.n.md" load<str> add entries_data size_of<str>;
            set checks checks_push checks check_str_eq "zeta.txt" load<str> add entries_data mul 2 size_of<str>;
            v::free<str> entries;
    let shown checks_print_report checks;
    checks_exit_code shown
```
