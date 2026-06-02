# text UTF-8 validation

## bytebuf_to_utf8_str_accepts_multibyte_text

このケースは、UTF-8 checked conversion が日本語を含む有効な byte 列を `str` として受け入れることを確認します。
source text の通常入力が invalid byte 対策によって退行しないことが目的です。

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: mlstr:
    ##: Checked [ok]
    ##: [0] ok
```neplg2
#entry main
#indent 4
#target std

#import "std/text" as *
#import "std/test" as *
#import "alloc/io" as *
#import "core/result" as *

fn main %impure fn void i32 \void:
    let mut checks checks_new;
    match text_bytebuf_to_utf8_str_result io_bytebuf_from_str "こんにちは":
        Result::Ok text:
            set checks checks_push checks check_str_eq "こんにちは" text
        Result::Err _e:
            set checks checks_push checks Result::Err "valid UTF-8 was rejected";
    let shown checks_print_report checks;
    checks_exit_code shown
```

## text_utf8_decode_next_reads_char_offsets

`text_utf8_decode_next` が raw bytes から char と次 byte offset を返し、byte length と char count を混同しないことを確認します。

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

#import "std/text" as *
#import "std/text/decode" as *
#import "std/test" as *
#import "alloc/io" as *
#import "core/char" as *
#import "core/field" as *
#import "core/option" as *
#import "core/result" as *

fn expect_decoded %impure fn str impure fn Result CharUtf8Step StdErrorKind impure fn i32 impure fn i32 Result unit str \label\got\expected_code\expected_next:
    match got:
        Result::Err _e:
            Result::Err label
        Result::Ok item:
            let c %char get item "value"
            let next %i32 get item "next"
            match check_eq_i32 expected_code char_to_i32 c:
                Result::Err e:
                    Result::Err e
                Result::Ok _:
                    check_eq_i32 expected_next next

fn main %impure fn void i32 \void:
    let bytes %ByteBuf io_bytebuf_from_str "Aあ"
    let checks:
        match io_bytebuf_ptr_ref &bytes:
            Option::Some data:
                let byte_len %i32 io_bytebuf_len_ref &bytes
                checks_new
                |> checks_push expect_decoded "decode A" text_utf8_decode_next data byte_len 0 'A' 1
                |> checks_push expect_decoded "decode hira" text_utf8_decode_next data byte_len 1 0x3042 4
                |> checks_push assert is_err text_utf8_decode_next data byte_len 4
            Option::None:
                checks_push checks_new Result::Err "missing byte buffer"
    io_bytebuf_free bytes
    let shown checks_print_report checks
    checks_exit_code shown
```

## text_utf8_encode_char_returns_bytebuf

`text_utf8_encode_char` が `char` を UTF-8 `ByteBuf` として返し、既存 checked conversion と接続できることを確認します。

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: mlstr:
    ##: Checked [ok]
    ##: [0] ok
```neplg2
#entry main
#indent 4
#target std

#import "std/text" as *
#import "std/text/decode" as *
#import "std/test" as *
#import "alloc/io" as *
#import "core/result" as *

fn main %impure fn void i32 \void:
    let mut checks checks_new
    match text_utf8_encode_char 'あ':
        Result::Err _e:
            set checks checks_push checks Result::Err "encode failed"
        Result::Ok bytes:
            match text_bytebuf_to_utf8_str_result bytes:
                Result::Err _e:
                    set checks checks_push checks Result::Err "encoded bytes rejected"
                Result::Ok text:
                    set checks checks_push checks check_str_eq "あ" text
    let shown checks_print_report checks
    checks_exit_code shown
```

## bytebuf_to_utf8_str_rejects_invalid_leading_byte

このケースは、continuation byte 単体を `str` に変換しないことを確認します。
source loader が byte offset / span の前提を壊す入力を境界で拒否するための回帰テストです。

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: mlstr:
    ##: Checked [ok]
    ##: [0] ok
```neplg2
#entry main
#indent 4
#target std

#import "std/text" as *
#import "std/test" as *
#import "alloc/io" as *
#import "core/mem" as *
#import "core/result" as *

fn main %impure fn void i32 \void:
    let mut checks checks_new;
    match io_bytebuf_alloc_region 1:
        Result::Err _e:
            set checks checks_push checks Result::Err "alloc failed"
        Result::Ok region:
            let data %MemPtr u8 region_ptr &region
            match store_u8 data 128:
                Result::Err e:
                    match dealloc_region<u8> region:
                        Result::Ok _:
                            unit
                        Result::Err _e:
                            unit;
                    set checks checks_push checks Result::Err e
                Result::Ok _:
                    match text_bytebuf_to_utf8_str_result io_bytebuf_finish_region region 1:
                        Result::Ok text:
                            set checks checks_push checks Result::Err text
                        Result::Err e:
                            set checks checks_push checks check_str_eq "InvalidUtf8" std_error_kind_str e;
    let shown checks_print_report checks;
    checks_exit_code shown
```

## io_bytebuf_to_str_result_rejects_invalid_utf8

このケースは、`alloc/io` の `ByteBuf -> str` 境界そのものが invalid UTF-8 を拒否することを確認します。
raw bytes が必要な場合は `ByteBuf` のまま扱い、`str` に変換する経路では UTF-8 保証を破らないための回帰テストです。

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: mlstr:
    ##: Checked [ok]
    ##: [0] ok
```neplg2
#entry main
#indent 4
#target std

#import "std/test" as *
#import "alloc/io" as *
#import "core/mem" as *
#import "core/result" as *

fn main %impure fn void i32 \void:
    let mut checks checks_new;
    match io_bytebuf_alloc_region 1:
        Result::Err _e:
            set checks checks_push checks Result::Err "alloc failed"
        Result::Ok region:
            let data %MemPtr u8 region_ptr &region
            match store_u8 data 128:
                Result::Err e:
                    match dealloc_region<u8> region:
                        Result::Ok _:
                            unit
                        Result::Err _e:
                            unit;
                    set checks checks_push checks Result::Err e
                Result::Ok _:
                    match io_bytebuf_to_str_result io_bytebuf_finish_region region 1:
                        Result::Ok text:
                            set checks checks_push checks Result::Err text
                        Result::Err e:
                            set checks checks_push checks check_str_eq "InvalidUtf8" std_error_kind_str e;
    let shown checks_print_report checks;
    checks_exit_code shown
```

## bytebuf_to_utf8_str_rejects_overlong_sequence

このケースは、overlong encoding を continuation byte の個数だけで受け入れないことを確認します。
UTF-8 scalar value の制約まで含めて検証するための回帰テストです。

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: mlstr:
    ##: Checked [ok]
    ##: [0] ok
```neplg2
#entry main
#indent 4
#target std

#import "std/text" as *
#import "std/test" as *
#import "alloc/io" as *
#import "core/mem" as *
#import "core/result" as *

fn main %impure fn void i32 \void:
    let mut checks checks_new;
    match io_bytebuf_alloc_region 3:
        Result::Err _e:
            set checks checks_push checks Result::Err "alloc failed"
        Result::Ok region:
            let data %MemPtr u8 region_ptr &region
            let mut ok %Result unit str Result::Ok unit
            match store_u8 data 224:
                Result::Err e:
                    set ok Result::Err e
                Result::Ok _:
                    match region_ptr_at<u8,u8> &region 1:
                        Result::Err e:
                            set ok Result::Err e
                        Result::Ok p1:
                            match store_u8 p1 128:
                                Result::Err e:
                                    set ok Result::Err e
                                Result::Ok _:
                                    match region_ptr_at<u8,u8> &region 2:
                                        Result::Err e:
                                            set ok Result::Err e
                                        Result::Ok p2:
                                            match store_u8 p2 128:
                                                Result::Err e:
                                                    set ok Result::Err e
                                                Result::Ok _:
                                                    unit
            match ok:
                Result::Err e:
                    match dealloc_region<u8> region:
                        Result::Ok _:
                            unit
                        Result::Err _e:
                            unit;
                    set checks checks_push checks Result::Err e
                Result::Ok _:
                    match text_bytebuf_to_utf8_str_result io_bytebuf_finish_region region 3:
                        Result::Ok text:
                            set checks checks_push checks Result::Err text
                        Result::Err e:
                            set checks checks_push checks check_str_eq "InvalidUtf8" std_error_kind_str e;
    let shown checks_print_report checks;
    checks_exit_code shown
```

## fs_read_to_string_checked_rejects_invalid_utf8

このケースは、file read の checked text API が invalid UTF-8 を errno 84 として拒否することを確認します。
binary 読み込みは `ByteBuf` のまま扱い、source text 読み込みだけを checked API に寄せるための回帰テストです。

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
#import "core/mem" as *
#import "core/result" as *

fn main %impure fn void i32 \void:
    let mut checks checks_new;
    let path %str "tmp/fs_invalid_utf8_checked_case.bin"
    match io_bytebuf_alloc_region 1:
        Result::Err _e:
            set checks checks_push checks Result::Err "alloc failed"
        Result::Ok region:
            let data %MemPtr u8 region_ptr &region
            match store_u8 data 128:
                Result::Err e:
                    match dealloc_region<u8> region:
                        Result::Ok _:
                            unit
                        Result::Err _e:
                            unit;
                    set checks checks_push checks Result::Err e
                Result::Ok _:
                    match fs_write_to_bytes path io_bytebuf_finish_region region 1:
                        Result::Err _e:
                            set checks checks_push checks Result::Err "write failed"
                        Result::Ok _:
                            match fs_read_to_string_checked path:
                                Result::Ok text:
                                    set checks checks_push checks Result::Err text
                                Result::Err e:
                                    set checks checks_push checks check_eq_i32 84 e;
    let shown checks_print_report checks;
    checks_exit_code shown
```

## fs_read_to_string_rejects_invalid_utf8

このケースは、通常の `fs_read_to_string` も invalid UTF-8 を errno 84 として拒否することを確認します。
`str` 型の UTF-8 保証をファイル読み込みの標準入口でも守るための回帰テストです。

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
#import "core/mem" as *
#import "core/result" as *

fn main %impure fn void i32 \void:
    let mut checks checks_new;
    let path %str "tmp/fs_invalid_utf8_default_case.bin"
    match io_bytebuf_alloc_region 1:
        Result::Err _e:
            set checks checks_push checks Result::Err "alloc failed"
        Result::Ok region:
            let data %MemPtr u8 region_ptr &region
            match store_u8 data 128:
                Result::Err e:
                    match dealloc_region<u8> region:
                        Result::Ok _:
                            unit
                        Result::Err _e:
                            unit;
                    set checks checks_push checks Result::Err e
                Result::Ok _:
                    match fs_write_to_bytes path io_bytebuf_finish_region region 1:
                        Result::Err _e:
                            set checks checks_push checks Result::Err "write failed"
                        Result::Ok _:
                            match fs_read_to_string path:
                                Result::Ok text:
                                    set checks checks_push checks Result::Err text
                                Result::Err e:
                                    set checks checks_push checks check_eq_i32 84 e;
    let shown checks_print_report checks;
    checks_exit_code shown
```

## io_read_bytes_target_rejects_invalid_utf8_text

このケースは、`std/io` の `ReadStream::Bytes` を text として読むときにも checked conversion が使われることを確認します。
target facade 経由で unchecked `str` が作られる経路を残さないための回帰テストです。

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: mlstr:
    ##: Checked [ok]
    ##: [0] ok
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

fn main %impure fn void i32 \void:
    let mut checks checks_new;
    match io_bytebuf_alloc_region 1:
        Result::Err _e:
            set checks checks_push checks Result::Err "alloc failed"
        Result::Ok region:
            let data %MemPtr u8 region_ptr &region
            match store_u8 data 128:
                Result::Err e:
                    match dealloc_region<u8> region:
                        Result::Ok _:
                            unit
                        Result::Err _e:
                            unit;
                    set checks checks_push checks Result::Err e
                Result::Ok _:
                    let target %ReadStream ReadStream::Bytes io_bytebuf_finish_region region 1
                    let text_result %Result str StdErrorKind read target;
                    match text_result:
                        Result::Ok text:
                            set checks checks_push checks Result::Err text
                        Result::Err e:
                            set checks checks_push checks check_str_eq "InvalidUtf8" std_error_kind_str e;
    let shown checks_print_report checks;
    checks_exit_code shown
```
