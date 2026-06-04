# NEPLg2 self-host canonical type key

## projects_equivalent_applied_types_to_equal_canonical_keys

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
#import "core/math" as *
#import "core/result" as *
#import "neplg2/core/ty/ty" as *
#import "std/test" as *

fn add_single_arg_applied %impure fn SelfhostTypeArena impure fn SelfhostNamedTypeId impure fn SelfhostTypeId Result SelfhostTypeArenaAlloc StdErrorKind \arena\nominal_id\arg:
    let params_result %Result Vec SelfhostTypeId StdErrorKind new
    match params_result:
        Result::Ok params0:
            match push params0 arg:
                Result::Ok params1:
                    selfhost_type_arena_add_applied_named arena nominal_id params1
                Result::Err e:
                    let error %StdErrorKind vec_push_error_kind &e
                    let returned %Vec SelfhostTypeId vec_push_error_vec e
                    free returned
                    selfhost_type_arena_free arena
                    Result::Err error
        Result::Err e:
            selfhost_type_arena_free arena
            Result::Err e

fn main %impure fn void i32 \void:
    let checks0 checks_new
    match selfhost_type_arena_new:
        Result::Ok arena0:
            match selfhost_type_arena_add_primitive arena0 SelfhostPrimitiveTypeKind::I32:
                Result::Ok alloc1:
                    let i32_id %SelfhostTypeId selfhost_type_arena_alloc_type_id &alloc1
                    match selfhost_type_arena_add_primitive selfhost_type_arena_alloc_into_arena alloc1 SelfhostPrimitiveTypeKind::Bool:
                        Result::Ok alloc2:
                            let bool_id %SelfhostTypeId selfhost_type_arena_alloc_type_id &alloc2
                            let box_id %SelfhostNamedTypeId selfhost_named_type_id_new 42
                            match add_single_arg_applied selfhost_type_arena_alloc_into_arena alloc2 box_id i32_id:
                                Result::Ok alloc3:
                                    let box_i32_a %SelfhostTypeId selfhost_type_arena_alloc_type_id &alloc3
                                    match add_single_arg_applied selfhost_type_arena_alloc_into_arena alloc3 box_id i32_id:
                                        Result::Ok alloc4:
                                            let box_i32_b %SelfhostTypeId selfhost_type_arena_alloc_type_id &alloc4
                                            match add_single_arg_applied selfhost_type_arena_alloc_into_arena alloc4 box_id bool_id:
                                                Result::Ok alloc5:
                                                    let box_bool %SelfhostTypeId selfhost_type_arena_alloc_type_id &alloc5
                                                    let type_arena %SelfhostTypeArena selfhost_type_arena_alloc_into_arena alloc5
                                                    match selfhost_canonical_type_key_arena_new:
                                                        Result::Ok key_arena0:
                                                            match selfhost_canonical_type_key_project_into_arena &type_arena key_arena0 box_i32_a:
                                                                Result::Ok key_alloc1:
                                                                    let key_a %SelfhostCanonicalTypeKeyId selfhost_canonical_type_key_arena_alloc_key_id &key_alloc1
                                                                    let key_arena1 %SelfhostCanonicalTypeKeyArena selfhost_canonical_type_key_arena_alloc_into_arena key_alloc1
                                                                    match selfhost_canonical_type_key_project_into_arena &type_arena key_arena1 box_i32_b:
                                                                        Result::Ok key_alloc2:
                                                                            let key_b %SelfhostCanonicalTypeKeyId selfhost_canonical_type_key_arena_alloc_key_id &key_alloc2
                                                                            let key_arena2 %SelfhostCanonicalTypeKeyArena selfhost_canonical_type_key_arena_alloc_into_arena key_alloc2
                                                                            match selfhost_canonical_type_key_project_into_arena &type_arena key_arena2 box_bool:
                                                                                Result::Ok key_alloc3:
                                                                                    let key_bool %SelfhostCanonicalTypeKeyId selfhost_canonical_type_key_arena_alloc_key_id &key_alloc3
                                                                                    let key_arena3 %SelfhostCanonicalTypeKeyArena selfhost_canonical_type_key_arena_alloc_into_arena key_alloc3
                                                                                    let checks1 checks_push checks0 check selfhost_canonical_type_key_equal &key_arena3 key_a key_b
                                                                                    let checks2 checks_push checks1 check not selfhost_canonical_type_key_equal &key_arena3 key_a key_bool
                                                                                    let checks3 checks_push checks2 check_eq_i32 6 selfhost_canonical_type_key_arena_node_len &key_arena3
                                                                                    let checks4 checks_push checks3 check_eq_i32 3 selfhost_canonical_type_key_arena_arg_len &key_arena3
                                                                                    selfhost_canonical_type_key_arena_free key_arena3
                                                                                    selfhost_type_arena_free type_arena
                                                                                    let shown checks_print_report checks4
                                                                                    checks_exit_code shown
                                                                                Result::Err _e:
                                                                                    selfhost_type_arena_free type_arena
                                                                                    let checks1 checks_push checks0 Result::Err "Box bool canonical key projection failed"
                                                                                    let shown checks_print_report checks1
                                                                                    checks_exit_code shown
                                                                        Result::Err _e:
                                                                            selfhost_type_arena_free type_arena
                                                                            let checks1 checks_push checks0 Result::Err "second Box i32 canonical key projection failed"
                                                                            let shown checks_print_report checks1
                                                                            checks_exit_code shown
                                                                Result::Err _e:
                                                                    selfhost_type_arena_free type_arena
                                                                    let checks1 checks_push checks0 Result::Err "first Box i32 canonical key projection failed"
                                                                    let shown checks_print_report checks1
                                                                    checks_exit_code shown
                                                        Result::Err _e:
                                                            selfhost_type_arena_free type_arena
                                                            let checks1 checks_push checks0 Result::Err "canonical key arena allocation failed"
                                                            let shown checks_print_report checks1
                                                            checks_exit_code shown
                                                Result::Err _e:
                                                    let checks1 checks_push checks0 Result::Err "Box bool allocation failed"
                                                    let shown checks_print_report checks1
                                                    checks_exit_code shown
                                        Result::Err _e:
                                            let checks1 checks_push checks0 Result::Err "second Box i32 allocation failed"
                                            let shown checks_print_report checks1
                                            checks_exit_code shown
                                Result::Err _e:
                                    let checks1 checks_push checks0 Result::Err "first Box i32 allocation failed"
                                    let shown checks_print_report checks1
                                    checks_exit_code shown
                        Result::Err _e:
                            let checks1 checks_push checks0 Result::Err "bool type allocation failed"
                            let shown checks_print_report checks1
                            checks_exit_code shown
                Result::Err _e:
                    let checks1 checks_push checks0 Result::Err "i32 type allocation failed"
                    let shown checks_print_report checks1
                    checks_exit_code shown
        Result::Err _e:
            let checks1 checks_push checks0 Result::Err "type arena allocation failed"
            let shown checks_print_report checks1
            checks_exit_code shown
```

## preserves_zero_argument_function_boundary

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
#import "core/math" as *
#import "core/result" as *
#import "neplg2/core/ty/ty" as *
#import "std/test" as *

fn add_one_arg_function %impure fn SelfhostTypeArena impure fn SelfhostTypeId impure fn SelfhostTypeId Result SelfhostTypeArenaAlloc StdErrorKind \arena\arg_id\result_id:
    let params_result %Result Vec SelfhostTypeId StdErrorKind new
    match params_result:
        Result::Ok params0:
            match push params0 arg_id:
                Result::Ok params1:
                    selfhost_type_arena_add_function arena params1 result_id
                Result::Err e:
                    let error %StdErrorKind vec_push_error_kind &e
                    let returned %Vec SelfhostTypeId vec_push_error_vec e
                    free returned
                    selfhost_type_arena_free arena
                    Result::Err error
        Result::Err e:
            selfhost_type_arena_free arena
            Result::Err e

fn add_zero_arg_function %impure fn SelfhostTypeArena impure fn SelfhostTypeId Result SelfhostTypeArenaAlloc StdErrorKind \arena\result_id:
    let params_result %Result Vec SelfhostTypeId StdErrorKind new
    match params_result:
        Result::Ok params:
            selfhost_type_arena_add_function arena params result_id
        Result::Err e:
            selfhost_type_arena_free arena
            Result::Err e

fn main %impure fn void i32 \void:
    let checks0 checks_new
    match selfhost_type_arena_new:
        Result::Ok arena0:
            match selfhost_type_arena_add_primitive arena0 SelfhostPrimitiveTypeKind::Unit:
                Result::Ok alloc1:
                    let unit_id %SelfhostTypeId selfhost_type_arena_alloc_type_id &alloc1
                    match selfhost_type_arena_add_primitive selfhost_type_arena_alloc_into_arena alloc1 SelfhostPrimitiveTypeKind::Bool:
                        Result::Ok alloc2:
                            let bool_id %SelfhostTypeId selfhost_type_arena_alloc_type_id &alloc2
                            match add_one_arg_function selfhost_type_arena_alloc_into_arena alloc2 unit_id bool_id:
                                Result::Ok alloc3:
                                    let inner_fn %SelfhostTypeId selfhost_type_arena_alloc_type_id &alloc3
                                    match add_zero_arg_function selfhost_type_arena_alloc_into_arena alloc3 inner_fn:
                                        Result::Ok alloc4:
                                            let outer_fn %SelfhostTypeId selfhost_type_arena_alloc_type_id &alloc4
                                            let type_arena %SelfhostTypeArena selfhost_type_arena_alloc_into_arena alloc4
                                            match selfhost_canonical_type_key_arena_new:
                                                Result::Ok key_arena0:
                                                    match selfhost_canonical_type_key_project_into_arena &type_arena key_arena0 outer_fn:
                                                        Result::Ok key_alloc1:
                                                            let outer_key1 %SelfhostCanonicalTypeKeyId selfhost_canonical_type_key_arena_alloc_key_id &key_alloc1
                                                            let key_arena1 %SelfhostCanonicalTypeKeyArena selfhost_canonical_type_key_arena_alloc_into_arena key_alloc1
                                                            match selfhost_canonical_type_key_project_into_arena &type_arena key_arena1 inner_fn:
                                                                Result::Ok key_alloc2:
                                                                    let inner_key %SelfhostCanonicalTypeKeyId selfhost_canonical_type_key_arena_alloc_key_id &key_alloc2
                                                                    let key_arena2 %SelfhostCanonicalTypeKeyArena selfhost_canonical_type_key_arena_alloc_into_arena key_alloc2
                                                                    match selfhost_canonical_type_key_project_into_arena &type_arena key_arena2 outer_fn:
                                                                        Result::Ok key_alloc3:
                                                                            let outer_key2 %SelfhostCanonicalTypeKeyId selfhost_canonical_type_key_arena_alloc_key_id &key_alloc3
                                                                            let key_arena3 %SelfhostCanonicalTypeKeyArena selfhost_canonical_type_key_arena_alloc_into_arena key_alloc3
                                                                            let checks1 checks_push checks0 check selfhost_canonical_type_key_equal &key_arena3 outer_key1 outer_key2
                                                                            let checks2 checks_push checks1 check not selfhost_canonical_type_key_equal &key_arena3 outer_key1 inner_key
                                                                            let checks3 checks_push checks2 check_eq_i32 11 selfhost_canonical_type_key_arena_node_len &key_arena3
                                                                            let checks4 checks_push checks3 check_eq_i32 3 selfhost_canonical_type_key_arena_arg_len &key_arena3
                                                                            selfhost_canonical_type_key_arena_free key_arena3
                                                                            selfhost_type_arena_free type_arena
                                                                            let shown checks_print_report checks4
                                                                            checks_exit_code shown
                                                                        Result::Err _e:
                                                                            selfhost_type_arena_free type_arena
                                                                            let checks1 checks_push checks0 Result::Err "second outer function canonical key projection failed"
                                                                            let shown checks_print_report checks1
                                                                            checks_exit_code shown
                                                                Result::Err _e:
                                                                    selfhost_type_arena_free type_arena
                                                                    let checks1 checks_push checks0 Result::Err "inner function canonical key projection failed"
                                                                    let shown checks_print_report checks1
                                                                    checks_exit_code shown
                                                        Result::Err _e:
                                                            selfhost_type_arena_free type_arena
                                                            let checks1 checks_push checks0 Result::Err "first outer function canonical key projection failed"
                                                            let shown checks_print_report checks1
                                                            checks_exit_code shown
                                                Result::Err _e:
                                                    selfhost_type_arena_free type_arena
                                                    let checks1 checks_push checks0 Result::Err "canonical key arena allocation failed"
                                                    let shown checks_print_report checks1
                                                    checks_exit_code shown
                                        Result::Err _e:
                                            let checks1 checks_push checks0 Result::Err "outer function allocation failed"
                                            let shown checks_print_report checks1
                                            checks_exit_code shown
                                Result::Err _e:
                                    let checks1 checks_push checks0 Result::Err "inner function allocation failed"
                                    let shown checks_print_report checks1
                                    checks_exit_code shown
                        Result::Err _e:
                            let checks1 checks_push checks0 Result::Err "bool type allocation failed"
                            let shown checks_print_report checks1
                            checks_exit_code shown
                Result::Err _e:
                    let checks1 checks_push checks0 Result::Err "unit type allocation failed"
                    let shown checks_print_report checks1
                    checks_exit_code shown
        Result::Err _e:
            let checks1 checks_push checks0 Result::Err "type arena allocation failed"
            let shown checks_print_report checks1
            checks_exit_code shown
```

## projects_type_parameters_by_binder_identity

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

#import "core/math" as *
#import "core/result" as *
#import "neplg2/core/ty/ty" as *
#import "std/test" as *

fn main %impure fn void i32 \void:
    let checks0 checks_new
    match selfhost_type_arena_new:
        Result::Ok arena0:
            let t_binding %SelfhostTypeParameterBinding selfhost_type_parameter_binding_new_unchecked 0 0
            let e_binding %SelfhostTypeParameterBinding selfhost_type_parameter_binding_new_unchecked 0 1
            match selfhost_type_arena_add_type_parameter arena0 t_binding:
                Result::Ok alloc1:
                    let t_first %SelfhostTypeId selfhost_type_arena_alloc_type_id &alloc1
                    match selfhost_type_arena_add_type_parameter selfhost_type_arena_alloc_into_arena alloc1 t_binding:
                        Result::Ok alloc2:
                            let t_second %SelfhostTypeId selfhost_type_arena_alloc_type_id &alloc2
                            match selfhost_type_arena_add_type_parameter selfhost_type_arena_alloc_into_arena alloc2 e_binding:
                                Result::Ok alloc3:
                                    let e_type %SelfhostTypeId selfhost_type_arena_alloc_type_id &alloc3
                                    let type_arena %SelfhostTypeArena selfhost_type_arena_alloc_into_arena alloc3
                                    match selfhost_canonical_type_key_arena_new:
                                        Result::Ok key_arena0:
                                            match selfhost_canonical_type_key_project_into_arena &type_arena key_arena0 t_first:
                                                Result::Ok key_alloc1:
                                                    let key_t_first %SelfhostCanonicalTypeKeyId selfhost_canonical_type_key_arena_alloc_key_id &key_alloc1
                                                    let key_arena1 %SelfhostCanonicalTypeKeyArena selfhost_canonical_type_key_arena_alloc_into_arena key_alloc1
                                                    match selfhost_canonical_type_key_project_into_arena &type_arena key_arena1 t_second:
                                                        Result::Ok key_alloc2:
                                                            let key_t_second %SelfhostCanonicalTypeKeyId selfhost_canonical_type_key_arena_alloc_key_id &key_alloc2
                                                            let key_arena2 %SelfhostCanonicalTypeKeyArena selfhost_canonical_type_key_arena_alloc_into_arena key_alloc2
                                                            match selfhost_canonical_type_key_project_into_arena &type_arena key_arena2 e_type:
                                                                Result::Ok key_alloc3:
                                                                    let key_e %SelfhostCanonicalTypeKeyId selfhost_canonical_type_key_arena_alloc_key_id &key_alloc3
                                                                    let key_arena3 %SelfhostCanonicalTypeKeyArena selfhost_canonical_type_key_arena_alloc_into_arena key_alloc3
                                                                    let checks1 checks_push checks0 check selfhost_canonical_type_key_equal &key_arena3 key_t_first key_t_second
                                                                    let checks2 checks_push checks1 check not selfhost_canonical_type_key_equal &key_arena3 key_t_first key_e
                                                                    let checks3 checks_push checks2 check_eq_i32 3 selfhost_canonical_type_key_arena_node_len &key_arena3
                                                                    let checks4 checks_push checks3 check_eq_i32 0 selfhost_canonical_type_key_arena_arg_len &key_arena3
                                                                    selfhost_canonical_type_key_arena_free key_arena3
                                                                    selfhost_type_arena_free type_arena
                                                                    let shown checks_print_report checks4
                                                                    checks_exit_code shown
                                                                Result::Err _e:
                                                                    selfhost_type_arena_free type_arena
                                                                    let checks1 checks_push checks0 Result::Err "E parameter canonical key projection failed"
                                                                    let shown checks_print_report checks1
                                                                    checks_exit_code shown
                                                        Result::Err _e:
                                                            selfhost_type_arena_free type_arena
                                                            let checks1 checks_push checks0 Result::Err "second T parameter canonical key projection failed"
                                                            let shown checks_print_report checks1
                                                            checks_exit_code shown
                                                Result::Err _e:
                                                    selfhost_type_arena_free type_arena
                                                    let checks1 checks_push checks0 Result::Err "first T parameter canonical key projection failed"
                                                    let shown checks_print_report checks1
                                                    checks_exit_code shown
                                        Result::Err _e:
                                            selfhost_type_arena_free type_arena
                                            let checks1 checks_push checks0 Result::Err "canonical key arena allocation failed"
                                            let shown checks_print_report checks1
                                            checks_exit_code shown
                                Result::Err _e:
                                    let checks1 checks_push checks0 Result::Err "E parameter allocation failed"
                                    let shown checks_print_report checks1
                                    checks_exit_code shown
                        Result::Err _e:
                            let checks1 checks_push checks0 Result::Err "second T parameter allocation failed"
                            let shown checks_print_report checks1
                            checks_exit_code shown
                Result::Err _e:
                    let checks1 checks_push checks0 Result::Err "first T parameter allocation failed"
                    let shown checks_print_report checks1
                    checks_exit_code shown
        Result::Err _e:
            let checks1 checks_push checks0 Result::Err "type arena allocation failed"
            let shown checks_print_report checks1
            checks_exit_code shown
```
