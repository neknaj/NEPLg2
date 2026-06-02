# std/fs fd raw boundary regression

## std/fs/raw は fd_read raw MemPtr helper を公開しない

neplg2:test[compile_fail]
diag_code: resolve.identifier.undefined
```neplg2
#entry main
#indent 4
#target std

#import "core/result" as *
#import "std/fs/raw" as *

fn main %impure fn void i32 \void:
    match fs_fd_read_into_result 0 0 0 0 1:
        Result::Ok _:
            0
        Result::Err _:
            1
```

## std/fs/read/fd を直接 import しても fd_read raw MemPtr helper は見えない

neplg2:test[compile_fail]
diag_code: resolve.identifier.undefined
```neplg2
#entry main
#indent 4
#target std

#import "core/result" as *
#import "std/fs/read/fd" as *

fn main %impure fn void i32 \void:
    match fs_fd_read_into_result 0 0 0 0 1:
        Result::Ok _:
            0
        Result::Err _:
            1
```

## std/fs/raw は fd_write raw MemPtr helper を公開しない

neplg2:test[compile_fail]
diag_code: resolve.identifier.undefined
```neplg2
#entry main
#indent 4
#target std

#import "core/result" as *
#import "std/fs/raw" as *

fn main %impure fn void i32 \void:
    match fs_fd_write_from_result 1 0 0 0 1:
        Result::Ok _:
            0
        Result::Err _:
            1
```

## std/fs/write/fd を直接 import しても fd_write raw MemPtr helper は見えない

neplg2:test[compile_fail]
diag_code: resolve.identifier.undefined
```neplg2
#entry main
#indent 4
#target std

#import "core/result" as *
#import "std/fs/write/fd" as *

fn main %impure fn void i32 \void:
    match fs_fd_write_from_result 1 0 0 0 1:
        Result::Ok _:
            0
        Result::Err _:
            1
```
