# ByteBuilder

## byte_builder_push_u8_builds_wasm_header

このケースは、`ByteBuilder` が byte を順に追加し、`finish` で exact-size の `ByteBuf` を返すことを確認します。
WASM emitter が raw memory へ直接書かずに binary header を組み立てるための回帰テストです。

neplg2:test[normalize_newlines]
stdout: "byte_builder_push_u8_builds_wasm_header ok\n"
exit_code: 0
```neplg2
#entry main
#indent 4
#target std

#import "core/test" as *
#import "std/stdio" as *
#import "alloc/io" as *
#import "core/option" as *
#import "core/result" as *

fn expect_byte <(&ByteBuf,i32,i32)->()> (bytes, idx, expected):
    match io_bytebuf_byte_at bytes idx:
        Option::Some actual:
            assert_eq_i32 expected actual
        Option::None:
            test_fail "missing byte"

fn main <()*>i32> ():
    match byte_builder_with_capacity 1:
        Result::Err _e:
            test_fail "builder alloc failed";
            1
        Result::Ok b0:
            match byte_builder_push_u8 b0 0:
                Result::Err _e:
                    test_fail "push 0 failed";
                    1
                Result::Ok b1:
                    match byte_builder_push_u8 b1 'a':
                        Result::Err _e:
                            test_fail "push a failed";
                            1
                        Result::Ok b2:
                            match byte_builder_push_u8 b2 's':
                                Result::Err _e:
                                    test_fail "push s failed";
                                    1
                                Result::Ok b3:
                                    match byte_builder_push_u8 b3 'm':
                                        Result::Err _e:
                                            test_fail "push m failed";
                                            1
                                        Result::Ok b4:
                                            match byte_builder_push_u8 b4 1:
                                                Result::Err _e:
                                                    test_fail "push version failed";
                                                    1
                                                Result::Ok b5:
                                                    match byte_builder_push_u8 b5 0:
                                                        Result::Err _e:
                                                            test_fail "push v0 failed";
                                                            1
                                                        Result::Ok b6:
                                                            match byte_builder_push_u8 b6 0:
                                                                Result::Err _e:
                                                                    test_fail "push v1 failed";
                                                                    1
                                                                Result::Ok b7:
                                                                    match byte_builder_push_u8 b7 0:
                                                                        Result::Err _e:
                                                                            test_fail "push v2 failed";
                                                                            1
                                                                        Result::Ok b8:
                                                                            match byte_builder_finish b8:
                                                                                Result::Err _e:
                                                                                    test_fail "finish failed";
                                                                                    1
                                                                                Result::Ok bytes:
                                                                                    assert_eq_i32 8 io_bytebuf_len_ref &bytes;
                                                                                    expect_byte &bytes 0 0;
                                                                                    expect_byte &bytes 1 'a';
                                                                                    expect_byte &bytes 2 's';
                                                                                    expect_byte &bytes 3 'm';
                                                                                    expect_byte &bytes 4 1;
                                                                                    expect_byte &bytes 5 0;
                                                                                    expect_byte &bytes 6 0;
                                                                                    expect_byte &bytes 7 0;
                                                                                    io_bytebuf_free bytes;
                                                                                    println "byte_builder_push_u8_builds_wasm_header ok";
                                                                                    0
```

## byte_builder_push_leb_u32_known_vector

このケースは、unsigned LEB128 の代表的な known vector `624485 -> E5 8E 26` を確認します。
WASM section size / index encoding の基礎を固定するための回帰テストです。

neplg2:test[normalize_newlines]
stdout: "byte_builder_push_leb_u32_known_vector ok\n"
exit_code: 0
```neplg2
#entry main
#indent 4
#target std

#import "core/test" as *
#import "std/stdio" as *
#import "alloc/io" as *
#import "core/option" as *
#import "core/result" as *

fn expect_byte <(&ByteBuf,i32,i32)->()> (bytes, idx, expected):
    match io_bytebuf_byte_at bytes idx:
        Option::Some actual:
            assert_eq_i32 expected actual
        Option::None:
            test_fail "missing byte"

fn main <()*>i32> ():
    match byte_builder_new:
        Result::Err _e:
            test_fail "builder alloc failed";
            1
        Result::Ok b0:
            match byte_builder_push_leb_u32 b0 624485:
                Result::Err _e:
                    test_fail "leb push failed";
                    1
                Result::Ok b1:
                    match byte_builder_finish b1:
                        Result::Err _e:
                            test_fail "finish failed";
                            1
                        Result::Ok bytes:
                            assert_eq_i32 3 io_bytebuf_len_ref &bytes;
                            expect_byte &bytes 0 229;
                            expect_byte &bytes 1 142;
                            expect_byte &bytes 2 38;
                            io_bytebuf_free bytes;
                            println "byte_builder_push_leb_u32_known_vector ok";
                            0
```

## byte_builder_growth_preserves_existing_bytes

このケースは、capacity を超えて growth したあとも既存 byte が保持されることを確認します。
section を複数段階で組み立てる emitter が途中の realloc で前半を壊さないための回帰テストです。

neplg2:test[normalize_newlines]
stdout: "byte_builder_growth_preserves_existing_bytes ok\n"
exit_code: 0
```neplg2
#entry main
#indent 4
#target std

#import "core/test" as *
#import "std/stdio" as *
#import "alloc/io" as *
#import "core/mem" as *
#import "core/option" as *
#import "core/result" as *

fn expect_byte <(&ByteBuf,i32,i32)->()> (bytes, idx, expected):
    match io_bytebuf_byte_at bytes idx:
        Option::Some actual:
            assert_eq_i32 expected actual
        Option::None:
            test_fail "missing byte"

fn main <()*>i32> ():
    match alloc_ptr<u8> 10:
        Result::Err _e:
            test_fail "source alloc failed";
            1
        Result::Ok src:
            let src_raw <i32> mem_ptr_addr src
            store_u8 src_raw 'A';
            store_u8 add src_raw 1 'B';
            store_u8 add src_raw 2 'C';
            store_u8 add src_raw 3 'D';
            store_u8 add src_raw 4 'E';
            store_u8 add src_raw 5 'F';
            store_u8 add src_raw 6 'G';
            store_u8 add src_raw 7 'H';
            store_u8 add src_raw 8 'I';
            store_u8 add src_raw 9 'J';
            match byte_builder_with_capacity 2:
                Result::Err _e:
                    match dealloc_ptr<u8> src 10:
                        Result::Ok _:
                            ()
                        Result::Err _:
                            ()
                    test_fail "builder alloc failed";
                    1
                Result::Ok b0:
                    match byte_builder_push_bytes_ref b0 src 10:
                        Result::Err _e:
                            match dealloc_ptr<u8> src 10:
                                Result::Ok _:
                                    ()
                                Result::Err _:
                                    ()
                            test_fail "push bytes failed";
                            1
                        Result::Ok b1:
                            match dealloc_ptr<u8> src 10:
                                Result::Ok _:
                                    ()
                                Result::Err _:
                                    ()
                            match byte_builder_finish b1:
                                Result::Err _e:
                                    test_fail "finish failed";
                                    1
                                Result::Ok bytes:
                                    assert_eq_i32 10 io_bytebuf_len_ref &bytes;
                                    expect_byte &bytes 0 'A';
                                    expect_byte &bytes 4 'E';
                                    expect_byte &bytes 9 'J';
                                    io_bytebuf_free bytes;
                                    println "byte_builder_growth_preserves_existing_bytes ok";
                                    0
```
