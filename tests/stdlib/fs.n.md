# fs facade

## fs_read_to_string_missing_file

このケースは、存在しないファイルを読んだときに `fs_read_to_string` が `Err` を返すことを確認します。
ファイルシステム依存の失敗を成功扱いしないことが目的です。

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: mlstr:
    ##: Checked [ok]
    ##: [0] ok
```neplg2
#entry main
#indent 4
#target std

#import "std/fs" as *
#import "std/test" as *
#import "core/result" as *

fn main %impure fn unit i32 \unit:
    let mut checks checks_new;
    match fs_read_to_string "__definitely_missing_file__.txt":
        Result::Ok s:
            set checks checks_push checks check_str_eq "__expected_missing_file_error__" s
        Result::Err _e:
            set checks checks_push checks Result::Ok unit;
    let shown checks_print_report checks;
    checks_exit_code shown
```

## fs_bytes_to_string_roundtrip

このケースは、`ByteBuf` を `fs_bytes_to_string` で `str` に戻せることを確認します。
host filesystem の preopen に依存しない形で、`std/fs` の binary helper が `ByteBuf` 前提で保たれていることを確認するのが目的です。

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: mlstr:
    ##: Checked [ok]
    ##: [0] ok
```neplg2
#entry main
#indent 4
#target std

#import "std/fs" as *
#import "std/test" as *
#import "alloc/io" as *
#import "core/result" as *

fn main %impure fn unit i32 \unit:
    let mut checks checks_new;
    let bytes %ByteBuf io_bytebuf_from_str "fs helper";
    let text %str fs_bytes_to_string bytes;
    set checks checks_push checks check_str_eq "fs helper" text;
    let shown checks_print_report checks;
    checks_exit_code shown
```

## fs_write_to_string_creates_and_truncates

このケースは、`fs_write_to_string` が存在しないファイルを作成し、既存ファイルを truncate してから内容を書き直すことを確認します。
self-host compiler が同じ output path へ artifact を再生成するとき、古い内容が末尾に残らないことが目的です。

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: mlstr:
    ##: Checked [ok]
    ##: [0] ok
```neplg2
#entry main
#indent 4
#target std

#import "std/fs" as *
#import "std/test" as *
#import "core/result" as *

fn main %impure fn unit i32 \unit:
    let path %str "tmp/fs_write_to_string_case.txt"
    let mut checks checks_new;
    match fs_write_to_string path "first-longer":
        Result::Err _e:
            set checks checks_push checks Result::Err "first write failed"
        Result::Ok _:
            match fs_write_to_string path "second":
                Result::Err _e:
                    set checks checks_push checks Result::Err "second write failed"
                Result::Ok _:
                    match fs_read_to_string path:
                        Result::Err _e:
                            set checks checks_push checks Result::Err "read after write failed"
                        Result::Ok text:
                            set checks checks_push checks check_str_eq "second" text;
    let shown checks_print_report checks;
    checks_exit_code shown
```

## fs_write_to_bytes_preserves_nul

このケースは、`fs_write_to_bytes` が text ではなく binary buffer として NUL byte を含む内容をそのまま保存できることを確認します。
`.wasm` artifact は NUL を含むため、文字列 roundtrip だけでは write API の十分な回帰テストになりません。

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: mlstr:
    ##: Checked [ok,ok,ok,ok]
    ##: [0] ok
    ##: [1] ok
    ##: [2] ok
    ##: [3] ok
```neplg2
#entry main
#indent 4
#target std

#import "std/fs" as *
#import "std/test" as *
#import "alloc/io" as *
#import "core/option" as *
#import "core/result" as *
#import "core/math" as *

fn main %impure fn unit i32 \unit:
    let path %str "tmp/fs_write_to_bytes_case.bin"
    let mut checks checks_new;
    match io_bytebuf_from_str_result "A\x00B":
        Result::Err _e:
            set checks checks_push checks Result::Err "alloc failed"
        Result::Ok data:
            match fs_write_to_bytes path data:
                Result::Err _e:
                    set checks checks_push checks Result::Err "binary write failed"
                Result::Ok _:
                    match fs_read_to_bytes path:
                        Result::Err _e:
                            set checks checks_push checks Result::Err "binary read failed"
                        Result::Ok read_buf:
                            set checks checks_push checks check_eq_i32 3 io_bytebuf_len_ref &read_buf;
                            match io_bytebuf_byte_at &read_buf 0:
                                Option::Some b0:
                                    set checks checks_push checks check_eq_i32 65 b0;
                                Option::None:
                                    set checks checks_push checks Result::Err "missing byte 0";
                            match io_bytebuf_byte_at &read_buf 1:
                                Option::Some b1:
                                    set checks checks_push checks check_eq_i32 0 b1;
                                Option::None:
                                    set checks checks_push checks Result::Err "missing byte 1";
                            match io_bytebuf_byte_at &read_buf 2:
                                Option::Some b2:
                                    set checks checks_push checks check_eq_i32 66 b2;
                                Option::None:
                                    set checks checks_push checks Result::Err "missing byte 2";
                            io_bytebuf_free read_buf;
    let shown checks_print_report checks;
    checks_exit_code shown
```

## fs_file_kind_helpers

このケースは、`fs_exists` / `fs_is_file` / `fs_is_dir` が読み込み副作用なしで file kind を判定できることを確認します。
stdlib discovery が候補 path を read 失敗で分類しないための回帰テストです。

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: mlstr:
    ##: Checked [ok,ok,ok,ok]
    ##: [0] ok
    ##: [1] ok
    ##: [2] ok
    ##: [3] ok
```neplg2
#entry main
#indent 4
#target std

#import "std/fs" as *
#import "std/test" as *
#import "core/result" as *

fn main %impure fn unit i32 \unit:
    let mut checks checks_new;
    set checks checks_push checks check fs_exists "tests/fixtures/fs/read_sample.txt";
    set checks checks_push checks check fs_is_file "tests/fixtures/fs/read_sample.txt";
    set checks checks_push checks check fs_is_dir "tests/fixtures/fs/dirlist";
    if:
        fs_exists "tests/fixtures/fs/__missing__.txt"
        then:
            set checks checks_push checks Result::Err "missing path unexpectedly exists"
        else:
            set checks checks_push checks Result::Ok unit;
    let shown checks_print_report checks;
    checks_exit_code shown
```

## fs_normalize_relative_rejects_escape

このケースは、相対 path 正規化が `.` と内部 `..` を畳みつつ、preopen root 外へ出る `..` を拒否することを確認します。
host path 文字列を caller が手作業で扱う経路を増やさないための回帰テストです。

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: mlstr:
    ##: Checked [ok,ok,ok]
    ##: [0] ok
    ##: [1] ok
    ##: [2] ok
```neplg2
#entry main
#indent 4
#target std

#import "std/fs" as *
#import "std/test" as *
#import "core/result" as *
#import "core/math" as *

fn main %impure fn unit i32 \unit:
    let mut checks checks_new;
    match fs_normalize_relative "tests/fixtures/fs/dirlist/../read_sample.txt":
        Result::Ok path:
            set checks checks_push checks check_str_eq "tests/fixtures/fs/read_sample.txt" path
        Result::Err _e:
            set checks checks_push checks Result::Err "normalizing internal parent failed";
    match fs_normalize_relative "a/./b//c":
        Result::Ok path:
            set checks checks_push checks check_str_eq "a/b/c" path
        Result::Err _e:
            set checks checks_push checks Result::Err "normalizing dot and empty components failed";
    match fs_normalize_relative "../outside":
        Result::Ok path:
            set checks checks_push checks check_str_eq "__expected_parent_escape_error__" path
        Result::Err e:
            set checks checks_push checks check_eq_i32 76 e;
    let shown checks_print_report checks;
    checks_exit_code shown
```

## fs_read_dir_returns_sorted_entries

このケースは、`fs_read_dir` が directory entry 名だけを安定した byte 順で返すことを確認します。
host filesystem の列挙順が違っても self-host compiler の discovery 結果がぶれないようにするための回帰テストです。

neplg2:test[skip, stdio, normalize_newlines]
exit_code: 0
stdout: mlstr:
    ##: Checked [ok,ok,ok,ok]
    ##: [0] ok
    ##: [1] ok
    ##: [2] ok
    ##: [3] ok
```neplg2
#entry main
#indent 4
#target std

#import "std/fs" as *
#import "std/test" as *
#import "core/result" as *
#import "core/option" as *
#import "alloc/collections/vec" as v
#import "alloc/string" as *
#import "core/math" as *

fn main %impure fn unit i32 \unit:
    let mut checks checks_new;
    match fs_read_dir "tests/fixtures/fs/dirlist":
        Result::Err e:
            set checks checks_push checks Result::Err concat "fs_read_dir failed errno=" from_i32 e
        Result::Ok entries:
            set checks checks_push checks check_eq_i32 3 v::len &entries;
            match v::get &entries 0:
                Option::Some entry0:
                    set checks checks_push checks check_str_eq "alpha.nepl" entry0
                Option::None:
                    set checks checks_push checks Result::Err "missing directory entry 0";
            match v::get &entries 1:
                Option::Some entry1:
                    set checks checks_push checks check_str_eq "beta.n.md" entry1
                Option::None:
                    set checks checks_push checks Result::Err "missing directory entry 1";
            match v::get &entries 2:
                Option::Some entry2:
                    set checks checks_push checks check_str_eq "zeta.txt" entry2
                Option::None:
                    set checks checks_push checks Result::Err "missing directory entry 2";
            v::free entries;
    let shown checks_print_report checks;
    checks_exit_code shown
```

## fs_sort_strings_uses_vec_boundary

このケースは、directory entry sort が `Vec` の公開 API 経由で `str` view を並べ替えることを確認します。
host filesystem に依存せず、`fs_read_dir` の sort 境界だけを直接検証するための回帰テストです。

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: mlstr:
    ##: Checked [ok,ok,ok]
    ##: [0] ok
    ##: [1] ok
    ##: [2] ok
```neplg2
#entry main
#indent 4
#target std

#import "std/fs/path" as *
#import "std/test" as *
#import "core/result" as *
#import "core/option" as *
#import "alloc/collections/vec" as v

fn main %impure fn unit i32 \unit:
    let entries %Vec str:
        unwrap_ok v::new<str>
        |> v::push<str> "zeta.txt" |> uwok
        |> v::push<str> "alpha.nepl" |> uwok
        |> v::push<str> "beta.n.md" |> uwok
    let mut checks checks_new;
    match fs_sort_strings &entries:
        Result::Err _e:
            set checks checks_push checks Result::Err "fs_sort_strings failed"
        Result::Ok _:
            match v::get &entries 0:
                Option::Some entry0:
                    set checks checks_push checks check_str_eq "alpha.nepl" entry0
                Option::None:
                    set checks checks_push checks Result::Err "missing sorted entry 0";
            match v::get &entries 1:
                Option::Some entry1:
                    set checks checks_push checks check_str_eq "beta.n.md" entry1
                Option::None:
                    set checks checks_push checks Result::Err "missing sorted entry 1";
            match v::get &entries 2:
                Option::Some entry2:
                    set checks checks_push checks check_str_eq "zeta.txt" entry2
                Option::None:
                    set checks checks_push checks Result::Err "missing sorted entry 2";
    v::free entries;
    let shown checks_print_report checks;
    checks_exit_code shown
```
