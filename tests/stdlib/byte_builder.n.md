# ByteBuilder

## byte_builder_push_u8_builds_wasm_header

このケースは、`ByteBuilder` が byte を順に追加し、`finish` で exact-size の `ByteBuf` を返すことを確認します。
WASM emitter が raw memory へ直接書かずに binary header を組み立てるための回帰テストです。

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"byte_builder_push_u8_builds_wasm_header\" count=9 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"header length\" expected=\"8\" actual=\"8\" message=\"\"\nassertion index=1 status=ok kind=eq_i32 label=\"header byte 0\" expected=\"0\" actual=\"0\" message=\"\"\nassertion index=2 status=ok kind=eq_i32 label=\"header byte a\" expected=\"97\" actual=\"97\" message=\"\"\nassertion index=3 status=ok kind=eq_i32 label=\"header byte s\" expected=\"115\" actual=\"115\" message=\"\"\nassertion index=4 status=ok kind=eq_i32 label=\"header byte m\" expected=\"109\" actual=\"109\" message=\"\"\nassertion index=5 status=ok kind=eq_i32 label=\"header version\" expected=\"1\" actual=\"1\" message=\"\"\nassertion index=6 status=ok kind=eq_i32 label=\"header v0\" expected=\"0\" actual=\"0\" message=\"\"\nassertion index=7 status=ok kind=eq_i32 label=\"header v1\" expected=\"0\" actual=\"0\" message=\"\"\nassertion index=8 status=ok kind=eq_i32 label=\"header v2\" expected=\"0\" actual=\"0\" message=\"\"\n"
```neplg2
#entry main
#indent 4
#target std

#import "std/test" as *
#import "alloc/io" as *
#import "core/mem" as *
#import "core/mem/internal" as *
#import "core/mem/allocator" as *
#import "core/mem/raw" as *
#import "core/option" as *
#import "core/result" as *

fn byte_at_or_neg1 %fn &ByteBuf fn i32 i32 \bytes\idx:
    match io_bytebuf_byte_at bytes idx:
        Option::Some actual:
            actual
        Option::None:
            -1

fn main %impure fn void i32 \void:
    let mut built_len %i32 -1;
    let mut byte0 %i32 -1;
    let mut byte1 %i32 -1;
    let mut byte2 %i32 -1;
    let mut byte3 %i32 -1;
    let mut byte4 %i32 -1;
    let mut byte5 %i32 -1;
    let mut byte6 %i32 -1;
    let mut byte7 %i32 -1;
    match byte_builder_with_capacity 1:
        Result::Err _e:
            unit
        Result::Ok b0:
            match byte_builder_push_u8 b0 0:
                Result::Err e:
                    byte_builder_error_free e
                    unit
                Result::Ok b1:
                    match byte_builder_push_u8 b1 'a':
                        Result::Err e:
                            byte_builder_error_free e
                            unit
                        Result::Ok b2:
                            match byte_builder_push_u8 b2 's':
                                Result::Err e:
                                    byte_builder_error_free e
                                    unit
                                Result::Ok b3:
                                    match byte_builder_push_u8 b3 'm':
                                        Result::Err e:
                                            byte_builder_error_free e
                                            unit
                                        Result::Ok b4:
                                            match byte_builder_push_u8 b4 1:
                                                Result::Err e:
                                                    byte_builder_error_free e
                                                    unit
                                                Result::Ok b5:
                                                    match byte_builder_push_u8 b5 0:
                                                        Result::Err e:
                                                            byte_builder_error_free e
                                                            unit
                                                        Result::Ok b6:
                                                            match byte_builder_push_u8 b6 0:
                                                                Result::Err e:
                                                                    byte_builder_error_free e
                                                                    unit
                                                                Result::Ok b7:
                                                                    match byte_builder_push_u8 b7 0:
                                                                        Result::Err e:
                                                                            byte_builder_error_free e
                                                                            unit
                                                                        Result::Ok b8:
                                                                            match byte_builder_finish b8:
                                                                                Result::Err e:
                                                                                    byte_builder_error_free e
                                                                                    unit
                                                                                Result::Ok bytes:
                                                                                    set built_len io_bytebuf_len_ref &bytes;
                                                                                    set byte0 byte_at_or_neg1 &bytes 0;
                                                                                    set byte1 byte_at_or_neg1 &bytes 1;
                                                                                    set byte2 byte_at_or_neg1 &bytes 2;
                                                                                    set byte3 byte_at_or_neg1 &bytes 3;
                                                                                    set byte4 byte_at_or_neg1 &bytes 4;
                                                                                    set byte5 byte_at_or_neg1 &bytes 5;
                                                                                    set byte6 byte_at_or_neg1 &bytes 6;
                                                                                    set byte7 byte_at_or_neg1 &bytes 7;
                                                                                    io_bytebuf_free bytes;
    let report:
        test_report_new "byte_builder_push_u8_builds_wasm_header"
        |> test_report_push assert_eq_i32 "header length" 8 built_len
        |> test_report_push assert_eq_i32 "header byte 0" 0 byte0
        |> test_report_push assert_eq_i32 "header byte a" 97 byte1
        |> test_report_push assert_eq_i32 "header byte s" 115 byte2
        |> test_report_push assert_eq_i32 "header byte m" 109 byte3
        |> test_report_push assert_eq_i32 "header version" 1 byte4
        |> test_report_push assert_eq_i32 "header v0" 0 byte5
        |> test_report_push assert_eq_i32 "header v1" 0 byte6
        |> test_report_push assert_eq_i32 "header v2" 0 byte7
    let shown test_report_print_stdout report
    test_report_exit_code shown
```

## byte_builder_push_leb_u32_known_vector

このケースは、unsigned LEB128 の代表的な known vector `624485 -> E5 8E 26` を確認します。
WASM section size / index encoding の基礎を固定するための回帰テストです。

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"byte_builder_push_leb_u32_known_vector\" count=4 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"leb length\" expected=\"3\" actual=\"3\" message=\"\"\nassertion index=1 status=ok kind=eq_i32 label=\"leb byte 0\" expected=\"229\" actual=\"229\" message=\"\"\nassertion index=2 status=ok kind=eq_i32 label=\"leb byte 1\" expected=\"142\" actual=\"142\" message=\"\"\nassertion index=3 status=ok kind=eq_i32 label=\"leb byte 2\" expected=\"38\" actual=\"38\" message=\"\"\n"
```neplg2
#entry main
#indent 4
#target std

#import "std/test" as *
#import "alloc/io" as *
#import "core/mem" as *
#import "core/mem/internal" as *
#import "core/mem/allocator" as *
#import "core/mem/raw" as *
#import "core/option" as *
#import "core/result" as *

fn byte_at_or_neg1 %fn &ByteBuf fn i32 i32 \bytes\idx:
    match io_bytebuf_byte_at bytes idx:
        Option::Some actual:
            actual
        Option::None:
            -1

fn main %impure fn void i32 \void:
    let mut built_len %i32 -1;
    let mut byte0 %i32 -1;
    let mut byte1 %i32 -1;
    let mut byte2 %i32 -1;
    match byte_builder_new:
        Result::Err _e:
            unit
        Result::Ok b0:
            match byte_builder_push_leb_u32 b0 624485:
                Result::Err e:
                    byte_builder_error_free e
                    unit
                Result::Ok b1:
                    match byte_builder_finish b1:
                        Result::Err e:
                            byte_builder_error_free e
                            unit
                        Result::Ok bytes:
                            set built_len io_bytebuf_len_ref &bytes;
                            set byte0 byte_at_or_neg1 &bytes 0;
                            set byte1 byte_at_or_neg1 &bytes 1;
                            set byte2 byte_at_or_neg1 &bytes 2;
                            io_bytebuf_free bytes;
    let report:
        test_report_new "byte_builder_push_leb_u32_known_vector"
        |> test_report_push assert_eq_i32 "leb length" 3 built_len
        |> test_report_push assert_eq_i32 "leb byte 0" 229 byte0
        |> test_report_push assert_eq_i32 "leb byte 1" 142 byte1
        |> test_report_push assert_eq_i32 "leb byte 2" 38 byte2
    let shown test_report_print_stdout report
    test_report_exit_code shown
```

## byte_builder_growth_preserves_existing_bytes

このケースは、capacity を超えて growth したあとも既存 byte が保持されることを確認します。
public `ByteBuf` からの追加で capacity を超えさせ、emitter が途中の realloc で前半を壊さないことを確認する回帰テストです。

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"byte_builder_growth_preserves_existing_bytes\" count=4 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"growth length\" expected=\"10\" actual=\"10\" message=\"\"\nassertion index=1 status=ok kind=eq_i32 label=\"growth byte A\" expected=\"65\" actual=\"65\" message=\"\"\nassertion index=2 status=ok kind=eq_i32 label=\"growth byte E\" expected=\"69\" actual=\"69\" message=\"\"\nassertion index=3 status=ok kind=eq_i32 label=\"growth byte J\" expected=\"74\" actual=\"74\" message=\"\"\n"
```neplg2
#entry main
#indent 4
#target std

#import "std/test" as *
#import "alloc/io" as *
#import "core/option" as *
#import "core/result" as *

fn byte_at_or_neg1 %fn &ByteBuf fn i32 i32 \bytes\idx:
    match io_bytebuf_byte_at bytes idx:
        Option::Some actual:
            actual
        Option::None:
            -1

fn main %impure fn void i32 \void:
    let mut built_len %i32 -1;
    let mut byte0 %i32 -1;
    let mut byte4 %i32 -1;
    let mut byte9 %i32 -1;
    match byte_builder_with_capacity 2:
        Result::Err _e:
            unit
        Result::Ok b0:
            match io_bytebuf_from_str_result "ABCDEFGHIJ":
                Result::Err _e:
                    byte_builder_free b0
                Result::Ok src:
                    match byte_builder_push_bytebuf b0 src:
                        Result::Err e:
                            byte_builder_bytebuf_error_free e
                            unit
                        Result::Ok b1:
                            match byte_builder_finish b1:
                                Result::Err e:
                                    byte_builder_error_free e
                                    unit
                                Result::Ok bytes:
                                    set built_len io_bytebuf_len_ref &bytes;
                                    set byte0 byte_at_or_neg1 &bytes 0;
                                    set byte4 byte_at_or_neg1 &bytes 4;
                                    set byte9 byte_at_or_neg1 &bytes 9;
                                    io_bytebuf_free bytes;
    let report:
        test_report_new "byte_builder_growth_preserves_existing_bytes"
        |> test_report_push assert_eq_i32 "growth length" 10 built_len
        |> test_report_push assert_eq_i32 "growth byte A" 65 byte0
        |> test_report_push assert_eq_i32 "growth byte E" 69 byte4
        |> test_report_push assert_eq_i32 "growth byte J" 74 byte9
    let shown test_report_print_stdout report
    test_report_exit_code shown
```
