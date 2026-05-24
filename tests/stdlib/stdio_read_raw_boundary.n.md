# std/stdio/read raw boundary regression

## std/stdio/read は raw MemPtr fd_read helper を公開しない

neplg2:test[compile_fail]
diag_code: resolve.identifier.undefined
```neplg2
#entry main
#indent 4
#target std

#import "core/result" as *
#import "std/stdio/read" as *

fn main %impure fn () i32 \():
    match stdio_fd_read_into_result 0 0 0 0 1:
        Result::Ok _:
            0
        Result::Err _:
            1
```

## std/stdio/read/buffer を直接 import しても raw MemPtr fd_read helper は見えない

neplg2:test[compile_fail]
diag_code: resolve.identifier.undefined
```neplg2
#entry main
#indent 4
#target std

#import "core/result" as *
#import "std/stdio/read/buffer" as *

fn main %impure fn () i32 \():
    match stdio_fd_read_into_result 0 0 0 0 1:
        Result::Ok _:
            0
        Result::Err _:
            1
```

## std/stdio/read/buffer は fd_read slice wrapper も公開しない

neplg2:test[compile_fail]
diag_code: resolve.identifier.undefined
```neplg2
#entry main
#indent 4
#target std

#import "core/mem" as *
#import "core/result" as *
#import "std/stdio/read/buffer" as *

fn main %impure fn () i32 \():
    match alloc_region<u8> 8:
        Result::Err _:
            1
        Result::Ok iov_region:
            match alloc_region<u8> 4:
                Result::Err _:
                    match dealloc_region<u8> iov_region:
                        Result::Ok _:
                            ()
                        Result::Err _:
                            ()
                    1
                Result::Ok nread_region:
                    match alloc_region<u8> 1:
                        Result::Err _:
                            match dealloc_region<u8> nread_region:
                                Result::Ok _:
                                    ()
                                Result::Err _:
                                    ()
                            match dealloc_region<u8> iov_region:
                                Result::Ok _:
                                    ()
                                Result::Err _:
                                    ()
                            1
                        Result::Ok data_region:
                            let mut code %i32 1;
                            match stdio_fd_read_region_slice_result 0 &iov_region &nread_region &data_region 0 1:
                                Result::Ok _:
                                    set code 0;
                                Result::Err _:
                                    set code 1;
                            match dealloc_region<u8> data_region:
                                Result::Ok _:
                                    ()
                                Result::Err _:
                                    ()
                            match dealloc_region<u8> nread_region:
                                Result::Ok _:
                                    ()
                                Result::Err _:
                                    ()
                            match dealloc_region<u8> iov_region:
                                Result::Ok _:
                                    ()
                                Result::Err _:
                                    ()
                            code
```
