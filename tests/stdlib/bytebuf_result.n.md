# bytebuf_result.n.md

## io_bytebuf_result_roundtrip_preserves_bytes

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"io_bytebuf_result_roundtrip_preserves_bytes\" count=1 failed=0\nassertion index=0 status=ok kind=bool label=\"roundtrip preserves bytes\" expected=\"true\" actual=\"true\" message=\"\"\n"
```neplg2
#entry main
#indent 4
#target std

#import "alloc/io" as *
#import "alloc/string" as *
#import "core/result" as *
#import "std/test" as *

fn main <()*>i32> ():
    let ok <bool> match io_bytebuf_from_str_result "A\x00B":
        Result::Ok bytes:
            match io_bytebuf_to_str_result bytes:
                Result::Ok text:
                    str_eq text "A\x00B"
                Result::Err _e:
                    false
        Result::Err _e:
            false
    let report:
        test_report_new "io_bytebuf_result_roundtrip_preserves_bytes"
        |> test_report_push assert "roundtrip preserves bytes" ok
    let shown test_report_print_stdout report
    test_report_exit_code shown
```

## io_bytebuf_to_str_result_accepts_empty_buffer

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"io_bytebuf_to_str_result_accepts_empty_buffer\" count=1 failed=0\nassertion index=0 status=ok kind=bool label=\"empty buffer to empty string\" expected=\"true\" actual=\"true\" message=\"\"\n"
```neplg2
#entry main
#indent 4
#target std

#import "alloc/io" as *
#import "alloc/string" as *
#import "core/result" as *
#import "std/test" as *

fn main <()*>i32> ():
    let ok <bool> match io_bytebuf_to_str_result io_bytebuf_empty:
        Result::Ok text:
            str_eq text ""
        Result::Err _e:
            false
    let report:
        test_report_new "io_bytebuf_to_str_result_accepts_empty_buffer"
        |> test_report_push assert "empty buffer to empty string" ok
    let shown test_report_print_stdout report
    test_report_exit_code shown
```

## io_bytebuf_to_str_result_reports_invalid_utf8

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"io_bytebuf_to_str_result_reports_invalid_utf8\" count=1 failed=0\nassertion index=0 status=ok kind=bool label=\"invalid utf8 reports error\" expected=\"true\" actual=\"true\" message=\"\"\n"
```neplg2
#entry main
#indent 4
#target std

#import "alloc/diag/error" as *
#import "alloc/io" as *
#import "alloc/string" as *
#import "core/result" as *
#import "std/test" as *

fn main <()*>i32> ():
    let ok <bool> match byte_builder_new:
        Result::Ok b0:
            match byte_builder_push_u8 b0 255:
                Result::Ok b1:
                    match byte_builder_finish b1:
                        Result::Ok bytes:
                            match io_bytebuf_to_str_result bytes:
                                Result::Ok _text:
                                    false
                                Result::Err kind:
                                    str_eq std_error_kind_str kind "InvalidUtf8"
                        Result::Err e:
                            byte_builder_error_free e
                            false
                Result::Err e:
                    byte_builder_error_free e
                    false
        Result::Err kind:
            false
    let report:
        test_report_new "io_bytebuf_to_str_result_reports_invalid_utf8"
        |> test_report_push assert "invalid utf8 reports error" ok
    let shown test_report_print_stdout report
    test_report_exit_code shown
```

## std_io_text_read_propagates_bytebuf_result

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"std_io_text_read_propagates_bytebuf_result\" count=1 failed=0\nassertion index=0 status=ok kind=bool label=\"text read propagates bytebuf result\" expected=\"true\" actual=\"true\" message=\"\"\n"
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
#import "std/test" as *

fn main <()*>i32> ():
    let ok <bool> match read ReadStream::Text "abc":
        Result::Ok bytes:
            match io_bytebuf_to_str_result bytes:
                Result::Ok text:
                    str_eq text "abc"
                Result::Err _e:
                    false
        Result::Err _e:
            false
    let report:
        test_report_new "std_io_text_read_propagates_bytebuf_result"
        |> test_report_push assert "text read propagates bytebuf result" ok
    let shown test_report_print_stdout report
    test_report_exit_code shown
```

## fs_bytes_to_string_result_reports_invalid_utf8

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"fs_bytes_to_string_result_reports_invalid_utf8\" count=1 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"fs invalid utf8 reports ilseq\" expected=\"84\" actual=\"84\" message=\"\"\n"
```neplg2
#entry main
#indent 4
#target std

#import "alloc/io" as *
#import "std/fs" as *
#import "core/result" as *
#import "std/test" as *

fn main <()*>i32> ():
    let errno <i32> match byte_builder_new:
        Result::Ok b0:
            match byte_builder_push_u8 b0 255:
                Result::Ok b1:
                    match byte_builder_finish b1:
                        Result::Ok bytes:
                            match fs_bytes_to_string_result bytes:
                                Result::Ok _text:
                                    0
                                Result::Err e:
                                    e
                        Result::Err e:
                            byte_builder_error_free e
                            0
                Result::Err e:
                    byte_builder_error_free e
                    0
        Result::Err _e:
            0
    let report:
        test_report_new "fs_bytes_to_string_result_reports_invalid_utf8"
        |> test_report_push assert_eq_i32 "fs invalid utf8 reports ilseq" 84 errno
    let shown test_report_print_stdout report
    test_report_exit_code shown
```

## io_bytebuf_rejects_raw_memptr_ownership_forging

neplg2:test[compile_fail]
diag_codes: resolve.identifier.undefined
```neplg2
#entry main
#indent 4
#target std

#import "alloc/io" as *

fn main <()*>i32> ():
    let _buf <ByteBuf> io_bytebuf_from_owned_ptr 0 1
    0
```

## stream_bytes_result_roundtrip_preserves_bytes

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"stream_bytes_result_roundtrip_preserves_bytes\" count=1 failed=0\nassertion index=0 status=ok kind=bool label=\"stream byte roundtrip preserves bytes\" expected=\"true\" actual=\"true\" message=\"\"\n"
```neplg2
#entry main
#indent 4
#target std

#import "alloc/string" as *
#import "core/result" as *
#import "std/streamio" as *
#import "std/test" as *

fn main <()*>i32> ():
    let ok <bool> match stream_bytes_from_str_result "CD":
        Result::Ok bytes:
            match stream_bytes_to_str_result bytes:
                Result::Ok text:
                    str_eq text "CD"
                Result::Err _e:
                    false
        Result::Err _e:
            false
    let report:
        test_report_new "stream_bytes_result_roundtrip_preserves_bytes"
        |> test_report_push assert "stream byte roundtrip preserves bytes" ok
    let shown test_report_print_stdout report
    test_report_exit_code shown
```
