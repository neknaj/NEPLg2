# stdio raw boundary regression

`std/stdio/raw` は target ABI に近い実装境界であり、通常 source が
`MemPtr` と長さを自由に組み合わせる fd I/O helper を direct import できてはならない。

## stdio/raw は fd_read MemPtr wrapper を公開しない

neplg2:test[compile_fail]
diag_code: resolve.identifier.undefined
```neplg2
#entry main
#target std
#import "std/stdio/raw" as raw

fn main %impure fn void i32 \void:
    raw::stdio_fd_read_mem 0 0 0 0
```

## stdio/raw は fd_write MemPtr wrapper を公開しない

neplg2:test[compile_fail]
diag_code: resolve.identifier.undefined
```neplg2
#entry main
#target std
#import "std/stdio/raw" as raw

fn main %impure fn void i32 \void:
    raw::stdio_fd_write_mem 1 0 0 0
```

## stdio/raw は fd_write layout helper を公開しない

neplg2:test[compile_fail]
diag_code: resolve.identifier.undefined
```neplg2
#entry main
#target std
#import "std/stdio/raw" as raw

fn main %impure fn void i32 \void:
    raw::stdio_fd_write_from_result 1 0 0 0 1
```

## write/fd の raw layout helper は private のままにする

neplg2:test[compile_fail]
diag_code: resolve.identifier.undefined
```neplg2
#entry main
#target std
#import "std/stdio/write/fd" as fd

fn main %impure fn void i32 \void:
    fd::stdio_fd_write_from_result 1 0 0 0 1
```

## read/buffer の raw layout helper は private のままにする

neplg2:test[compile_fail]
diag_code: resolve.identifier.undefined
```neplg2
#entry main
#target std
#import "std/stdio/read/buffer" as read_buffer

fn main %impure fn void i32 \void:
    read_buffer::stdio_fd_read_into_result 0 0 0 0 1
```
