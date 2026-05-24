# std/env/cliarg raw boundary regression

## std/env/cliarg/raw は argv sizes raw helper を公開しない

neplg2:test[compile_fail]
diag_code: resolve.identifier.undefined
```neplg2
#entry main
#indent 4
#target std

#import "alloc/string/storage" as *
#import "core/result" as *
#import "std/env/cliarg/raw" as cli_raw

fn main %impure fn () i32 \():
    let p %MemPtr u8 string_data_ptr "argv";
    match cli_raw::cli_args_sizes_result p:
        Result::Ok _:
            0
        Result::Err _:
            1
```

## std/env/cliarg/raw は unchecked byte load helper を公開しない

neplg2:test[compile_fail]
diag_code: resolve.identifier.undefined
```neplg2
#entry main
#indent 4
#target std

#import "alloc/string/storage" as *
#import "core/result" as *
#import "std/env/cliarg/raw" as cli_raw

fn main %impure fn () i32 \():
    let p %MemPtr u8 string_data_ptr "argv";
    match cli_raw::cli_load_u8_result p 0:
        Result::Ok b:
            b
        Result::Err _:
            1
```
