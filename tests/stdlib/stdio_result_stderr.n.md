# stdio Result stderr

## stderr_stream_write_str_separates_stdout

neplg2:test[normalize_newlines]
stdout: "artifact\n"
stderr: "diagnostic\n"
```neplg2
#entry main
#indent 4
#target std

#import "std/streamio" as *
#import "core/result" as *

fn main %impure fn unit i32 \unit:
    match write_str_result StdoutStream unit "artifact\n":
        Result::Ok out:
            match flush_result out:
                Result::Ok _:
                    match write_str_result StderrStream unit "diagnostic\n":
                        Result::Ok err:
                            match flush_result err:
                                Result::Ok _:
                                    0
                                Result::Err _e:
                                    4
                        Result::Err _e:
                            3
                Result::Err _e:
                    2
        Result::Err _e:
            1
```

## io_facade_write_stderr_target

neplg2:test[normalize_newlines]
stdout: "json\n"
stderr: "error: bad input\n"
```neplg2
#entry main
#indent 4
#target std

#import "std/io" as *
#import "core/result" as *

fn main %impure fn unit i32 \unit:
    match writeln WriteStream::Stdio "json":
        Result::Ok out:
            match writeln WriteStream::Stderr "error: bad input":
                Result::Ok err:
                    match flush out:
                        Result::Ok _:
                            match flush err:
                                Result::Ok _:
                                    0
                                Result::Err _e:
                                    5
                        Result::Err _e:
                            4
                Result::Err _e:
                    3
        Result::Err _e:
            2
```

## stdio_write_invalid_fd_returns_io_error

neplg2:test
```neplg2
#entry main
#indent 4
#target std

#import "std/stdio" as *
#import "alloc/string" as *
#import "alloc/string/storage" as *
#import "core/mem" as *
#import "core/result" as *

fn main %impure fn unit i32 \unit:
    let text %str "x"
    match stdio_write_fd_str_result 9999 text:
        Result::Ok _:
            1
        Result::Err kind:
            if str_eq std_error_kind_str kind "IoError" 0 2
```
