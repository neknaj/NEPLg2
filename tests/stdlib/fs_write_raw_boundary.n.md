# std/fs/write raw boundary regression

## std/fs/write は raw MemPtr span writer を公開しない

neplg2:test[compile_fail]
diag_code: resolve.identifier.undefined
```neplg2
#entry main
#indent 4
#target std

#import "core/result" as *
#import "std/fs/write" as *

fn main <()*>i32> ():
    match fs_write_fd_mem_result 1 0 0:
        Result::Ok _:
            0
        Result::Err _:
            1
```

## std/fs/write/fd を直接 import しても raw span writer は見えない

neplg2:test[compile_fail]
diag_code: resolve.identifier.undefined
```neplg2
#entry main
#indent 4
#target std

#import "core/result" as *
#import "std/fs/write/fd" as *

fn main <()*>i32> ():
    match fs_write_fd_mem_result 1 0 0:
        Result::Ok _:
            0
        Result::Err _:
            1
```
