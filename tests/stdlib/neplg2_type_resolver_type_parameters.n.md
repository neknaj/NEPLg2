# NEPLg2 self-host type resolver type parameters

## reduces_type_parameter_inside_generic_application

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: mlstr:
    ##: Checked [ok,ok,ok,ok,ok]
    ##: [0] ok
    ##: [1] ok
    ##: [2] ok
    ##: [3] ok
    ##: [4] ok
```neplg2
#entry main
#target std
#indent 4

#import "alloc/collections/vec" as v
#import "core/option" as *
#import "core/result" as *
#import "neplg2/core/infra/span" as *
#import "neplg2/core/resolve/type_resolver" as *
#import "neplg2/core/syntax/lexer" as *
#import "neplg2/core/syntax/parser/module_parser" as *
#import "neplg2/core/ty/ty/id" as *
#import "std/test" as *

fn reduce_header_with_type_parameters %impure fn str impure fn &SelfhostTypeConstructorTable impure fn &SelfhostTypeParameterEnv Result SelfhostResolvedTypeTreeRoot str \source\constructors\type_parameters:
    match lex_all source:
        Result::Ok tokens:
            let range %SelfhostSyntaxRange selfhost_parser_header_type_annotation_range &tokens v::len &tokens 0
            match selfhost_type_prefix_list_from_syntax_range &tokens range:
                Result::Ok list:
                    match selfhost_type_prefix_list_reduce_with_constructors_and_type_parameters source constructors type_parameters &list:
                        Result::Ok root:
                            selfhost_type_prefix_list_free list
                            v::free tokens
                            Result::Ok root
                        Result::Err _e:
                            selfhost_type_prefix_list_free list
                            v::free tokens
                            Result::Err "reduce failed"
                Result::Err _e:
                    v::free tokens
                    Result::Err "prefix list build failed"
        Result::Err _diag:
            Result::Err "lex failed"

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

fn check_named_id_option %fn Option SelfhostNamedTypeId fn SelfhostNamedTypeId Result unit str \actual\expected:
    match actual:
        Option::Some nominal_id:
            if selfhost_named_type_id_eq nominal_id expected Result::Ok unit Result::Err "named id mismatch"
        Option::None:
            Result::Err "named id missing"

fn check_applied_arg_kind %fn &SelfhostResolvedTypeTree fn SelfhostResolvedTypeNodeId fn SelfhostResolvedTypeNodeKind Result unit str \tree\root_id\expected:
    match selfhost_resolved_type_tree_applied_arg tree root_id 0:
        Option::Some arg_id:
            check_node_kind tree arg_id expected
        Option::None:
            Result::Err "applied argument missing"

fn check_applied_arg_parameter %fn &SelfhostResolvedTypeTree fn SelfhostResolvedTypeNodeId fn SelfhostTypeParameterId Result unit str \tree\root_id\expected:
    match selfhost_resolved_type_tree_applied_arg tree root_id 0:
        Option::Some arg_id:
            match selfhost_resolved_type_tree_parameter_id tree arg_id:
                Option::Some actual:
                    if selfhost_type_parameter_id_eq actual expected Result::Ok unit Result::Err "parameter id mismatch"
                Option::None:
                    Result::Err "parameter id missing"
        Option::None:
            Result::Err "applied argument missing"

fn main %impure fn void i32 \void:
    let checks0 checks_new
    let source %str "fn use_box %Box T \\x:\n    x\n"
    match selfhost_type_constructor_table_new:
        Result::Ok constructors0:
            match selfhost_type_constructor_table_add constructors0 "Box" 1 source_span_empty_unchecked 0 0:
                Result::Ok constructor_added:
                    let box_id %SelfhostNamedTypeId selfhost_type_constructor_add_result_nominal_id &constructor_added
                    let constructors %SelfhostTypeConstructorTable selfhost_type_constructor_add_result_into_table constructor_added
                    match selfhost_type_parameter_env_new:
                        Result::Ok params0:
                            match selfhost_type_parameter_env_add_checked params0 "T" 0 source_span_empty_unchecked 0 0:
                                Result::Ok parameter_added:
                                    let t_id %SelfhostTypeParameterId selfhost_type_parameter_env_add_result_parameter_id &parameter_added
                                    let params %SelfhostTypeParameterEnv selfhost_type_parameter_env_add_result_into_env parameter_added
                                    match reduce_header_with_type_parameters source &constructors &params:
                                        Result::Ok root:
                                            let tree %&SelfhostResolvedTypeTree selfhost_resolved_type_tree_root_tree &root
                                            let root_id %SelfhostResolvedTypeNodeId selfhost_resolved_type_tree_root_id &root
                                            let checks1 checks_push checks0 check_node_kind tree root_id SelfhostResolvedTypeNodeKind::Applied
                                            let checks2 checks_push checks1 check_named_id_option (selfhost_resolved_type_tree_applied_constructor_id tree root_id) box_id
                                            let checks3 checks_push checks2 check_i32_option (selfhost_resolved_type_tree_applied_arg_count tree root_id) 1
                                            let checks4 checks_push checks3 check_applied_arg_kind tree root_id SelfhostResolvedTypeNodeKind::Parameter
                                            let checks5 checks_push checks4 check_applied_arg_parameter tree root_id t_id
                                            selfhost_resolved_type_tree_root_free root
                                            selfhost_type_parameter_env_free params
                                            selfhost_type_constructor_table_free constructors
                                            let shown checks_print_report checks5
                                            checks_exit_code shown
                                        Result::Err e:
                                            selfhost_type_parameter_env_free params
                                            selfhost_type_constructor_table_free constructors
                                            let checks1 checks_push checks0 Result::Err e
                                            let shown checks_print_report checks1
                                            checks_exit_code shown
                                Result::Err _e:
                                    selfhost_type_constructor_table_free constructors
                                    let checks1 checks_push checks0 Result::Err "type parameter add failed"
                                    let shown checks_print_report checks1
                                    checks_exit_code shown
                        Result::Err _e:
                            selfhost_type_constructor_table_free constructors
                            let checks1 checks_push checks0 Result::Err "type parameter env allocation failed"
                            let shown checks_print_report checks1
                            checks_exit_code shown
                Result::Err _e:
                    let checks1 checks_push checks0 Result::Err "constructor add failed"
                    let shown checks_print_report checks1
                    checks_exit_code shown
        Result::Err _e:
            let checks1 checks_push checks0 Result::Err "constructor table allocation failed"
            let shown checks_print_report checks1
            checks_exit_code shown
```

## projects_resolved_type_parameter_application_into_type_arena

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
#import "neplg2/core/ty/ty" as *
#import "std/test" as *

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

fn check_binding_option %fn Option SelfhostTypeParameterBinding fn SelfhostTypeParameterBinding Result unit str \actual\expected:
    match actual:
        Option::Some binding:
            if selfhost_type_parameter_binding_eq binding expected Result::Ok unit Result::Err "parameter binding mismatch"
        Option::None:
            Result::Err "parameter binding missing"

fn check_arena_applied_arg_binding %fn &SelfhostTypeArena fn SelfhostTypeId fn SelfhostTypeParameterBinding Result unit str \arena\root_id\expected:
    match selfhost_type_arena_applied_arg arena root_id 0:
        Option::Some arg_type_id:
            check_binding_option (selfhost_type_arena_type_parameter_binding arena arg_type_id) expected
        Option::None:
            Result::Err "arena applied argument missing"

fn build_box_t_root %impure fn SelfhostNamedTypeId Result SelfhostResolvedTypeTreeRoot str \box_id:
    match selfhost_resolved_type_tree_new:
        Result::Ok tree0:
            let t_id %SelfhostTypeParameterId selfhost_type_parameter_id_new 0
            match selfhost_resolved_type_tree_add_parameter tree0 t_id source_span_empty_unchecked 0 0:
                Result::Ok parameter_added:
                    let parameter_node %SelfhostResolvedTypeNodeId selfhost_resolved_type_tree_alloc_node_id &parameter_added
                    let tree1 %SelfhostResolvedTypeTree selfhost_resolved_type_tree_alloc_into_tree parameter_added
                    let args_result %Result Vec SelfhostResolvedTypeNodeId StdErrorKind v::new
                    match args_result:
                        Result::Ok args0:
                            match v::push args0 parameter_node:
                                Result::Ok args1:
                                    match selfhost_resolved_type_tree_add_applied_named tree1 box_id source_span_empty_unchecked 0 0 args1:
                                        Result::Ok applied_added:
                                            let root_node %SelfhostResolvedTypeNodeId selfhost_resolved_type_tree_alloc_node_id &applied_added
                                            let tree2 %SelfhostResolvedTypeTree selfhost_resolved_type_tree_alloc_into_tree applied_added
                                            Result::Ok selfhost_resolved_type_tree_root_new tree2 root_node
                                        Result::Err _e:
                                            Result::Err "applied type node allocation failed"
                                Result::Err e:
                                    let owned %Vec SelfhostResolvedTypeNodeId v::vec_push_error_vec e
                                    v::free owned
                                    selfhost_resolved_type_tree_free tree1
                                    Result::Err "type argument vector push failed"
                        Result::Err _e:
                            selfhost_resolved_type_tree_free tree1
                            Result::Err "type argument vector allocation failed"
                Result::Err _e:
                    Result::Err "parameter node allocation failed"
        Result::Err _e:
            Result::Err "resolved tree allocation failed"

fn main %impure fn void i32 \void:
    let checks0 checks_new
    let source %str ""
    match selfhost_type_constructor_table_new:
        Result::Ok constructors0:
            match selfhost_type_constructor_table_add constructors0 "Box" 1 source_span_empty_unchecked 0 0:
                Result::Ok constructor_added:
                    let box_id %SelfhostNamedTypeId selfhost_type_constructor_add_result_nominal_id &constructor_added
                    let constructors %SelfhostTypeConstructorTable selfhost_type_constructor_add_result_into_table constructor_added
                    match build_box_t_root box_id:
                        Result::Ok root:
                            match selfhost_type_arena_new:
                                Result::Ok arena0:
                                    match selfhost_type_project_root_with_constructors_into_arena arena0 &source &constructors &root:
                                        Result::Ok projected:
                                            let type_id %SelfhostTypeId selfhost_type_arena_alloc_type_id &projected
                                            let arena1 %SelfhostTypeArena selfhost_type_arena_alloc_into_arena projected
                                            let expected_binding %SelfhostTypeParameterBinding selfhost_type_parameter_binding_new_unchecked 0 0
                                            let checks1 checks_push checks0 check_type_kind_option &arena1 type_id SelfhostTypeKind::Named
                                            let checks2 checks_push checks1 check_i32_option (selfhost_type_arena_applied_arg_count &arena1 type_id) 1
                                            let checks3 checks_push checks2 check_arena_applied_arg_binding &arena1 type_id expected_binding
                                            let checks4 checks_push checks3 check_eq_i32 2 selfhost_type_arena_len &arena1
                                            selfhost_type_arena_free arena1
                                            selfhost_resolved_type_tree_root_free root
                                            selfhost_type_constructor_table_free constructors
                                            let shown checks_print_report checks4
                                            checks_exit_code shown
                                        Result::Err _e:
                                            selfhost_resolved_type_tree_root_free root
                                            selfhost_type_constructor_table_free constructors
                                            let checks1 checks_push checks0 Result::Err "type parameter projection failed"
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
                    let checks1 checks_push checks0 Result::Err "constructor add failed"
                    let shown checks_print_report checks1
                    checks_exit_code shown
        Result::Err _e:
            let checks1 checks_push checks0 Result::Err "constructor table allocation failed"
            let shown checks_print_report checks1
            checks_exit_code shown
```

## rejects_type_parameter_constructor_name_conflict

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: mlstr:
    ##: Checked [ok]
    ##: [0] ok
```neplg2
#entry main
#target std
#indent 4

#import "alloc/collections/vec" as v
#import "core/option" as *
#import "core/result" as *
#import "neplg2/core/infra/span" as *
#import "neplg2/core/resolve/type_resolver" as *
#import "neplg2/core/syntax/lexer" as *
#import "neplg2/core/syntax/parser/module_parser" as *
#import "neplg2/core/ty/ty" as *
#import "std/test" as *

fn reduce_header_error_kind_with_type_parameters %impure fn str impure fn &SelfhostTypeConstructorTable impure fn &SelfhostTypeParameterEnv Result SelfhostTypeReduceErrorKind str \source\constructors\type_parameters:
    match lex_all source:
        Result::Ok tokens:
            let range %SelfhostSyntaxRange selfhost_parser_header_type_annotation_range &tokens v::len &tokens 0
            match selfhost_type_prefix_list_from_syntax_range &tokens range:
                Result::Ok list:
                    match selfhost_type_prefix_list_reduce_with_constructors_and_type_parameters source constructors type_parameters &list:
                        Result::Ok root:
                            selfhost_resolved_type_tree_root_free root
                            selfhost_type_prefix_list_free list
                            v::free tokens
                            Result::Err "reduction unexpectedly succeeded"
                        Result::Err e:
                            selfhost_type_prefix_list_free list
                            v::free tokens
                            Result::Ok e.kind
                Result::Err _e:
                    v::free tokens
                    Result::Err "prefix list build failed"
        Result::Err _diag:
            Result::Err "lex failed"

fn main %impure fn void i32 \void:
    let checks0 checks_new
    let source %str "fn use_t %T \\x:\n    x\n"
    match selfhost_type_constructor_table_new:
        Result::Ok constructors0:
            match selfhost_type_constructor_table_add constructors0 "T" 0 source_span_empty_unchecked 0 0:
                Result::Ok constructor_added:
                    let constructors %SelfhostTypeConstructorTable selfhost_type_constructor_add_result_into_table constructor_added
                    match selfhost_type_parameter_env_new:
                        Result::Ok params0:
                            match selfhost_type_parameter_env_add_checked params0 "T" 0 source_span_empty_unchecked 0 0:
                                Result::Ok parameter_added:
                                    let params %SelfhostTypeParameterEnv selfhost_type_parameter_env_add_result_into_env parameter_added
                                    match reduce_header_error_kind_with_type_parameters source &constructors &params:
                                        Result::Ok kind:
                                            let checks1 checks_push checks0:
                                                if selfhost_type_reduce_error_kind_eq kind SelfhostTypeReduceErrorKind::TypeParameterConstructorNameConflict Result::Ok unit Result::Err "wrong reduce error kind"
                                            selfhost_type_parameter_env_free params
                                            selfhost_type_constructor_table_free constructors
                                            let shown checks_print_report checks1
                                            checks_exit_code shown
                                        Result::Err e:
                                            selfhost_type_parameter_env_free params
                                            selfhost_type_constructor_table_free constructors
                                            let checks1 checks_push checks0 Result::Err e
                                            let shown checks_print_report checks1
                                            checks_exit_code shown
                                Result::Err _e:
                                    selfhost_type_constructor_table_free constructors
                                    let checks1 checks_push checks0 Result::Err "type parameter add failed"
                                    let shown checks_print_report checks1
                                    checks_exit_code shown
                        Result::Err _e:
                            selfhost_type_constructor_table_free constructors
                            let checks1 checks_push checks0 Result::Err "type parameter env allocation failed"
                            let shown checks_print_report checks1
                            checks_exit_code shown
                Result::Err _e:
                    let checks1 checks_push checks0 Result::Err "constructor add failed"
                    let shown checks_print_report checks1
                    checks_exit_code shown
        Result::Err _e:
            let checks1 checks_push checks0 Result::Err "constructor table allocation failed"
            let shown checks_print_report checks1
            checks_exit_code shown
```
