# std/stdio/write raw boundary regression

## std/stdio/write は raw MemPtr span writer を公開しない

neplg2:test[compile_fail]
diag_code: resolve.identifier.undefined
```neplg2
#entry main
#indent 4
#target std

#import "core/result" as *
#import "std/stdio/write" as *

fn main <()*>i32> ():
    match stdio_write_fd_mem_result 1 0 0:
        Result::Ok _:
            0
        Result::Err _:
            1
```

## std/stdio/write/fd を直接 import しても raw span writer は見えない

neplg2:test[compile_fail]
diag_code: resolve.identifier.undefined
```neplg2
#entry main
#indent 4
#target std

#import "core/result" as *
#import "std/stdio/write/fd" as *

fn main <()*>i32> ():
    match stdio_write_mem_result 0 0:
        Result::Ok _:
            0
        Result::Err _:
            1
```
