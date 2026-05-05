# io facade

## io_stdio_text_roundtrip

neplg2:test
stdin: "io facade text"
stdout: "io facade text"
```neplg2
#entry main
#indent 4
#target std

#import "std/io" as *
#import "core/result" as *

fn main <()*>i32> ():
    let input <ReadStream> ReadStream::Stdio
    let output <WriteStream> WriteStream::Stdio
    let text0 <Result<str, StdErrorKind>> read input
    match text0:
        Result::Ok text:
            match write output text:
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

## io_stdio_pipe_bytes

neplg2:test
stdout: "pipe bytes\n"
```neplg2
#entry main
#indent 4
#target std

#import "std/io" as *
#import "std/streamio" as *
#import "core/result" as *

fn main <()*>i32> ():
    let output <WriteStream> WriteStream::Stdio
    let bytes0 <ByteBuf> stream_bytes_from_str "pipe bytes\n"
    match bytes0 |> write output:
        Result::Ok out:
            match flush out:
                Result::Ok _:
                    0
                Result::Err _e:
                    1
        Result::Err _e:
            1
```

## io_fs_missing_file_is_io_error

neplg2:test
```neplg2
#entry main
#indent 4
#target std

#import "std/io" as *
#import "std/test" as *

fn main <()*>i32> ():
    let mut checks checks_new
    let missing <ReadStream> ReadStream::Fs "__definitely_missing_file__.txt"
    let result0 <Result<str, StdErrorKind>> read missing
    match result0:
        Result::Ok text:
            let msg <str> concat "io fs read unexpectedly succeeded: " text
            set checks checks_push checks Result<(),str>::Err msg
        Result::Err kind:
            set checks checks_push checks check_str_eq "IoError" std_error_kind_str kind
    let shown checks_print_report checks
    checks_exit_code shown
```

## io_text_target_reads_like_other_sources

neplg2:test
stdout: "literal source"
```neplg2
#entry main
#indent 4
#target std

#import "std/io" as *
#import "core/result" as *

fn main <()*>i32> ():
    let target <ReadStream> ReadStream::Text "literal source"
    let output <WriteStream> WriteStream::Stdio
    let text0 <Result<str, StdErrorKind>> read target
    match text0:
        Result::Ok text:
            match write output text:
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

## io_read_rejects_write_target_at_compile_time

neplg2:test[compile_fail]
```neplg2
#entry main
#indent 4
#target std

#import "std/io" as *

fn main <()*>i32> ():
    let _text <Result<str, StdErrorKind>> read WriteStream::Stdio
    0
```

## io_write_rejects_read_target_at_compile_time

neplg2:test[compile_fail]
```neplg2
#entry main
#indent 4
#target std

#import "std/io" as *

fn main <()*>i32> ():
    match write ReadStream::Stdio "x":
        Result::Ok _:
            0
        Result::Err _e:
            1
```
