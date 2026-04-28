# NEPLg2 self-host type arena

## stores_primitive_types_with_stable_ids

neplg2:test
ret: 0
```neplg2
#entry main
#target std
#indent 4

#import "alloc/collections/vec" as *
#import "core/option" as *
#import "core/result" as *
#import "neplg2/core/ty/ty" as *
#import "std/test" as *

fn check_kind <(Vec<Result<(),str>>,Option<SelfhostTypeKind>,SelfhostTypeKind)*>Vec<Result<(),str>>> (checks, actual, expected):
    match actual:
        Option::Some kind:
            checks_push checks check selfhost_type_kind_eq kind expected
        Option::None:
            checks_push checks Result<(),str>::Err "type kind was absent"

fn main <()*>i32> ():
    let checks0 <Vec<Result<(),str>>> checks_new
    match selfhost_type_arena_new:
        Result::Ok arena0:
            match selfhost_type_arena_add_primitive arena0 SelfhostTypeKind::Unit:
                Result::Ok alloc1:
                    let unit_id <SelfhostTypeId> alloc1.type_id
                    match selfhost_type_arena_add_primitive alloc1.arena SelfhostTypeKind::Bool:
                        Result::Ok alloc2:
                            let bool_id <SelfhostTypeId> alloc2.type_id
                            let arena2 <SelfhostTypeArena> alloc2.arena
                            let checks1 <Vec<Result<(),str>>> checks_push checks0 check_eq_i32 0 selfhost_type_id_index unit_id
                            let checks2 <Vec<Result<(),str>>> checks_push checks1 check_eq_i32 1 selfhost_type_id_index bool_id
                            let checks3 <Vec<Result<(),str>>> checks_push checks2 check_eq_i32 2 selfhost_type_arena_len &arena2
                            let checks4 <Vec<Result<(),str>>> check_kind checks3 (selfhost_type_arena_get_kind &arena2 unit_id) SelfhostTypeKind::Unit
                            let checks5 <Vec<Result<(),str>>> check_kind checks4 (selfhost_type_arena_get_kind &arena2 bool_id) SelfhostTypeKind::Bool
                            selfhost_type_arena_free arena2
                            let shown <Vec<Result<(),str>>> checks_print_report checks5
                            checks_exit_code shown
                        Result::Err _e:
                            let checks1 <Vec<Result<(),str>>> checks_push checks0 Result<(),str>::Err "bool type allocation failed"
                            let shown <Vec<Result<(),str>>> checks_print_report checks1
                            checks_exit_code shown
                Result::Err _e:
                    let checks1 <Vec<Result<(),str>>> checks_push checks0 Result<(),str>::Err "unit type allocation failed"
                    let shown <Vec<Result<(),str>>> checks_print_report checks1
                    checks_exit_code shown
        Result::Err _e:
            let checks1 <Vec<Result<(),str>>> checks_push checks0 Result<(),str>::Err "type arena allocation failed"
            let shown <Vec<Result<(),str>>> checks_print_report checks1
            checks_exit_code shown
```

## stores_function_type_arguments_and_result

neplg2:test
ret: 0
```neplg2
#entry main
#target std
#indent 4

#import "alloc/collections/vec" as *
#import "core/option" as *
#import "core/result" as *
#import "neplg2/core/ty/ty" as *
#import "std/test" as *

fn check_type_id <(Vec<Result<(),str>>,Option<SelfhostTypeId>,SelfhostTypeId)*>Vec<Result<(),str>>> (checks, actual, expected):
    match actual:
        Option::Some type_id:
            checks_push checks check selfhost_type_id_eq type_id expected
        Option::None:
            checks_push checks Result<(),str>::Err "type id was absent"

fn check_i32_option <(Vec<Result<(),str>>,Option<i32>,i32)*>Vec<Result<(),str>>> (checks, actual, expected):
    match actual:
        Option::Some value:
            checks_push checks check_eq_i32 expected value
        Option::None:
            checks_push checks Result<(),str>::Err "i32 option was absent"

fn main <()*>i32> ():
    let checks0 <Vec<Result<(),str>>> checks_new
    match selfhost_type_arena_new:
        Result::Ok arena0:
            match selfhost_type_arena_add_primitive arena0 SelfhostTypeKind::I32:
                Result::Ok alloc1:
                    let i32_id <SelfhostTypeId> alloc1.type_id
                    match selfhost_type_arena_add_primitive alloc1.arena SelfhostTypeKind::Bool:
                        Result::Ok alloc2:
                            let bool_id <SelfhostTypeId> alloc2.type_id
                            match new<SelfhostTypeId>:
                                Result::Ok params0:
                                    match push<SelfhostTypeId> params0 i32_id:
                                        Result::Ok params1:
                                            match push<SelfhostTypeId> params1 bool_id:
                                                Result::Ok params2:
                                                    match selfhost_type_arena_add_function alloc2.arena params2 bool_id:
                                                        Result::Ok alloc3:
                                                            let fn_id <SelfhostTypeId> alloc3.type_id
                                                            let arena3 <SelfhostTypeArena> alloc3.arena
                                                            let checks1 <Vec<Result<(),str>>> checks_push checks0 check_eq_i32 3 selfhost_type_arena_len &arena3
                                                            let checks2 <Vec<Result<(),str>>> checks_push checks1 check_eq_i32 2 selfhost_type_arena_function_arg_len &arena3
                                                            let checks3 <Vec<Result<(),str>>> check_i32_option checks2 (selfhost_type_arena_function_arg_count &arena3 fn_id) 2
                                                            let checks4 <Vec<Result<(),str>>> check_type_id checks3 (selfhost_type_arena_function_arg &arena3 fn_id 0) i32_id
                                                            let checks5 <Vec<Result<(),str>>> check_type_id checks4 (selfhost_type_arena_function_arg &arena3 fn_id 1) bool_id
                                                            let checks6 <Vec<Result<(),str>>> check_type_id checks5 (selfhost_type_arena_function_result &arena3 fn_id) bool_id
                                                            selfhost_type_arena_free arena3
                                                            let shown <Vec<Result<(),str>>> checks_print_report checks6
                                                            checks_exit_code shown
                                                        Result::Err _e:
                                                            let checks1 <Vec<Result<(),str>>> checks_push checks0 Result<(),str>::Err "function type allocation failed"
                                                            let shown <Vec<Result<(),str>>> checks_print_report checks1
                                                            checks_exit_code shown
                                                Result::Err _e:
                                                    let checks1 <Vec<Result<(),str>>> checks_push checks0 Result<(),str>::Err "second param push failed"
                                                    let shown <Vec<Result<(),str>>> checks_print_report checks1
                                                    checks_exit_code shown
                                        Result::Err _e:
                                            let checks1 <Vec<Result<(),str>>> checks_push checks0 Result<(),str>::Err "first param push failed"
                                            let shown <Vec<Result<(),str>>> checks_print_report checks1
                                            checks_exit_code shown
                                Result::Err _e:
                                    let checks1 <Vec<Result<(),str>>> checks_push checks0 Result<(),str>::Err "param vector allocation failed"
                                    let shown <Vec<Result<(),str>>> checks_print_report checks1
                                    checks_exit_code shown
                        Result::Err _e:
                            let checks1 <Vec<Result<(),str>>> checks_push checks0 Result<(),str>::Err "bool type allocation failed"
                            let shown <Vec<Result<(),str>>> checks_print_report checks1
                            checks_exit_code shown
                Result::Err _e:
                    let checks1 <Vec<Result<(),str>>> checks_push checks0 Result<(),str>::Err "i32 type allocation failed"
                    let shown <Vec<Result<(),str>>> checks_print_report checks1
                    checks_exit_code shown
        Result::Err _e:
            let checks1 <Vec<Result<(),str>>> checks_push checks0 Result<(),str>::Err "type arena allocation failed"
            let shown <Vec<Result<(),str>>> checks_print_report checks1
            checks_exit_code shown
```

## returns_none_for_invalid_and_non_function_access

neplg2:test
ret: 0
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
    let checks0 <Vec<Result<(),str>>> checks_new
    match selfhost_type_arena_new:
        Result::Ok arena0:
            match selfhost_type_arena_add_primitive arena0 SelfhostTypeKind::Bool:
                Result::Ok alloc1:
                    let bool_id <SelfhostTypeId> alloc1.type_id
                    let arena1 <SelfhostTypeArena> alloc1.arena
                    let invalid_id <SelfhostTypeId> selfhost_type_id_invalid
                    let checks1 <Vec<Result<(),str>>> checks_push checks0 check is_none<SelfhostTypeRecord> selfhost_type_arena_get_record &arena1 invalid_id
                    let checks2 <Vec<Result<(),str>>> checks_push checks1 check is_none<i32> selfhost_type_arena_function_arg_count &arena1 bool_id
                    let checks3 <Vec<Result<(),str>>> checks_push checks2 check is_none<SelfhostTypeId> selfhost_type_arena_function_arg &arena1 bool_id 0
                    let checks4 <Vec<Result<(),str>>> checks_push checks3 check is_none<SelfhostTypeId> selfhost_type_arena_function_result &arena1 invalid_id
                    selfhost_type_arena_free arena1
                    let shown <Vec<Result<(),str>>> checks_print_report checks4
                    checks_exit_code shown
                Result::Err _e:
                    let checks1 <Vec<Result<(),str>>> checks_push checks0 Result<(),str>::Err "bool type allocation failed"
                    let shown <Vec<Result<(),str>>> checks_print_report checks1
                    checks_exit_code shown
        Result::Err _e:
            let checks1 <Vec<Result<(),str>>> checks_push checks0 Result<(),str>::Err "type arena allocation failed"
            let shown <Vec<Result<(),str>>> checks_print_report checks1
            checks_exit_code shown
```
