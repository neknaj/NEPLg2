# NEPLg2 self-host type constructor type parameters

## two_arity_constructor_accepts_type_parameter_arguments

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

#import "alloc/collections/vec" as v
#import "core/option" as *
#import "core/result" as *
#import "neplg2/core/infra/span" as *
#import "neplg2/core/resolve/type_resolver" as *
#import "std/test" as *

fn push_named_item %impure fn Vec SelfhostTypePrefixItem impure fn i32 impure fn i32 impure fn i32 Result Vec SelfhostTypePrefixItem str \items\token_index\start\end:
    let span %SelfhostSourceSpan source_span_new_unchecked 0 start end
    let item %SelfhostTypePrefixItem selfhost_type_prefix_item_new SelfhostTypePrefixItemKind::NamedType token_index span
    match v::push items item:
        Result::Ok next_items:
            Result::Ok next_items
        Result::Err e:
            v::free v::vec_push_error_vec e
            Result::Err "prefix item push failed"

fn build_result_t_e_list %impure fn void Result SelfhostTypePrefixList str \void:
    let items_result %Result Vec SelfhostTypePrefixItem StdErrorKind v::new
    match items_result:
        Result::Ok items0:
            match push_named_item items0 0 0 6:
                Result::Ok items1:
                    match push_named_item items1 1 7 8:
                        Result::Ok items2:
                            match push_named_item items2 2 9 10:
                                Result::Ok items3:
                                    Result::Ok selfhost_type_prefix_list_new items3
                                Result::Err e:
                                    Result::Err e
                        Result::Err e:
                            Result::Err e
                Result::Err e:
                    Result::Err e
        Result::Err _e:
            Result::Err "prefix item allocation failed"

fn add_param %impure fn SelfhostTypeParameterEnv impure fn str impure fn i32 Result SelfhostTypeParameterEnvAddResult str \env\name\kind_arity:
    let span %SelfhostSourceSpan source_span_empty_unchecked 0 0
    match selfhost_type_parameter_env_add_checked env name kind_arity span:
        Result::Ok added:
            Result::Ok added
        Result::Err _e:
            Result::Err "type parameter add failed"

fn reduce_list_with_type_parameters %impure fn SelfhostTypePrefixList impure fn &SelfhostTypeConstructorTable impure fn &SelfhostTypeParameterEnv Result SelfhostResolvedTypeTreeRoot str \list\constructors\params:
    match selfhost_type_prefix_list_reduce_with_constructors_and_type_parameters "Result T E" constructors params &list:
        Result::Ok root:
            selfhost_type_prefix_list_free list
            Result::Ok root
        Result::Err _e:
            selfhost_type_prefix_list_free list
            Result::Err "type parameter reduce failed"

fn check_node_kind %fn &SelfhostResolvedTypeTree fn SelfhostResolvedTypeNodeId fn SelfhostResolvedTypeNodeKind Result unit str \tree\node_id\expected:
    match selfhost_resolved_type_tree_get_node_kind tree node_id:
        Option::Some actual:
            if selfhost_resolved_type_node_kind_eq actual expected Result::Ok unit Result::Err "node kind mismatch"
        Option::None:
            Result::Err "node kind missing"

fn check_i32_option %fn Option i32 fn i32 Result unit str \actual\expected:
    match actual:
        Option::Some value:
            check_eq_i32 expected value
        Option::None:
            Result::Err "i32 option missing"

fn check_applied_arg_parameter %fn &SelfhostResolvedTypeTree fn SelfhostResolvedTypeNodeId fn i32 fn SelfhostTypeParameterId Result unit str \tree\root_id\idx\expected:
    match selfhost_resolved_type_tree_applied_arg tree root_id idx:
        Option::Some arg_id:
            match selfhost_resolved_type_tree_parameter_id tree arg_id:
                Option::Some actual:
                    if selfhost_type_parameter_id_eq actual expected Result::Ok unit Result::Err "parameter id mismatch"
                Option::None:
                    Result::Err "parameter id missing"
        Option::None:
            Result::Err "applied parameter argument missing"

fn main %impure fn void i32 \void:
    let checks0 checks_new
    match selfhost_type_constructor_table_new:
        Result::Ok constructors0:
            let span %SelfhostSourceSpan source_span_empty_unchecked 0 0
            match selfhost_type_constructor_table_add_checked constructors0 "Result" 2 span:
                Result::Ok constructor_added:
                    let constructors %SelfhostTypeConstructorTable selfhost_type_constructor_add_result_into_table constructor_added
                    match selfhost_type_parameter_env_new:
                        Result::Ok params0:
                            match add_param params0 "T" 0:
                                Result::Ok added_t:
                                    let t_id %SelfhostTypeParameterId selfhost_type_parameter_env_add_result_parameter_id &added_t
                                    let params1 %SelfhostTypeParameterEnv selfhost_type_parameter_env_add_result_into_env added_t
                                    match add_param params1 "E" 0:
                                        Result::Ok added_e:
                                            let e_id %SelfhostTypeParameterId selfhost_type_parameter_env_add_result_parameter_id &added_e
                                            let params %SelfhostTypeParameterEnv selfhost_type_parameter_env_add_result_into_env added_e
                                            match build_result_t_e_list:
                                                Result::Ok list:
                                                    match reduce_list_with_type_parameters list &constructors &params:
                                                        Result::Ok root:
                                                            let tree %&SelfhostResolvedTypeTree selfhost_resolved_type_tree_root_tree &root
                                                            let root_id %SelfhostResolvedTypeNodeId selfhost_resolved_type_tree_root_id &root
                                                            let checks1 checks_push checks0 check_node_kind tree root_id SelfhostResolvedTypeNodeKind::Applied
                                                            let checks2 checks_push checks1 check_i32_option (selfhost_resolved_type_tree_applied_arg_count tree root_id) 2
                                                            let checks3 checks_push checks2 check_applied_arg_parameter tree root_id 0 t_id
                                                            let checks4 checks_push checks3 check_applied_arg_parameter tree root_id 1 e_id
                                                            selfhost_resolved_type_tree_root_free root
                                                            selfhost_type_parameter_env_free params
                                                            selfhost_type_constructor_table_free constructors
                                                            let shown checks_print_report checks4
                                                            checks_exit_code shown
                                                        Result::Err e:
                                                            selfhost_type_parameter_env_free params
                                                            selfhost_type_constructor_table_free constructors
                                                            let checks1 checks_push checks0 Result::Err e
                                                            let shown checks_print_report checks1
                                                            checks_exit_code shown
                                                Result::Err e:
                                                    selfhost_type_parameter_env_free params
                                                    selfhost_type_constructor_table_free constructors
                                                    let checks1 checks_push checks0 Result::Err e
                                                    let shown checks_print_report checks1
                                                    checks_exit_code shown
                                        Result::Err e:
                                            selfhost_type_constructor_table_free constructors
                                            let checks1 checks_push checks0 Result::Err e
                                            let shown checks_print_report checks1
                                            checks_exit_code shown
                                Result::Err e:
                                    selfhost_type_constructor_table_free constructors
                                    let checks1 checks_push checks0 Result::Err e
                                    let shown checks_print_report checks1
                                    checks_exit_code shown
                        Result::Err _e:
                            selfhost_type_constructor_table_free constructors
                            let checks1 checks_push checks0 Result::Err "type parameter env allocation failed"
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
