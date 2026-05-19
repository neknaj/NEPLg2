# streamio facade

## stdout_text_writer_via_common_write

neplg2:test
stdout: "stream text\n"
```neplg2
#entry main
#indent 4
#target std

#import "std/streamio" as *
#import "std/iotarget" as *
#import "core/result" as *

fn main <()*>i32> ():
    unwrap_ok open WriteStream::Stdio
    |> write "stream text\n"
    |> flush
    |> close;
    0
```

## stdout_binary_writer_roundtrip

neplg2:test
stdout: "AB\n"
```neplg2
#entry main
#indent 4
#target std

#import "std/streamio" as *
#import "std/iotarget" as *
#import "core/result" as *

fn main <()*>i32> ():
    let bytes0 <ByteBuf> stream_bytes_from_str "AB\n"
    unwrap_ok open WriteStream::Stdio
    |> write bytes0
    |> flush
    |> close;
    0
```

## stdout_binary_writer_preserves_nul

neplg2:test
```neplg2
#entry main
#indent 4
#target std

#import "std/streamio" as *
#import "std/test" as *
#import "alloc/string" as *

fn main <()*>i32> ():
    let bytes0 <ByteBuf> stream_bytes_from_str "A\x00B\n"
    let text <str> stream_bytes_to_str bytes0
    if str_eq text "A\x00B\n" 0 1
```


## stream_writer_text_and_i32

neplg2:test
stdout: "sum=42\n"
```neplg2
#entry main
#indent 4
#target std

#import "std/streamio" as *
#import "std/iotarget" as *
#import "core/result" as *

fn main <()*>i32> ():
    unwrap_ok open WriteStream::Stdio
    |> write "sum="
    |> writeln 42
    |> flush
    |> close;
    0
```

## stream_writer_space_and_i64

neplg2:test
stdout: "1 2\n"
```neplg2
#entry main
#indent 4
#target std

#import "std/streamio" as *
#import "std/iotarget" as *
#import "core/cast" as *
#import "core/result" as *

fn main <()*>i32> ():
    unwrap_ok open WriteStream::Stdio
    |> write 1
    |> write " "
    |> writeln <i64> cast 2
    |> flush
    |> close;
    0
```

## stdin_binary_reader_to_stdout

neplg2:test
stdin: "line1\nline2"
stdout: "line1\nline2"
```neplg2
#entry main
#indent 4
#target std

#import "std/streamio" as *
#import "std/iotarget" as *
#import "core/result" as *

fn read_stdin_bytes <()*>Result<ByteBuf, StdErrorKind>> ():
    read StdinStream ()

fn main <()*>i32> ():
    match read_stdin_bytes:
        Result::Ok bytes:
            match write StdoutStream () bytes:
                Result::Ok out:
                    match flush out:
                        Result::Ok _:
                            0
                        Result::Err _e:
                            1
                Result::Err _e:
                    1
        Result::Err _e:
            1
```

## stdin_text_reader_via_common_read

neplg2:test
stdin: "text via read"
stdout: "text via read"
```neplg2
#entry main
#indent 4
#target std

#import "std/streamio" as *
#import "std/iotarget" as *
#import "core/result" as *

fn read_stdin_text <()*>Result<str, StdErrorKind>> ():
    read StdinStream ()

fn main <()*>i32> ():
    match read_stdin_text:
        Result::Ok text:
            match write StdoutStream () text:
                Result::Ok out:
                    match flush out:
                        Result::Ok _:
                            0
                        Result::Err _e:
                            1
                Result::Err _e:
                    1
        Result::Err _e:
            1
```

## text_literal_stream_reads_like_other_streams

neplg2:test
stdout: "literal stream"
```neplg2
#entry main
#indent 4
#target std

#import "std/streamio" as *
#import "std/iotarget" as *
#import "core/result" as *

fn main <()*>i32> ():
    let input <ReadStream> ReadStream::Text "literal stream"
    let in_stream <TextInputStream> TextInputStream "literal stream"
    let text0 <Result<str, StdErrorKind>> read in_stream
    match text0:
        Result::Ok text:
            match write StdoutStream () text:
                Result::Ok out:
                    match flush out:
                        Result::Ok _:
                            0
                        Result::Err _e:
                            1
                Result::Err _e:
                    1
        Result::Err _e:
            1
```

## stream_scanner_reads_numbers

neplg2:test[normalize_newlines]
stdin: "10 -20 +30 4.5\n"
stdout: "10\n-20\n30\n4.500000\n"
```neplg2
#entry main
#indent 4
#target std

#import "std/streamio" as *
#import "std/iotarget" as *
#import "core/result" as *
fn main <()*>i32> ():
    let input <ReadStream> ReadStream::Stdio;
    let output <WriteStream> WriteStream::Stdio;
    let sc <StreamScanner> unwrap_ok open input;
    let a <i32> read &sc;
    let b <i32> read &sc;
    let c <i64> read &sc;
    let d <f64> read &sc;
    close sc;
    unwrap_ok open output
    |> writeln a
    |> writeln b
    |> writeln c
    |> writeln d
    |> flush
    |> close;
    0
```

## stream_scanner_reads_unsigned_numbers

neplg2:test[normalize_newlines]
stdin: "4294967295 18446744073709551615\n"
stdout: "4294967295\n18446744073709551615\n"
```neplg2
#entry main
#indent 4
#target std

#import "std/streamio" as *
#import "std/iotarget" as *
#import "core/result" as *

fn main <()*>i32> ():
    let input <ReadStream> ReadStream::Stdio;
    let output <WriteStream> WriteStream::Stdio;
    let sc <StreamScanner> unwrap_ok open input;
    let a <u32> read &sc;
    let b <u64> read &sc;
    close sc;
    unwrap_ok open output
    |> writeln a
    |> writeln b
    |> flush
    |> close;
    0
```

## stream_scanner_skips_bom_and_token

neplg2:test[normalize_newlines]
stdin: "\ufeffabc 42\n"
stdout: "abc\n42\n"
```neplg2
#entry main
#indent 4
#target std

#import "std/streamio" as *
#import "std/iotarget" as *
#import "std/stdio" as *
#import "core/result" as *

fn main <()*>i32> ():
    let input <ReadStream> ReadStream::Stdio;
    let sc <StreamScanner> unwrap_ok open input;
    let token <str> read &sc;
    let value <i32> read &sc;
    close sc;
    print token;
    println "";
    println_i32 value;
    0
```

## stream_scanner_rejects_invalid_utf8_token_bytes

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: mlstr:
    ##: Checked [ok]
    ##: [0] ok
```neplg2
#entry main
#indent 4
#target std

#import "std/streamio" as *
#import "std/iotarget" as *
#import "std/test" as *
#import "alloc/io" as *
#import "core/mem" as *
#import "core/result" as *

fn main <()*>i32> ():
    let mut checks checks_new
    match io_bytebuf_alloc_region 1:
        Result::Err _e:
            set checks checks_push checks Result<(),str>::Err "alloc failed"
        Result::Ok region:
            let data <MemPtr<u8>> io_bytebuf_region_ptr &region
            match store_u8 data 128:
                Result::Err e:
                    match dealloc_region<u8> region:
                        Result::Ok _:
                            ()
                        Result::Err _e:
                            ()
                    set checks checks_push checks Result<(),str>::Err e
                Result::Ok _:
                    let sc <StreamScanner> unwrap_ok open ReadStream::Bytes io_bytebuf_finish_region region 1
                    let token <str> read &sc
                    close sc
                    set checks checks_push checks assert_str_eq "" token
    let shown checks_print_report checks
    checks_exit_code shown
```

## stream_scanner_rejects_unsupported_read_type

neplg2:test[compile_fail]
diag_code: type.overload.no_match
```neplg2
#entry main
#indent 4
#target std

#import "std/streamio" as *
#import "std/iotarget" as *
#import "core/result" as *

fn main <()*>i32> ():
    let input <ReadStream> ReadStream::Text "true";
    let sc <StreamScanner> unwrap_ok open input;
    let value <bool> read &sc;
    close sc;
    if value 0 1
```

## stream_scanner_read_requires_borrowed_handle

neplg2:test[compile_fail]
diag_code: type.overload.no_match
```neplg2
#entry main
#indent 4
#target std

#import "std/streamio" as *
#import "std/iotarget" as *
#import "core/result" as *

fn main <()*>i32> ():
    let input <ReadStream> ReadStream::Text "1";
    let sc <StreamScanner> unwrap_ok open input;
    let value <i32> read sc;
    close sc;
    value
```

## stdout_binary_writer_pipe_data_to_target

neplg2:test
stdout: "CD\n"
```neplg2
#entry main
#indent 4
#target std

#import "std/streamio" as *
#import "std/iotarget" as *
#import "core/result" as *

fn main <()*>i32> ():
    let output <WriteStream> WriteStream::Stdio
    let bytes0 <ByteBuf> stream_bytes_from_str "CD\n"
    unwrap_ok open output
    |> write bytes0
    |> flush
    |> close;
    0
```

## stream_scanners_can_coexist

neplg2:test[normalize_newlines]
stdout: "13\n24\n"
```neplg2
#entry main
#indent 4
#target std

#import "std/streamio" as *
#import "std/iotarget" as *
#import "core/math" as *
#import "core/result" as *

fn main <()*>i32> ():
    let left <StreamScanner> unwrap_ok open ReadStream::Text "10 3"
    let right <StreamScanner> unwrap_ok open ReadStream::Text "20 4"
    let a <i32> read &left
    let b <i32> read &left
    let c <i32> read &right
    let d <i32> read &right
    close left;
    close right;
    unwrap_ok open WriteStream::Stdio
    |> writeln add a b
    |> writeln add c d
    |> flush
    |> close;
    0
```
