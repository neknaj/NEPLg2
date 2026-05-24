# block_diagnostics

## nested_generic_function_reports_diag_code

neplg2:test[compile_fail]
diag_code: type.nested_function.generic_unsupported
```neplg2
#entry main
#indent 4
#target core

fn main %fn () i32 \():
    fn id <.T> %fn .T .T \x:
        x
    id<i32> 1
```

## nested_raw_block_reports_diag_code

neplg2:test[compile_fail]
diag_code: type.raw_block.invalid_placement
```neplg2
#entry main
#indent 4
#target wasm

fn main %fn () i32 \():
    block:
        #wasm:
            i32.const 1
    0
```
