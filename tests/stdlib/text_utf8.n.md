# text UTF-8 validation

## bytebuf_to_utf8_str_accepts_multibyte_text

このケースは、UTF-8 checked conversion が日本語を含む有効な byte 列を `str` として受け入れることを確認します。
source text の通常入力が invalid byte 対策によって退行しないことが目的です。

neplg2:test
ret: 0
```neplg2
#entry main
#indent 4
#target std

#import "std/text" as *
#import "std/test" as *
#import "alloc/io" as *
#import "core/result" as *

fn main <()*>i32> ():
    let mut checks <Vec<Result<(),str>>> checks_new;
    match text_bytebuf_to_utf8_str_result io_bytebuf_from_str "こんにちは":
        Result::Ok text:
            set checks checks_push checks check_str_eq "こんにちは" text
        Result::Err _e:
            set checks checks_push checks Result<(),str>::Err "valid UTF-8 was rejected";
    let shown <Vec<Result<(),str>>> checks_print_report checks;
    checks_exit_code shown
```

## bytebuf_to_utf8_str_rejects_invalid_leading_byte

このケースは、continuation byte 単体を `str` に変換しないことを確認します。
source loader が byte offset / span の前提を壊す入力を境界で拒否するための回帰テストです。

neplg2:test
ret: 0
```neplg2
#entry main
#indent 4
#target std

#import "std/text" as *
#import "std/test" as *
#import "alloc/io" as *
#import "core/mem" as *
#import "core/result" as *

fn main <()*>i32> ():
    let mut checks <Vec<Result<(),str>>> checks_new;
    match alloc_ptr<u8> 1:
        Result::Err _e:
            set checks checks_push checks Result<(),str>::Err "alloc failed"
        Result::Ok data:
            let raw <i32> mem_ptr_addr data
            store_u8 raw 128;
            match text_bytebuf_to_utf8_str_result ByteBuf data 1:
                Result::Ok _text:
                    set checks checks_push checks Result<(),str>::Err "invalid leading byte was accepted"
                Result::Err e:
                    set checks checks_push checks check_str_eq "InvalidUtf8" std_error_kind_str e;
    let shown <Vec<Result<(),str>>> checks_print_report checks;
    checks_exit_code shown
```

## bytebuf_to_utf8_str_rejects_overlong_sequence

このケースは、overlong encoding を continuation byte の個数だけで受け入れないことを確認します。
UTF-8 scalar value の制約まで含めて検証するための回帰テストです。

neplg2:test
ret: 0
```neplg2
#entry main
#indent 4
#target std

#import "std/text" as *
#import "std/test" as *
#import "alloc/io" as *
#import "core/mem" as *
#import "core/result" as *

fn main <()*>i32> ():
    let mut checks <Vec<Result<(),str>>> checks_new;
    match alloc_ptr<u8> 3:
        Result::Err _e:
            set checks checks_push checks Result<(),str>::Err "alloc failed"
        Result::Ok data:
            let raw <i32> mem_ptr_addr data
            store_u8 raw 224;
            store_u8 add raw 1 128;
            store_u8 add raw 2 128;
            match text_bytebuf_to_utf8_str_result ByteBuf data 3:
                Result::Ok _text:
                    set checks checks_push checks Result<(),str>::Err "overlong sequence was accepted"
                Result::Err e:
                    set checks checks_push checks check_str_eq "InvalidUtf8" std_error_kind_str e;
    let shown <Vec<Result<(),str>>> checks_print_report checks;
    checks_exit_code shown
```

## fs_read_to_string_checked_rejects_invalid_utf8

このケースは、file read の checked text API が invalid UTF-8 を errno 84 として拒否することを確認します。
binary 読み込みは `ByteBuf` のまま扱い、source text 読み込みだけを checked API に寄せるための回帰テストです。

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
    let mut checks <Vec<Result<(),str>>> checks_new;
    let path <str> "tmp/fs_invalid_utf8_checked_case.bin"
    match alloc_ptr<u8> 1:
        Result::Err _e:
            set checks checks_push checks Result<(),str>::Err "alloc failed"
        Result::Ok data:
            let raw <i32> mem_ptr_addr data
            store_u8 raw 128;
            match fs_write_to_bytes path ByteBuf data 1:
                Result::Err _e:
                    set checks checks_push checks Result<(),str>::Err "write failed"
                Result::Ok _:
                    match fs_read_to_string_checked path:
                        Result::Ok _text:
                            set checks checks_push checks Result<(),str>::Err "invalid UTF-8 file was accepted"
                        Result::Err e:
                            set checks checks_push checks check_eq_i32 84 e;
    let shown <Vec<Result<(),str>>> checks_print_report checks;
    checks_exit_code shown
```

## io_read_bytes_target_rejects_invalid_utf8_text

このケースは、`std/io` の `ReadStream::Bytes` を text として読むときにも checked conversion が使われることを確認します。
target facade 経由で unchecked `str` が作られる経路を残さないための回帰テストです。

neplg2:test
ret: 0
```neplg2
#entry main
#indent 4
#target std

#import "std/io" as *
#import "std/iotarget" as *
#import "std/test" as *
#import "alloc/io" as *
#import "core/mem" as *
#import "core/result" as *

fn main <()*>i32> ():
    let mut checks <Vec<Result<(),str>>> checks_new;
    match alloc_ptr<u8> 1:
        Result::Err _e:
            set checks checks_push checks Result<(),str>::Err "alloc failed"
        Result::Ok data:
            let raw <i32> mem_ptr_addr data
            store_u8 raw 128;
            let target <ReadStream> ReadStream::Bytes ByteBuf data 1
            let text_result <Result<str, StdErrorKind>> read target;
            match text_result:
                Result::Ok _text:
                    set checks checks_push checks Result<(),str>::Err "invalid bytes target was accepted as text"
                Result::Err e:
                    set checks checks_push checks check_str_eq "InvalidUtf8" std_error_kind_str e;
    let shown <Vec<Result<(),str>>> checks_print_report checks;
    checks_exit_code shown
```
