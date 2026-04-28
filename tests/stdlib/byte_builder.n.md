# ByteBuilder

## byte_builder_push_u8_builds_wasm_header

このケースは、`ByteBuilder` が byte を順に追加し、`finish` で exact-size の `ByteBuf` を返すことを確認します。
WASM emitter が raw memory へ直接書かずに binary header を組み立てるための回帰テストです。

neplg2:test
ret: 0
```neplg2
#entry main
#indent 4
#target std

#import "std/test" as *
#import "alloc/io" as *
#import "core/mem" as *
#import "core/result" as *

fn main <()*>i32> ():
    let mut checks <Vec<Result<(),str>>> checks_new;
    match byte_builder_with_capacity 1:
        Result::Err _e:
            set checks checks_push checks Result<(),str>::Err "builder alloc failed"
        Result::Ok b0:
            match byte_builder_push_u8 b0 0:
                Result::Err _e:
                    set checks checks_push checks Result<(),str>::Err "push 0 failed"
                Result::Ok b1:
                    match byte_builder_push_u8 b1 'a':
                        Result::Err _e:
                            set checks checks_push checks Result<(),str>::Err "push a failed"
                        Result::Ok b2:
                            match byte_builder_push_u8 b2 's':
                                Result::Err _e:
                                    set checks checks_push checks Result<(),str>::Err "push s failed"
                                Result::Ok b3:
                                    match byte_builder_push_u8 b3 'm':
                                        Result::Err _e:
                                            set checks checks_push checks Result<(),str>::Err "push m failed"
                                        Result::Ok b4:
                                            match byte_builder_push_u8 b4 1:
                                                Result::Err _e:
                                                    set checks checks_push checks Result<(),str>::Err "push version failed"
                                                Result::Ok b5:
                                                    match byte_builder_push_u8 b5 0:
                                                        Result::Err _e:
                                                            set checks checks_push checks Result<(),str>::Err "push v0 failed"
                                                        Result::Ok b6:
                                                            match byte_builder_push_u8 b6 0:
                                                                Result::Err _e:
                                                                    set checks checks_push checks Result<(),str>::Err "push v1 failed"
                                                                Result::Ok b7:
                                                                    match byte_builder_push_u8 b7 0:
                                                                        Result::Err _e:
                                                                            set checks checks_push checks Result<(),str>::Err "push v2 failed"
                                                                        Result::Ok b8:
                                                                            match byte_builder_finish b8:
                                                                                Result::Err _e:
                                                                                    set checks checks_push checks Result<(),str>::Err "finish failed"
                                                                                Result::Ok bytes:
                                                                                    let ptr <MemPtr<u8>> get bytes "ptr"
                                                                                    let raw <i32> mem_ptr_addr ptr
                                                                                    set checks checks_push checks check_eq_i32 8 get bytes "len";
                                                                                    set checks checks_push checks check_eq_i32 0 load_u8 raw;
                                                                                    set checks checks_push checks check_eq_i32 'a' load_u8 add raw 1;
                                                                                    set checks checks_push checks check_eq_i32 's' load_u8 add raw 2;
                                                                                    set checks checks_push checks check_eq_i32 'm' load_u8 add raw 3;
                                                                                    set checks checks_push checks check_eq_i32 1 load_u8 add raw 4;
                                                                                    set checks checks_push checks check_eq_i32 0 load_u8 add raw 5;
                                                                                    set checks checks_push checks check_eq_i32 0 load_u8 add raw 6;
                                                                                    set checks checks_push checks check_eq_i32 0 load_u8 add raw 7;
                                                                                    io_bytebuf_free bytes;
    let shown <Vec<Result<(),str>>> checks_print_report checks;
    checks_exit_code shown
```

## byte_builder_push_leb_u32_known_vector

このケースは、unsigned LEB128 の代表的な known vector `624485 -> E5 8E 26` を確認します。
WASM section size / index encoding の基礎を固定するための回帰テストです。

neplg2:test
ret: 0
```neplg2
#entry main
#indent 4
#target std

#import "std/test" as *
#import "alloc/io" as *
#import "core/mem" as *
#import "core/result" as *

fn main <()*>i32> ():
    let mut checks <Vec<Result<(),str>>> checks_new;
    match byte_builder_new:
        Result::Err _e:
            set checks checks_push checks Result<(),str>::Err "builder alloc failed"
        Result::Ok b0:
            match byte_builder_push_leb_u32 b0 624485:
                Result::Err _e:
                    set checks checks_push checks Result<(),str>::Err "leb push failed"
                Result::Ok b1:
                    match byte_builder_finish b1:
                        Result::Err _e:
                            set checks checks_push checks Result<(),str>::Err "finish failed"
                        Result::Ok bytes:
                            let ptr <MemPtr<u8>> get bytes "ptr"
                            let raw <i32> mem_ptr_addr ptr
                            set checks checks_push checks check_eq_i32 3 get bytes "len";
                            set checks checks_push checks check_eq_i32 229 load_u8 raw;
                            set checks checks_push checks check_eq_i32 142 load_u8 add raw 1;
                            set checks checks_push checks check_eq_i32 38 load_u8 add raw 2;
                            io_bytebuf_free bytes;
    let shown <Vec<Result<(),str>>> checks_print_report checks;
    checks_exit_code shown
```

## byte_builder_growth_preserves_existing_bytes

このケースは、capacity を超えて growth したあとも既存 byte が保持されることを確認します。
section を複数段階で組み立てる emitter が途中の realloc で前半を壊さないための回帰テストです。

neplg2:test
ret: 0
```neplg2
#entry main
#indent 4
#target std

#import "std/test" as *
#import "alloc/io" as *
#import "core/mem" as *
#import "core/result" as *

fn main <()*>i32> ():
    let mut checks <Vec<Result<(),str>>> checks_new;
    match alloc_ptr<u8> 10:
        Result::Err _e:
            set checks checks_push checks Result<(),str>::Err "source alloc failed"
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
                    set checks checks_push checks Result<(),str>::Err "builder alloc failed"
                Result::Ok b0:
                    match byte_builder_push_bytes_ref b0 src 10:
                        Result::Err _e:
                            match dealloc_ptr<u8> src 10:
                                Result::Ok _:
                                    ()
                                Result::Err _:
                                    ()
                            set checks checks_push checks Result<(),str>::Err "push bytes failed"
                        Result::Ok b1:
                            match dealloc_ptr<u8> src 10:
                                Result::Ok _:
                                    ()
                                Result::Err _:
                                    ()
                            match byte_builder_finish b1:
                                Result::Err _e:
                                    set checks checks_push checks Result<(),str>::Err "finish failed"
                                Result::Ok bytes:
                                    let ptr <MemPtr<u8>> get bytes "ptr"
                                    let raw <i32> mem_ptr_addr ptr
                                    set checks checks_push checks check_eq_i32 10 get bytes "len";
                                    set checks checks_push checks check_eq_i32 'A' load_u8 raw;
                                    set checks checks_push checks check_eq_i32 'E' load_u8 add raw 4;
                                    set checks checks_push checks check_eq_i32 'J' load_u8 add raw 9;
                                    io_bytebuf_free bytes;
    let shown <Vec<Result<(),str>>> checks_print_report checks;
    checks_exit_code shown
```
