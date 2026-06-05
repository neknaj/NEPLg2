# NEPLg2 self-host type constructor projection

## two_arity_constructor_projects_type_parameter_arguments

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

#import "alloc/collections/vec" as v
#import "core/option" as *
#import "core/result" as *
#import "neplg2/core/infra/span" as *
#import "neplg2/core/resolve/type_resolver" as *
#import "neplg2/core/ty/ty" as *
#import "std/test" as *

fn push_node_id %impure fn Vec SelfhostResolvedTypeNodeId impure fn SelfhostResolvedTypeNodeId Result Vec SelfhostResolvedTypeNodeId str \items\node_id:
    match v::push items node_id:
        Result::Ok next_items:
            Result::Ok next_items
        Result::Err e:
            v::free v::vec_push_error_vec e
            Result::Err "type argument push failed"

fn build_result_t_e_root %impure fn SelfhostNamedTypeId Result SelfhostResolvedTypeTreeRoot str \result_id:
    match selfhost_resolved_type_tree_new:
        Result::Ok tree0:
            let t_id %SelfhostTypeParameterId selfhost_type_parameter_id_new 0
            match selfhost_resolved_type_tree_add_parameter tree0 t_id source_span_empty_unchecked 0 0:
                Result::Ok t_added:
                    let t_node %SelfhostResolvedTypeNodeId selfhost_resolved_type_tree_alloc_node_id &t_added
                    let tree1 %SelfhostResolvedTypeTree selfhost_resolved_type_tree_alloc_into_tree t_added
                    let e_id %SelfhostTypeParameterId selfhost_type_parameter_id_new 1
                    match selfhost_resolved_type_tree_add_parameter tree1 e_id source_span_empty_unchecked 0 0:
                        Result::Ok e_added:
                            let e_node %SelfhostResolvedTypeNodeId selfhost_resolved_type_tree_alloc_node_id &e_added
                            let tree2 %SelfhostResolvedTypeTree selfhost_resolved_type_tree_alloc_into_tree e_added
                            let args_result %Result Vec SelfhostResolvedTypeNodeId StdErrorKind v::new
                            match args_result:
                                Result::Ok args0:
                                    match push_node_id args0 t_node:
                                        Result::Ok args1:
                                            match push_node_id args1 e_node:
                                                Result::Ok args2:
                                                    let span %SelfhostSourceSpan source_span_empty_unchecked 0 0
                                                    match selfhost_resolved_type_tree_add_applied_named tree2 result_id span args2:
                                                        Result::Ok applied_added:
                                                            let root_node %SelfhostResolvedTypeNodeId selfhost_resolved_type_tree_alloc_node_id &applied_added
                                                            let tree3 %SelfhostResolvedTypeTree selfhost_resolved_type_tree_alloc_into_tree applied_added
                                                            Result::Ok selfhost_resolved_type_tree_root_new tree3 root_node
                                                        Result::Err _e:
                                                            Result::Err "applied node allocation failed"
                                                Result::Err e:
                                                    selfhost_resolved_type_tree_free tree2
                                                    Result::Err e
                                        Result::Err e:
                                            selfhost_resolved_type_tree_free tree2
                                            Result::Err e
                                Result::Err _e:
                                    selfhost_resolved_type_tree_free tree2
                                    Result::Err "type argument vector allocation failed"
                        Result::Err _e:
                            Result::Err "E parameter node allocation failed"
                Result::Err _e:
                    Result::Err "T parameter node allocation failed"
        Result::Err _e:
            Result::Err "resolved tree allocation failed"

fn check_i32_option %fn Option i32 fn i32 Result unit str \actual\expected:
    match actual:
        Option::Some value:
            check_eq_i32 expected value
        Option::None:
            Result::Err "i32 option missing"

fn check_type_kind_option %fn &SelfhostTypeArena fn SelfhostTypeId fn SelfhostTypeKind Result unit str \arena\type_id\expected:
    match selfhost_type_arena_get_kind arena type_id:
        Option::Some actual:
            if selfhost_type_kind_eq actual expected Result::Ok unit Result::Err "type kind mismatch"
        Option::None:
            Result::Err "type kind missing"

fn check_named_id_option %fn Option SelfhostNamedTypeId fn SelfhostNamedTypeId Result unit str \actual\expected:
    match actual:
        Option::Some actual_id:
            if selfhost_named_type_id_eq actual_id expected Result::Ok unit Result::Err "constructor id mismatch"
        Option::None:
            Result::Err "constructor id missing"

fn check_binding_option %fn Option SelfhostTypeParameterBinding fn SelfhostTypeParameterBinding Result unit str \actual\expected:
    match actual:
        Option::Some binding:
            if selfhost_type_parameter_binding_eq binding expected Result::Ok unit Result::Err "parameter binding mismatch"
        Option::None:
            Result::Err "parameter binding missing"

fn check_arena_applied_arg_binding %fn &SelfhostTypeArena fn SelfhostTypeId fn i32 fn SelfhostTypeParameterBinding Result unit str \arena\root_id\idx\expected:
    match selfhost_type_arena_applied_arg arena root_id idx:
        Option::Some arg_type_id:
            check_binding_option (selfhost_type_arena_type_parameter_binding arena arg_type_id) expected
        Option::None:
            Result::Err "applied argument missing"

fn main %impure fn void i32 \void:
    let checks0 checks_new
    let source %str ""
    match selfhost_type_constructor_table_new:
        Result::Ok constructors0:
            let span %SelfhostSourceSpan source_span_empty_unchecked 0 0
            match selfhost_type_constructor_table_add_checked constructors0 "Result" 2 span:
                Result::Ok added:
                    let result_id %SelfhostNamedTypeId selfhost_type_constructor_add_result_nominal_id &added
                    let constructors %SelfhostTypeConstructorTable selfhost_type_constructor_add_result_into_table added
                    match build_result_t_e_root result_id:
                        Result::Ok root:
                            match selfhost_type_arena_new:
                                Result::Ok arena0:
                                    match selfhost_type_project_root_with_constructors_into_arena arena0 &source &constructors &root:
                                        Result::Ok projected:
                                            let type_id %SelfhostTypeId selfhost_type_arena_alloc_type_id &projected
                                            let arena1 %SelfhostTypeArena selfhost_type_arena_alloc_into_arena projected
                                            let t_binding %SelfhostTypeParameterBinding selfhost_type_parameter_binding_new_unchecked 0 0
                                            let e_binding %SelfhostTypeParameterBinding selfhost_type_parameter_binding_new_unchecked 0 1
                                            let checks1 checks_push checks0 check_type_kind_option &arena1 type_id SelfhostTypeKind::Named
                                            let checks2 checks_push checks1 check_named_id_option (selfhost_type_arena_applied_constructor_id &arena1 type_id) result_id
                                            let checks3 checks_push checks2 check_i32_option (selfhost_type_arena_applied_arg_count &arena1 type_id) 2
                                            let checks4 checks_push checks3 check_arena_applied_arg_binding &arena1 type_id 0 t_binding
                                            let checks5 checks_push checks4 check_arena_applied_arg_binding &arena1 type_id 1 e_binding
                                            let checks6 checks_push checks5 check_eq_i32 3 selfhost_type_arena_len &arena1
                                            selfhost_type_arena_free arena1
                                            selfhost_resolved_type_tree_root_free root
                                            selfhost_type_constructor_table_free constructors
                                            let shown checks_print_report checks6
                                            checks_exit_code shown
                                        Result::Err _e:
                                            selfhost_resolved_type_tree_root_free root
                                            selfhost_type_constructor_table_free constructors
                                            let checks1 checks_push checks0 Result::Err "projection failed"
                                            let shown checks_print_report checks1
                                            checks_exit_code shown
                                Result::Err _e:
                                    selfhost_resolved_type_tree_root_free root
                                    selfhost_type_constructor_table_free constructors
                                    let checks1 checks_push checks0 Result::Err "type arena allocation failed"
                                    let shown checks_print_report checks1
                                    checks_exit_code shown
                        Result::Err e:
                            selfhost_type_constructor_table_free constructors
                            let checks1 checks_push checks0 Result::Err e
                            let shown checks_print_report checks1
                            checks_exit_code shown
                Result::Err _e:
                    let checks1 checks_push checks0 Result::Err "Result constructor add failed"
                    let shown checks_print_report checks1
                    checks_exit_code shown
        Result::Err _e:
            let checks1 checks_push checks0 Result::Err "constructor table allocation failed"
            let shown checks_print_report checks1
            checks_exit_code shown
```
