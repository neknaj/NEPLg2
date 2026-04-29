# bytebuf_result.n.md

## io_bytebuf_result_roundtrip_preserves_bytes

neplg2:test
ret: 1
```neplg2
#entry main
#indent 4
#target std

#import "alloc/io" as *
#import "alloc/string" as *

fn main <()*>i32> ():
    match io_bytebuf_from_str_result "A\x00B":
        Result::Ok bytes:
            match io_bytebuf_to_str_result bytes:
                Result::Ok text:
                    if str_eq text "A\x00B" 1 0
                Result::Err _e:
                    0
        Result::Err _e:
            0
```

## io_bytebuf_to_str_result_accepts_empty_buffer

neplg2:test
ret: 1
```neplg2
#entry main
#indent 4
#target std

#import "alloc/io" as *
#import "alloc/string" as *

fn main <()*>i32> ():
    match io_bytebuf_to_str_result io_bytebuf_empty:
        Result::Ok text:
            if str_eq text "" 1 0
        Result::Err _e:
            0
```

## io_bytebuf_to_str_result_reports_allocation_failure

neplg2:test
ret: 1
```neplg2
#entry main
#indent 4
#target std

#import "alloc/diag/error" as *
#import "alloc/io" as *
#import "alloc/string" as *
#import "core/mem" as *

fn main <()*>i32> ():
    let huge <ByteBuf> io_bytebuf_from_owned_ptr mem_ptr_wrap 0 2147483647;
    match io_bytebuf_to_str_result huge:
        Result::Ok _text:
            0
        Result::Err kind:
            if str_eq std_error_kind_str kind "OutOfMemory" 1 0
```

## std_io_text_read_propagates_bytebuf_result

neplg2:test
ret: 1
```neplg2
#entry main
#indent 4
#target std

#import "alloc/diag/error" as *
#import "alloc/io" as *
#import "alloc/string" as *
#import "core/result" as *
#import "std/io" as *
#import "std/iotarget" as *

fn main <()*>i32> ():
    let r <Result<ByteBuf, StdErrorKind>> read ReadStream::Text "abc";
    match r:
        Result::Ok bytes:
            match io_bytebuf_to_str_result bytes:
                Result::Ok text:
                    if str_eq text "abc" 1 0
                Result::Err _e:
                    0
        Result::Err _e:
            0
```

## fs_bytes_to_string_result_reports_allocation_failure

neplg2:test
ret: 1
```neplg2
#entry main
#indent 4
#target std

#import "alloc/io" as *
#import "core/mem" as *
#import "std/fs" as *

fn main <()*>i32> ():
    let huge <ByteBuf> io_bytebuf_from_owned_ptr mem_ptr_wrap 0 2147483647;
    match fs_bytes_to_string_result huge:
        Result::Ok _text:
            0
        Result::Err errno:
            if eq errno 12 1 0
```

## stream_bytes_result_roundtrip_preserves_bytes

neplg2:test
ret: 1
```neplg2
#entry main
#indent 4
#target std

#import "alloc/string" as *
#import "std/streamio" as *

fn main <()*>i32> ():
    match stream_bytes_from_str_result "CD":
        Result::Ok bytes:
            match stream_bytes_to_str_result bytes:
                Result::Ok text:
                    if str_eq text "CD" 1 0
                Result::Err _e:
                    0
        Result::Err _e:
            0
```
