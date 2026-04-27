# raw body target precheck

## wasm_target_rejects_llvmir_body

neplg2:test[compile_fail]
diag_id: 3095
```neplg2
#target core
#entry main
#indent 4

fn main <()->i32> ():
    #llvmir:
        define i32 @main() {
        entry:
            ret i32 1
        }
```

## llvm_target_rejects_wasm_body

neplg2:test[llvm_cli, compile_fail]
diag_id: 3095
```neplg2
#target llvm
#entry main
#indent 4

fn main <()->i32> ():
    #wasm:
        i32.const 1
```

## active_raw_bodies_conflict_reports_diag

neplg2:test[compile_fail]
diag_id: 3094
```neplg2
#target core
#entry main
#indent 4

fn main <()->i32> ():
    #if[target=core]
    #wasm:
        i32.const 1
    #if[target=core]
    #llvmir:
        define i32 @main() {
        entry:
            ret i32 2
        }
```

## wasm_precheck_rejects_invalid_raw_line

neplg2:test[compile_fail]
diag_id: 4004
```neplg2
#target core
#entry main
#indent 4

fn main <()->i32> ():
    #wasm:
        i32.unknown
```

## pure_wasm_raw_memory_store_is_rejected

neplg2:test[compile_fail]
diag_id: 3025
```neplg2
#target core
#entry main
#indent 4

fn raw_store <(i32,i32)->()> (p, v):
    #wasm:
        local.get p
        local.get v
        i32.store

fn main <()->i32> ():
    raw_store 0 1
    0
```

## pure_llvm_raw_memory_store_is_rejected

neplg2:test[llvm_cli, compile_fail]
diag_id: 3025
```neplg2
#target llvm
#entry main
#indent 4

fn raw_store <(i32)->()> (v):
    #llvmir:
        define void @raw_store(i32 %v) {
        entry:
            %p = alloca i32
            store i32 %v, ptr %p, align 4
            ret void
        }

fn main <()->i32> ():
    raw_store 1
    0
```

## wasm_precheck_rejects_unsupported_extern_signature

neplg2:test[compile_fail]
diag_id: 4001
```neplg2
#target core
#entry main
#indent 4
#no_prelude

#extern "env" "f" fn f <()->never>

fn main <()->i32> ():
    1
```

## wasm_precheck_rejects_unsupported_function_result

neplg2:test[compile_fail]
diag_id: 4002
```neplg2
#target core
#entry main
#indent 4

fn main <()->never> ():
    #intrinsic "unreachable" <> ()
```
