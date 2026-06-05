# NEPLg2 self-host type constructor validation

## constructor_table_add_checked_validates_kind_boundary

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: mlstr:
    ##: Checked [ok,ok,ok,ok,ok,ok,ok]
    ##: [0] ok
    ##: [1] ok
    ##: [2] ok
    ##: [3] ok
    ##: [4] ok
    ##: [5] ok
    ##: [6] ok
```neplg2
#entry main
#target std
#indent 4

#import "core/math" as *
#import "core/option" as *
#import "core/result" as *
#import "neplg2/core/infra/span" as *
#import "neplg2/core/resolve/type_resolver" as *
#import "std/test" as *

fn check_constructor_kind %fn &SelfhostTypeConstructorTable fn str fn SelfhostTypeConstructorKind Result unit str \table\name\expected:
    match selfhost_type_constructor_table_find table name:
        Option::Some constructor:
            if:
                selfhost_type_constructor_kind_eq constructor.kind expected
                then:
                    Result::Ok unit
                else:
                    Result::Err "constructor kind mismatch"
        Option::None:
            Result::Err "constructor missing"

fn check_add_error_kind %impure fn Result SelfhostTypeConstructorAddResult SelfhostTypeConstructorTableError impure fn SelfhostTypeConstructorTableErrorKind Result unit str \result\expected:
    match result:
        Result::Ok added:
            selfhost_type_constructor_table_free selfhost_type_constructor_add_result_into_table added
            Result::Err "constructor add unexpectedly succeeded"
        Result::Err e:
            if:
                selfhost_type_constructor_table_error_kind_eq e.kind expected
                then:
                    Result::Ok unit
                else:
                    Result::Err "constructor add error kind mismatch"

fn check_negative_arity %impure fn void Result unit str \void:
    match selfhost_type_constructor_table_new:
        Result::Ok table:
            let bad_arity %i32 sub 0 1
            let span %SelfhostSourceSpan source_span_empty_unchecked 0 0
            let result %Result SelfhostTypeConstructorAddResult SelfhostTypeConstructorTableError selfhost_type_constructor_table_add_checked table "Bad" bad_arity span
            check_add_error_kind result SelfhostTypeConstructorTableErrorKind::NegativeConstructorArity
        Result::Err _e:
            Result::Err "constructor table allocation failed"

fn check_reserved_name %impure fn void Result unit str \void:
    match selfhost_type_constructor_table_new:
        Result::Ok table:
            let span %SelfhostSourceSpan source_span_empty_unchecked 0 0
            let result %Result SelfhostTypeConstructorAddResult SelfhostTypeConstructorTableError selfhost_type_constructor_table_add_checked table "unit" 0 span
            check_add_error_kind result SelfhostTypeConstructorTableErrorKind::ReservedTypeConstructorName
        Result::Err _e:
            Result::Err "constructor table allocation failed"

fn check_duplicate_name %impure fn void Result unit str \void:
    match selfhost_type_constructor_table_new:
        Result::Ok table0:
            let span0 %SelfhostSourceSpan source_span_empty_unchecked 0 0
            let first_result %Result SelfhostTypeConstructorAddResult SelfhostTypeConstructorTableError selfhost_type_constructor_table_add_checked table0 "Foo" 0 span0
            match first_result:
                Result::Ok added:
                    let table1 %SelfhostTypeConstructorTable selfhost_type_constructor_add_result_into_table added
                    let span1 %SelfhostSourceSpan source_span_empty_unchecked 0 0
                    let result %Result SelfhostTypeConstructorAddResult SelfhostTypeConstructorTableError selfhost_type_constructor_table_add_checked table1 "Foo" 1 span1
                    check_add_error_kind result SelfhostTypeConstructorTableErrorKind::DuplicateTypeConstructor
                Result::Err _e:
                    Result::Err "first constructor add failed"
        Result::Err _e:
            Result::Err "constructor table allocation failed"

fn main %impure fn void i32 \void:
    let checks0 checks_new
    match selfhost_type_constructor_table_new:
        Result::Ok table0:
            let span0 %SelfhostSourceSpan source_span_empty_unchecked 0 0
            let first_result %Result SelfhostTypeConstructorAddResult SelfhostTypeConstructorTableError selfhost_type_constructor_table_add_checked table0 "Foo" 0 span0
            match first_result:
                Result::Ok added0:
                    let table1 %SelfhostTypeConstructorTable selfhost_type_constructor_add_result_into_table added0
                    let span1 %SelfhostSourceSpan source_span_empty_unchecked 0 0
                    let second_result %Result SelfhostTypeConstructorAddResult SelfhostTypeConstructorTableError selfhost_type_constructor_table_add_checked table1 "Box" 1 span1
                    match second_result:
                        Result::Ok added1:
                            let table2 %SelfhostTypeConstructorTable selfhost_type_constructor_add_result_into_table added1
                            let checks1 checks_push checks0 check_eq_i32 2 selfhost_type_constructor_table_len &table2
                            let checks2 checks_push checks1 check_constructor_kind &table2 "Foo" SelfhostTypeConstructorKind::Type
                            let checks3 checks_push checks2 check_constructor_kind &table2 "Box" SelfhostTypeConstructorKind::TypeConstructor 1
                            let checks4 checks_push checks3 check_negative_arity
                            let checks5 checks_push checks4 check_reserved_name
                            let checks6 checks_push checks5 check_duplicate_name
                            let box_kind_arity %i32 selfhost_type_constructor_kind_arg_count SelfhostTypeConstructorKind::TypeConstructor 1
                            let checks7 checks_push checks6 check_eq_i32 1 box_kind_arity
                            selfhost_type_constructor_table_free table2
                            let shown checks_print_report checks7
                            checks_exit_code shown
                        Result::Err _e:
                            let checks1 checks_push checks0 Result::Err "Box constructor add failed"
                            let shown checks_print_report checks1
                            checks_exit_code shown
                Result::Err _e:
                    let checks1 checks_push checks0 Result::Err "Foo constructor add failed"
                    let shown checks_print_report checks1
                    checks_exit_code shown
        Result::Err _e:
            let checks1 checks_push checks0 Result::Err "constructor table allocation failed"
            let shown checks_print_report checks1
            checks_exit_code shown
```

## projection_rejects_malformed_applied_constructor_arity

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
#import "neplg2/core/ty/ty" as *
#import "std/test" as *

fn push_node_id %impure fn Vec SelfhostResolvedTypeNodeId impure fn SelfhostResolvedTypeNodeId Result Vec SelfhostResolvedTypeNodeId str \items\node_id:
    match v::push items node_id:
        Result::Ok next_items:
            Result::Ok next_items
        Result::Err e:
            v::free v::vec_push_error_vec e
            Result::Err "type argument push failed"

fn build_result_i32_wrong_arity_root %impure fn SelfhostNamedTypeId Result SelfhostResolvedTypeTreeRoot str \result_id:
    match selfhost_resolved_type_tree_new:
        Result::Ok tree0:
            match selfhost_resolved_type_tree_add_primitive tree0 SelfhostPrimitiveTypeKind::I32:
                Result::Ok i32_added:
                    let i32_node %SelfhostResolvedTypeNodeId selfhost_resolved_type_tree_alloc_node_id &i32_added
                    let tree1 %SelfhostResolvedTypeTree selfhost_resolved_type_tree_alloc_into_tree i32_added
                    let args_result %Result Vec SelfhostResolvedTypeNodeId StdErrorKind v::new
                    match args_result:
                        Result::Ok args0:
                            match push_node_id args0 i32_node:
                                Result::Ok args1:
                                    let span %SelfhostSourceSpan source_span_empty_unchecked 0 0
                                    match selfhost_resolved_type_tree_add_applied_named tree1 result_id span args1:
                                        Result::Ok applied_added:
                                            let root_node %SelfhostResolvedTypeNodeId selfhost_resolved_type_tree_alloc_node_id &applied_added
                                            let tree2 %SelfhostResolvedTypeTree selfhost_resolved_type_tree_alloc_into_tree applied_added
                                            Result::Ok selfhost_resolved_type_tree_root_new tree2 root_node
                                        Result::Err _e:
                                            Result::Err "applied node allocation failed"
                                Result::Err e:
                                    selfhost_resolved_type_tree_free tree1
                                    Result::Err e
                        Result::Err _e:
                            selfhost_resolved_type_tree_free tree1
                            Result::Err "type argument vector allocation failed"
                Result::Err _e:
                    Result::Err "primitive node allocation failed"
        Result::Err _e:
            Result::Err "resolved tree allocation failed"

fn project_wrong_arity_error_kind %impure fn &SelfhostTypeConstructorTable impure fn &SelfhostResolvedTypeTreeRoot Result SelfhostTypeProjectErrorKind str \constructors\root:
    match selfhost_type_arena_new:
        Result::Ok arena0:
            let source %str ""
            match selfhost_type_project_root_with_constructors_into_arena arena0 &source constructors root:
                Result::Ok projected:
                    selfhost_type_arena_free selfhost_type_arena_alloc_into_arena projected
                    Result::Err "projection unexpectedly succeeded"
                Result::Err e:
                    Result::Ok e.kind
        Result::Err _e:
            Result::Err "type arena allocation failed"

fn check_project_error_kind %fn Result SelfhostTypeProjectErrorKind str fn SelfhostTypeProjectErrorKind Result unit str \result\expected:
    match result:
        Result::Ok actual:
            if selfhost_type_project_error_kind_eq actual expected Result::Ok unit Result::Err "wrong project error kind"
        Result::Err e:
            Result::Err e

fn main %impure fn void i32 \void:
    let checks0 checks_new
    match selfhost_type_constructor_table_new:
        Result::Ok constructors0:
            let span %SelfhostSourceSpan source_span_empty_unchecked 0 0
            match selfhost_type_constructor_table_add_checked constructors0 "Result" 2 span:
                Result::Ok added:
                    let result_id %SelfhostNamedTypeId selfhost_type_constructor_add_result_nominal_id &added
                    let constructors %SelfhostTypeConstructorTable selfhost_type_constructor_add_result_into_table added
                    match build_result_i32_wrong_arity_root result_id:
                        Result::Ok root:
                            let actual %Result SelfhostTypeProjectErrorKind str project_wrong_arity_error_kind &constructors &root
                            let checks1 checks_push checks0 check_project_error_kind actual SelfhostTypeProjectErrorKind::GenericConstructorArgumentArityMismatch
                            selfhost_resolved_type_tree_root_free root
                            selfhost_type_constructor_table_free constructors
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

## two_arity_constructor_reduction_uses_bound_plan

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: mlstr:
    ##: Checked [ok,ok,ok,ok,ok,ok,ok]
    ##: [0] ok
    ##: [1] ok
    ##: [2] ok
    ##: [3] ok
    ##: [4] ok
    ##: [5] ok
    ##: [6] ok
```neplg2
#entry main
#target std
#indent 4

#import "alloc/collections/vec" as v
#import "core/option" as *
#import "core/result" as *
#import "neplg2/core/infra/span" as *
#import "neplg2/core/resolve/type_resolver" as *
#import "neplg2/core/ty/ty/id" as *
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

fn build_result_i32_str_list %impure fn void Result SelfhostTypePrefixList str \void:
    let items_result %Result Vec SelfhostTypePrefixItem StdErrorKind v::new
    match items_result:
        Result::Ok items0:
            match push_named_item items0 0 0 6:
                Result::Ok items1:
                    match push_named_item items1 1 7 10:
                        Result::Ok items2:
                            match push_named_item items2 2 11 14:
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

fn build_result_i32_list %impure fn void Result SelfhostTypePrefixList str \void:
    let items_result %Result Vec SelfhostTypePrefixItem StdErrorKind v::new
    match items_result:
        Result::Ok items0:
            match push_named_item items0 0 0 6:
                Result::Ok items1:
                    match push_named_item items1 1 7 10:
                        Result::Ok items2:
                            Result::Ok selfhost_type_prefix_list_new items2
                        Result::Err e:
                            Result::Err e
                Result::Err e:
                    Result::Err e
        Result::Err _e:
            Result::Err "prefix item allocation failed"

fn build_result_i32_str_unit_list %impure fn void Result SelfhostTypePrefixList str \void:
    let items_result %Result Vec SelfhostTypePrefixItem StdErrorKind v::new
    match items_result:
        Result::Ok items0:
            match push_named_item items0 0 0 6:
                Result::Ok items1:
                    match push_named_item items1 1 7 10:
                        Result::Ok items2:
                            match push_named_item items2 2 11 14:
                                Result::Ok items3:
                                    match push_named_item items3 3 15 19:
                                        Result::Ok items4:
                                            Result::Ok selfhost_type_prefix_list_new items4
                                        Result::Err e:
                                            Result::Err e
                                Result::Err e:
                                    Result::Err e
                        Result::Err e:
                            Result::Err e
                Result::Err e:
                    Result::Err e
        Result::Err _e:
            Result::Err "prefix item allocation failed"

fn reduce_list_with_constructors %impure fn str impure fn SelfhostTypePrefixList impure fn &SelfhostTypeConstructorTable Result SelfhostResolvedTypeTreeRoot str \source\list\constructors:
    match selfhost_type_prefix_list_reduce_with_constructors source constructors &list:
        Result::Ok root:
            selfhost_type_prefix_list_free list
            Result::Ok root
        Result::Err _e:
            selfhost_type_prefix_list_free list
            Result::Err "reduce failed"

fn reduce_list_error_kind_with_constructors %impure fn str impure fn SelfhostTypePrefixList impure fn &SelfhostTypeConstructorTable Result SelfhostTypeReduceErrorKind str \source\list\constructors:
    match selfhost_type_prefix_list_reduce_with_constructors source constructors &list:
        Result::Ok root:
            selfhost_resolved_type_tree_root_free root
            selfhost_type_prefix_list_free list
            Result::Err "reduction unexpectedly succeeded"
        Result::Err e:
            selfhost_type_prefix_list_free list
            Result::Ok e.kind

fn check_node_kind %fn &SelfhostResolvedTypeTree fn SelfhostResolvedTypeNodeId fn SelfhostResolvedTypeNodeKind Result unit str \tree\node_id\expected:
    match selfhost_resolved_type_tree_get_node_kind tree node_id:
        Option::Some actual:
            if selfhost_resolved_type_node_kind_eq actual expected Result::Ok unit Result::Err "node kind mismatch"
        Option::None:
            Result::Err "node kind missing"

fn check_named_id_option %fn Option SelfhostNamedTypeId fn SelfhostNamedTypeId Result unit str \actual\expected:
    match actual:
        Option::Some nominal_id:
            if selfhost_named_type_id_eq nominal_id expected Result::Ok unit Result::Err "named id mismatch"
        Option::None:
            Result::Err "named id missing"

fn check_i32_option %fn Option i32 fn i32 Result unit str \actual\expected:
    match actual:
        Option::Some value:
            check_eq_i32 expected value
        Option::None:
            Result::Err "i32 option missing"

fn check_applied_arg_kind %fn &SelfhostResolvedTypeTree fn SelfhostResolvedTypeNodeId fn i32 fn SelfhostResolvedTypeNodeKind Result unit str \tree\root_id\idx\expected:
    match selfhost_resolved_type_tree_applied_arg tree root_id idx:
        Option::Some arg_id:
            check_node_kind tree arg_id expected
        Option::None:
            Result::Err "applied argument missing"

fn check_error_kind %fn Result SelfhostTypeReduceErrorKind str fn SelfhostTypeReduceErrorKind Result unit str \result\expected:
    match result:
        Result::Ok actual:
            if selfhost_type_reduce_error_kind_eq actual expected Result::Ok unit Result::Err "unexpected error kind"
        Result::Err e:
            Result::Err e

fn main %impure fn void i32 \void:
    let checks0 checks_new
    match selfhost_type_constructor_table_new:
        Result::Ok constructors0:
            let span %SelfhostSourceSpan source_span_empty_unchecked 0 0
            match selfhost_type_constructor_table_add_checked constructors0 "Result" 2 span:
                Result::Ok added:
                    let result_id %SelfhostNamedTypeId selfhost_type_constructor_add_result_nominal_id &added
                    let constructors %SelfhostTypeConstructorTable selfhost_type_constructor_add_result_into_table added
                    match build_result_i32_str_list:
                        Result::Ok list:
                            match reduce_list_with_constructors "Result i32 str" list &constructors:
                                Result::Ok root:
                                    let tree %&SelfhostResolvedTypeTree selfhost_resolved_type_tree_root_tree &root
                                    let root_id %SelfhostResolvedTypeNodeId selfhost_resolved_type_tree_root_id &root
                                    let checks1 checks_push checks0 check_node_kind tree root_id SelfhostResolvedTypeNodeKind::Applied
                                    let checks2 checks_push checks1 check_named_id_option (selfhost_resolved_type_tree_applied_constructor_id tree root_id) result_id
                                    let checks3 checks_push checks2 check_i32_option (selfhost_resolved_type_tree_applied_arg_count tree root_id) 2
                                    let checks4 checks_push checks3 check_applied_arg_kind tree root_id 0 SelfhostResolvedTypeNodeKind::Primitive
                                    let checks5 checks_push checks4 check_applied_arg_kind tree root_id 1 SelfhostResolvedTypeNodeKind::Primitive
                                    selfhost_resolved_type_tree_root_free root
                                    match build_result_i32_list:
                                        Result::Ok missing_list:
                                            let missing %Result SelfhostTypeReduceErrorKind str reduce_list_error_kind_with_constructors "Result i32" missing_list &constructors
                                            let checks6 checks_push checks5 check_error_kind missing SelfhostTypeReduceErrorKind::GenericTypeArgumentMissing
                                            match build_result_i32_str_unit_list:
                                                Result::Ok trailing_list:
                                                    let trailing %Result SelfhostTypeReduceErrorKind str reduce_list_error_kind_with_constructors "Result i32 str unit" trailing_list &constructors
                                                    let checks7 checks_push checks6 check_error_kind trailing SelfhostTypeReduceErrorKind::TrailingItems
                                                    selfhost_type_constructor_table_free constructors
                                                    let shown checks_print_report checks7
                                                    checks_exit_code shown
                                                Result::Err e:
                                                    selfhost_type_constructor_table_free constructors
                                                    let checks7 checks_push checks6 Result::Err e
                                                    let shown checks_print_report checks7
                                                    checks_exit_code shown
                                        Result::Err e:
                                            selfhost_type_constructor_table_free constructors
                                            let checks6 checks_push checks5 Result::Err e
                                            let shown checks_print_report checks6
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
                    let checks1 checks_push checks0 Result::Err "Result constructor add failed"
                    let shown checks_print_report checks1
                    checks_exit_code shown
        Result::Err _e:
            let checks1 checks_push checks0 Result::Err "constructor table allocation failed"
            let shown checks_print_report checks1
            checks_exit_code shown
```
