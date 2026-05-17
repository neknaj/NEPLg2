# NEPLg2 self-host type arena

## stores_primitive_types_with_stable_ids

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: mlstr:
    ##: Checked [ok,ok,ok,ok,ok,ok,ok,ok,ok,ok,ok]
    ##: [0] ok
    ##: [1] ok
    ##: [2] ok
    ##: [3] ok
    ##: [4] ok
    ##: [5] ok
    ##: [6] ok
    ##: [7] ok
    ##: [8] ok
    ##: [9] ok
    ##: [10] ok
```neplg2
#entry main
#target std
#indent 4

#import "alloc/collections/vec" as *
#import "core/option" as *
#import "core/result" as *
#import "neplg2/core/ty/ty" as *
#import "std/test" as *

fn check_kind <(TestReport,Option<SelfhostTypeKind>,SelfhostTypeKind)*>TestReport> (checks, actual, expected):
    match actual:
        Option::Some kind:
            checks_push checks check selfhost_type_kind_eq kind expected
        Option::None:
            checks_push checks Result<(),str>::Err "type kind was absent"

fn main <()*>i32> ():
    let checks0 checks_new
    match selfhost_type_arena_new:
        Result::Ok arena0:
            match selfhost_type_arena_add_primitive arena0 SelfhostPrimitiveTypeKind::Unit:
                Result::Ok alloc1:
                    let unit_id <SelfhostTypeId> selfhost_type_arena_alloc_type_id &alloc1
                    match selfhost_type_arena_add_primitive selfhost_type_arena_alloc_into_arena alloc1 SelfhostPrimitiveTypeKind::Bool:
                        Result::Ok alloc2:
                            let bool_id <SelfhostTypeId> selfhost_type_arena_alloc_type_id &alloc2
                            match selfhost_type_arena_add_primitive selfhost_type_arena_alloc_into_arena alloc2 SelfhostPrimitiveTypeKind::F32:
                                Result::Ok alloc3:
                                    let f32_id <SelfhostTypeId> selfhost_type_arena_alloc_type_id &alloc3
                                    match selfhost_type_arena_add_primitive selfhost_type_arena_alloc_into_arena alloc3 SelfhostPrimitiveTypeKind::F64:
                                        Result::Ok alloc4:
                                            let f64_id <SelfhostTypeId> selfhost_type_arena_alloc_type_id &alloc4
                                            match selfhost_type_arena_add_primitive selfhost_type_arena_alloc_into_arena alloc4 SelfhostPrimitiveTypeKind::Never:
                                                Result::Ok alloc5:
                                                    let never_id <SelfhostTypeId> selfhost_type_arena_alloc_type_id &alloc5
                                                    let arena5 <SelfhostTypeArena> selfhost_type_arena_alloc_into_arena alloc5
                                                    let checks1 checks_push checks0 check_eq_i32 0 selfhost_type_id_index unit_id
                                                    let checks2 checks_push checks1 check_eq_i32 1 selfhost_type_id_index bool_id
                                                    let checks3 checks_push checks2 check_eq_i32 2 selfhost_type_id_index f32_id
                                                    let checks4 checks_push checks3 check_eq_i32 3 selfhost_type_id_index f64_id
                                                    let checks5 checks_push checks4 check_eq_i32 4 selfhost_type_id_index never_id
                                                    let checks6 checks_push checks5 check_eq_i32 5 selfhost_type_arena_len &arena5
                                                    let checks7 check_kind checks6 (selfhost_type_arena_get_kind &arena5 unit_id) SelfhostTypeKind::Unit
                                                    let checks8 check_kind checks7 (selfhost_type_arena_get_kind &arena5 bool_id) SelfhostTypeKind::Bool
                                                    let checks9 check_kind checks8 (selfhost_type_arena_get_kind &arena5 f32_id) SelfhostTypeKind::F32
                                                    let checks10 check_kind checks9 (selfhost_type_arena_get_kind &arena5 f64_id) SelfhostTypeKind::F64
                                                    let checks11 check_kind checks10 (selfhost_type_arena_get_kind &arena5 never_id) SelfhostTypeKind::Never
                                                    selfhost_type_arena_free arena5
                                                    let shown checks_print_report checks11
                                                    checks_exit_code shown
                                                Result::Err _e:
                                                    let checks1 checks_push checks0 Result<(),str>::Err "never type allocation failed"
                                                    let shown checks_print_report checks1
                                                    checks_exit_code shown
                                        Result::Err _e:
                                            let checks1 checks_push checks0 Result<(),str>::Err "f64 type allocation failed"
                                            let shown checks_print_report checks1
                                            checks_exit_code shown
                                Result::Err _e:
                                    let checks1 checks_push checks0 Result<(),str>::Err "f32 type allocation failed"
                                    let shown checks_print_report checks1
                                    checks_exit_code shown
                        Result::Err _e:
                            let checks1 checks_push checks0 Result<(),str>::Err "bool type allocation failed"
                            let shown checks_print_report checks1
                            checks_exit_code shown
                Result::Err _e:
                    let checks1 checks_push checks0 Result<(),str>::Err "unit type allocation failed"
                    let shown checks_print_report checks1
                    checks_exit_code shown
        Result::Err _e:
            let checks1 checks_push checks0 Result<(),str>::Err "type arena allocation failed"
            let shown checks_print_report checks1
            checks_exit_code shown
```

## stores_function_type_arguments_and_result

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: mlstr:
    ##: Checked [ok,ok,ok,ok,ok,ok]
    ##: [0] ok
    ##: [1] ok
    ##: [2] ok
    ##: [3] ok
    ##: [4] ok
    ##: [5] ok
```neplg2
#entry main
#target std
#indent 4

#import "alloc/collections/vec" as *
#import "core/option" as *
#import "core/result" as *
#import "neplg2/core/ty/ty" as *
#import "std/test" as *

fn check_type_id <(TestReport,Option<SelfhostTypeId>,SelfhostTypeId)*>TestReport> (checks, actual, expected):
    match actual:
        Option::Some type_id:
            checks_push checks check selfhost_type_id_eq type_id expected
        Option::None:
            checks_push checks Result<(),str>::Err "type id was absent"

fn check_i32_option <(TestReport,Option<i32>,i32)*>TestReport> (checks, actual, expected):
    match actual:
        Option::Some value:
            checks_push checks check_eq_i32 expected value
        Option::None:
            checks_push checks Result<(),str>::Err "i32 option was absent"

fn main <()*>i32> ():
    let checks0 checks_new
    match selfhost_type_arena_new:
        Result::Ok arena0:
            match selfhost_type_arena_add_primitive arena0 SelfhostPrimitiveTypeKind::I32:
                Result::Ok alloc1:
                    let i32_id <SelfhostTypeId> selfhost_type_arena_alloc_type_id &alloc1
                    match selfhost_type_arena_add_primitive selfhost_type_arena_alloc_into_arena alloc1 SelfhostPrimitiveTypeKind::Bool:
                        Result::Ok alloc2:
                            let bool_id <SelfhostTypeId> selfhost_type_arena_alloc_type_id &alloc2
                            match new<SelfhostTypeId>:
                                Result::Ok params0:
                                    match push<SelfhostTypeId> params0 i32_id:
                                        Result::Ok params1:
                                            match push<SelfhostTypeId> params1 bool_id:
                                                Result::Ok params2:
                                                    match selfhost_type_arena_add_function selfhost_type_arena_alloc_into_arena alloc2 params2 bool_id:
                                                        Result::Ok alloc3:
                                                            let fn_id <SelfhostTypeId> selfhost_type_arena_alloc_type_id &alloc3
                                                            let arena3 <SelfhostTypeArena> selfhost_type_arena_alloc_into_arena alloc3
                                                            let checks1 checks_push checks0 check_eq_i32 3 selfhost_type_arena_len &arena3
                                                            let checks2 checks_push checks1 check_eq_i32 2 selfhost_type_arena_function_arg_len &arena3
                                                            let checks3 check_i32_option checks2 (selfhost_type_arena_function_arg_count &arena3 fn_id) 2
                                                            let checks4 check_type_id checks3 (selfhost_type_arena_function_arg &arena3 fn_id 0) i32_id
                                                            let checks5 check_type_id checks4 (selfhost_type_arena_function_arg &arena3 fn_id 1) bool_id
                                                            let checks6 check_type_id checks5 (selfhost_type_arena_function_result &arena3 fn_id) bool_id
                                                            selfhost_type_arena_free arena3
                                                            let shown checks_print_report checks6
                                                            checks_exit_code shown
                                                        Result::Err _e:
                                                            let checks1 checks_push checks0 Result<(),str>::Err "function type allocation failed"
                                                            let shown checks_print_report checks1
                                                            checks_exit_code shown
                                                Result::Err _e:
                                                    let returned <Vec<SelfhostTypeId>> vec_push_error_vec<SelfhostTypeId> _e
                                                    free<SelfhostTypeId> returned
                                                    selfhost_type_arena_free selfhost_type_arena_alloc_into_arena alloc2
                                                    let checks1 checks_push checks0 Result<(),str>::Err "second param push failed"
                                                    let shown checks_print_report checks1
                                                    checks_exit_code shown
                                        Result::Err _e:
                                            let returned <Vec<SelfhostTypeId>> vec_push_error_vec<SelfhostTypeId> _e
                                            free<SelfhostTypeId> returned
                                            selfhost_type_arena_free selfhost_type_arena_alloc_into_arena alloc2
                                            let checks1 checks_push checks0 Result<(),str>::Err "first param push failed"
                                            let shown checks_print_report checks1
                                            checks_exit_code shown
                                Result::Err _e:
                                    selfhost_type_arena_free selfhost_type_arena_alloc_into_arena alloc2
                                    let checks1 checks_push checks0 Result<(),str>::Err "param vector allocation failed"
                                    let shown checks_print_report checks1
                                    checks_exit_code shown
                        Result::Err _e:
                            let checks1 checks_push checks0 Result<(),str>::Err "bool type allocation failed"
                            let shown checks_print_report checks1
                            checks_exit_code shown
                Result::Err _e:
                    let checks1 checks_push checks0 Result<(),str>::Err "i32 type allocation failed"
                    let shown checks_print_report checks1
                    checks_exit_code shown
        Result::Err _e:
            let checks1 checks_push checks0 Result<(),str>::Err "type arena allocation failed"
            let shown checks_print_report checks1
            checks_exit_code shown
```

## returns_none_for_invalid_and_non_function_access

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: mlstr:
    ##: Checked [ok,ok,ok,ok]
    ##: [0] ok
    ##: [1] ok
    ##: [2] ok
    ##: [3] ok
```neplg2
#entry main
#target std
#indent 4

#import "alloc/collections/vec" as *
#import "core/option" as *
#import "core/result" as *
#import "neplg2/core/ty/ty" as *
#import "std/test" as *

fn main <()*>i32> ():
    let checks0 checks_new
    match selfhost_type_arena_new:
        Result::Ok arena0:
            match selfhost_type_arena_add_primitive arena0 SelfhostPrimitiveTypeKind::Bool:
                Result::Ok alloc1:
                    let bool_id <SelfhostTypeId> selfhost_type_arena_alloc_type_id &alloc1
                    let arena1 <SelfhostTypeArena> selfhost_type_arena_alloc_into_arena alloc1
                    let invalid_id <SelfhostTypeId> selfhost_type_id_new -1
                    let checks1 checks_push checks0 check is_none<SelfhostTypeRecord> selfhost_type_arena_get_record &arena1 invalid_id
                    let checks2 checks_push checks1 check is_none<i32> selfhost_type_arena_function_arg_count &arena1 bool_id
                    let checks3 checks_push checks2 check is_none<SelfhostTypeId> selfhost_type_arena_function_arg &arena1 bool_id 0
                    let checks4 checks_push checks3 check is_none<SelfhostTypeId> selfhost_type_arena_function_result &arena1 invalid_id
                    selfhost_type_arena_free arena1
                    let shown checks_print_report checks4
                    checks_exit_code shown
                Result::Err _e:
                    let checks1 checks_push checks0 Result<(),str>::Err "bool type allocation failed"
                    let shown checks_print_report checks1
                    checks_exit_code shown
        Result::Err _e:
            let checks1 checks_push checks0 Result<(),str>::Err "type arena allocation failed"
            let shown checks_print_report checks1
            checks_exit_code shown
```

## compares_function_type_shapes_structurally

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: mlstr:
    ##: Checked [ok,ok]
    ##: [0] ok
    ##: [1] ok
```neplg2
#entry main
#target std
#indent 4

#import "alloc/collections/vec" as *
#import "core/result" as *
#import "neplg2/core/ty/ty" as *
#import "std/test" as *
#import "core/math" as *

fn add_one_arg_function <(SelfhostTypeArena,SelfhostTypeId,SelfhostTypeId)*>Result<SelfhostTypeArenaAlloc, StdErrorKind>> (arena, arg_id, result_id):
    match new<SelfhostTypeId>:
        Result::Ok params0:
            match push<SelfhostTypeId> params0 arg_id:
                Result::Ok params1:
                    selfhost_type_arena_add_function arena params1 result_id
                Result::Err e:
                    let error <StdErrorKind> vec_push_error_kind<SelfhostTypeId> &e
                    let returned <Vec<SelfhostTypeId>> vec_push_error_vec<SelfhostTypeId> e
                    free<SelfhostTypeId> returned
                    selfhost_type_arena_free arena
                    Result<SelfhostTypeArenaAlloc, StdErrorKind>::Err error
        Result::Err e:
            selfhost_type_arena_free arena
            Result<SelfhostTypeArenaAlloc, StdErrorKind>::Err e

fn main <()*>i32> ():
    let checks0 checks_new
    match selfhost_type_arena_new:
        Result::Ok arena0:
            match selfhost_type_arena_add_primitive arena0 SelfhostPrimitiveTypeKind::I32:
                Result::Ok alloc1:
                    let i32_id <SelfhostTypeId> selfhost_type_arena_alloc_type_id &alloc1
                    match selfhost_type_arena_add_primitive selfhost_type_arena_alloc_into_arena alloc1 SelfhostPrimitiveTypeKind::Bool:
                        Result::Ok alloc2:
                            let bool_id <SelfhostTypeId> selfhost_type_arena_alloc_type_id &alloc2
                            match add_one_arg_function selfhost_type_arena_alloc_into_arena alloc2 i32_id bool_id:
                                Result::Ok alloc3:
                                    let fn1_id <SelfhostTypeId> selfhost_type_arena_alloc_type_id &alloc3
                                    match add_one_arg_function selfhost_type_arena_alloc_into_arena alloc3 i32_id bool_id:
                                        Result::Ok alloc4:
                                            let fn2_id <SelfhostTypeId> selfhost_type_arena_alloc_type_id &alloc4
                                            let arena4 <SelfhostTypeArena> selfhost_type_arena_alloc_into_arena alloc4
                                            let checks1 checks_push checks0 check not selfhost_type_id_eq fn1_id fn2_id
                                            let checks2 checks_push checks1 check selfhost_type_arena_types_equal &arena4 fn1_id fn2_id
                                            selfhost_type_arena_free arena4
                                            let shown checks_print_report checks2
                                            checks_exit_code shown
                                        Result::Err _e:
                                            let checks1 checks_push checks0 Result<(),str>::Err "second function allocation failed"
                                            let shown checks_print_report checks1
                                            checks_exit_code shown
                                Result::Err _e:
                                    let checks1 checks_push checks0 Result<(),str>::Err "first function allocation failed"
                                    let shown checks_print_report checks1
                                    checks_exit_code shown
                        Result::Err _e:
                            let checks1 checks_push checks0 Result<(),str>::Err "bool type allocation failed"
                            let shown checks_print_report checks1
                            checks_exit_code shown
                Result::Err _e:
                    let checks1 checks_push checks0 Result<(),str>::Err "i32 type allocation failed"
                    let shown checks_print_report checks1
                    checks_exit_code shown
        Result::Err _e:
            let checks1 checks_push checks0 Result<(),str>::Err "type arena allocation failed"
            let shown checks_print_report checks1
            checks_exit_code shown
```

## rejects_mismatched_function_type_shapes

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: mlstr:
    ##: Checked [ok,ok,ok,ok]
    ##: [0] ok
    ##: [1] ok
    ##: [2] ok
    ##: [3] ok
```neplg2
#entry main
#target std
#indent 4

#import "alloc/collections/vec" as *
#import "core/result" as *
#import "neplg2/core/ty/ty" as *
#import "std/test" as *
#import "core/math" as *

fn add_one_arg_function <(SelfhostTypeArena,SelfhostTypeId,SelfhostTypeId)*>Result<SelfhostTypeArenaAlloc, StdErrorKind>> (arena, arg_id, result_id):
    match new<SelfhostTypeId>:
        Result::Ok params0:
            match push<SelfhostTypeId> params0 arg_id:
                Result::Ok params1:
                    selfhost_type_arena_add_function arena params1 result_id
                Result::Err e:
                    let error <StdErrorKind> vec_push_error_kind<SelfhostTypeId> &e
                    let returned <Vec<SelfhostTypeId>> vec_push_error_vec<SelfhostTypeId> e
                    free<SelfhostTypeId> returned
                    selfhost_type_arena_free arena
                    Result<SelfhostTypeArenaAlloc, StdErrorKind>::Err error
        Result::Err e:
            selfhost_type_arena_free arena
            Result<SelfhostTypeArenaAlloc, StdErrorKind>::Err e

fn add_zero_arg_function <(SelfhostTypeArena,SelfhostTypeId)*>Result<SelfhostTypeArenaAlloc, StdErrorKind>> (arena, result_id):
    match new<SelfhostTypeId>:
        Result::Ok params:
            selfhost_type_arena_add_function arena params result_id
        Result::Err e:
            selfhost_type_arena_free arena
            Result<SelfhostTypeArenaAlloc, StdErrorKind>::Err e

fn main <()*>i32> ():
    let checks0 checks_new
    match selfhost_type_arena_new:
        Result::Ok arena0:
            match selfhost_type_arena_add_primitive arena0 SelfhostPrimitiveTypeKind::I32:
                Result::Ok alloc1:
                    let i32_id <SelfhostTypeId> selfhost_type_arena_alloc_type_id &alloc1
                    match selfhost_type_arena_add_primitive selfhost_type_arena_alloc_into_arena alloc1 SelfhostPrimitiveTypeKind::Bool:
                        Result::Ok alloc2:
                            let bool_id <SelfhostTypeId> selfhost_type_arena_alloc_type_id &alloc2
                            match add_one_arg_function selfhost_type_arena_alloc_into_arena alloc2 i32_id bool_id:
                                Result::Ok alloc3:
                                    let fn1_id <SelfhostTypeId> selfhost_type_arena_alloc_type_id &alloc3
                                    match add_one_arg_function selfhost_type_arena_alloc_into_arena alloc3 bool_id bool_id:
                                        Result::Ok alloc4:
                                            let fn_arg_mismatch <SelfhostTypeId> selfhost_type_arena_alloc_type_id &alloc4
                                            match add_one_arg_function selfhost_type_arena_alloc_into_arena alloc4 i32_id i32_id:
                                                Result::Ok alloc5:
                                                    let fn_result_mismatch <SelfhostTypeId> selfhost_type_arena_alloc_type_id &alloc5
                                                    match add_zero_arg_function selfhost_type_arena_alloc_into_arena alloc5 bool_id:
                                                        Result::Ok alloc6:
                                                            let fn_arity_mismatch <SelfhostTypeId> selfhost_type_arena_alloc_type_id &alloc6
                                                            let arena6 <SelfhostTypeArena> selfhost_type_arena_alloc_into_arena alloc6
                                                            let invalid_id <SelfhostTypeId> selfhost_type_id_new -1
                                                            let checks1 checks_push checks0 check not selfhost_type_arena_types_equal &arena6 fn1_id fn_arg_mismatch
                                                            let checks2 checks_push checks1 check not selfhost_type_arena_types_equal &arena6 fn1_id fn_result_mismatch
                                                            let checks3 checks_push checks2 check not selfhost_type_arena_types_equal &arena6 fn1_id fn_arity_mismatch
                                                            let checks4 checks_push checks3 check not selfhost_type_arena_types_equal &arena6 invalid_id invalid_id
                                                            selfhost_type_arena_free arena6
                                                            let shown checks_print_report checks4
                                                            checks_exit_code shown
                                                        Result::Err _e:
                                                            let checks1 checks_push checks0 Result<(),str>::Err "arity mismatch function allocation failed"
                                                            let shown checks_print_report checks1
                                                            checks_exit_code shown
                                                Result::Err _e:
                                                    let checks1 checks_push checks0 Result<(),str>::Err "result mismatch function allocation failed"
                                                    let shown checks_print_report checks1
                                                    checks_exit_code shown
                                        Result::Err _e:
                                            let checks1 checks_push checks0 Result<(),str>::Err "arg mismatch function allocation failed"
                                            let shown checks_print_report checks1
                                            checks_exit_code shown
                                Result::Err _e:
                                    let checks1 checks_push checks0 Result<(),str>::Err "base function allocation failed"
                                    let shown checks_print_report checks1
                                    checks_exit_code shown
                        Result::Err _e:
                            let checks1 checks_push checks0 Result<(),str>::Err "bool type allocation failed"
                            let shown checks_print_report checks1
                            checks_exit_code shown
                Result::Err _e:
                    let checks1 checks_push checks0 Result<(),str>::Err "i32 type allocation failed"
                    let shown checks_print_report checks1
                    checks_exit_code shown
        Result::Err _e:
            let checks1 checks_push checks0 Result<(),str>::Err "type arena allocation failed"
            let shown checks_print_report checks1
            checks_exit_code shown
```
