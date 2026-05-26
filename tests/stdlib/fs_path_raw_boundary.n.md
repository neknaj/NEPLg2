# std/fs/path raw boundary regression

## std/fs/path は fd_readdir の raw byte conversion を公開しない

neplg2:test[compile_fail]
diag_code: resolve.identifier.undefined
```neplg2
#entry main
#indent 4
#target std

#import "core/result" as *
#import "std/fs/path" as *

fn main %fn unit i32 \unit:
    match fs_string_from_bytes 0 0:
        Result::Ok _name:
            1
        Result::Err _e:
            0
```
