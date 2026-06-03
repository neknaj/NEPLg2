# raw body target precheck

## wasm_target_rejects_llvmir_body

neplg2:test[compile_fail]
diag_code: effect.raw_body.target_mismatch
```neplg2
#target core
#entry main
#indent 4

fn main %fn void i32 \void:
    #llvmir:
        define i32 @main() {
        entry:
            ret i32 1
        }
```

## llvm_target_rejects_wasm_body

neplg2:test[llvm_cli, compile_fail]
diag_code: effect.raw_body.target_mismatch
```neplg2
#target llvm
#entry main
#indent 4

fn main %fn void i32 \void:
    #wasm:
        i32.const 1
```

## active_raw_bodies_conflict_reports_diag

neplg2:test[compile_fail]
diag_code: effect.raw_body.multiple_active
```neplg2
#target core
#entry main
#indent 4

fn main %fn void i32 \void:
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
diag_code: backend.wasm.raw_line_parse_error
```neplg2
#target core
#entry main
#indent 4

fn main %fn void i32 \void:
    #wasm:
        i32.unknown
```

## pure_wasm_raw_memory_store_is_rejected

neplg2:test[compile_fail]
diag_code: effect.pure.calls_impure
```neplg2
#target core
#entry main
#indent 4
#import "core/field" as *

fn raw_store %fn i32 fn i32 unit \p\v:
    #wasm:
        local.get p
        local.get v
        i32.store

fn main %fn void i32 \void:
    raw_store 0 1
    0
```

## pure_wasm_raw_helper_call_is_rejected

neplg2:test[compile_fail]
diag_code: effect.pure.calls_impure
```neplg2
#target core
#entry main
#indent 4
#import "core/mem" as *
#import "core/mem/raw" as *
#import "core/field" as *

fn raw_store_helper %fn i32 fn i32 unit \p\v:
    #wasm:
        local.get p
        local.get v
        call $store_i32

fn main %fn void i32 \void:
    raw_store_helper 0 1
    0
```

## pure_llvm_raw_memory_store_is_rejected

neplg2:test[llvm_cli, compile_fail]
diag_code: effect.pure.calls_impure
```neplg2
#target llvm
#entry main
#indent 4

fn raw_store %fn i32 unit \v:
    #llvmir:
        define void @raw_store(i32 %v) {
        entry:
            %p = alloca i32
            store i32 %v, ptr %p, align 4
            ret void
        }

fn main %fn void i32 \void:
    raw_store 1
    0
```

## pure_llvm_raw_helper_call_is_rejected

neplg2:test[llvm_cli, compile_fail]
diag_code: effect.pure.calls_impure
```neplg2
#target llvm
#entry main
#indent 4
#import "core/mem" as *
#import "core/mem/raw" as *

fn raw_grow_helper %fn i32 i32 \pages:
    #llvmir:
        define i32 @raw_grow_helper(i32 %pages) {
        entry:
            %x = call i32 @mem_grow(i32 %pages)
            ret i32 %x
        }

fn main %fn void i32 \void:
    raw_grow_helper 1
```

## wasm_precheck_rejects_unsupported_extern_signature

neplg2:test[compile_fail]
diag_code: backend.wasm.extern_signature_unsupported
```neplg2
#target core
#entry main
#indent 4
#no_prelude

#extern "env" "f" fn f %fn void never

fn main %fn void i32 \void:
    1
```

## wasm_precheck_rejects_unsupported_function_result

neplg2:test[compile_fail]
diag_code: backend.wasm.function_signature_unsupported
```neplg2
#target core
#entry main
#indent 4

fn main %fn void never \void:
    #intrinsic "unreachable" <> ()
```
