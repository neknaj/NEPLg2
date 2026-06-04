# NEPLg2 self-host type resolver input

## builds_flat_type_prefix_items_from_header_range

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
#import "neplg2/core/resolve/type_resolver" as *
#import "neplg2/core/syntax/lexer" as *
#import "neplg2/core/syntax/parser/module_parser" as *
#import "std/test" as *

fn check_item %fn &SelfhostTypePrefixList fn i32 fn SelfhostTypePrefixItemKind Result unit str \list\idx\expected:
    match selfhost_type_prefix_list_get list idx:
        Option::Some item:
            if:
                selfhost_type_prefix_item_kind_eq item.kind expected
                then:
                    Result::Ok unit
                else:
                    Result::Err "type prefix item kind mismatch"
        Option::None:
            Result::Err "type prefix item missing"

fn main %impure fn void i32 \void:
    let checks0 checks_new
    let source %str "fn add %fn i32 fn i32 i32 \\a\\b:\n    add a b\n"
    match lex_all source:
        Result::Ok tokens:
            let range %SelfhostSyntaxRange selfhost_parser_header_type_annotation_range &tokens v::len &tokens 0
            match selfhost_type_prefix_list_from_syntax_range &tokens range:
                Result::Ok list:
                    let checks1 checks_push checks0 check_eq_i32 5 selfhost_type_prefix_list_len &list
                    let checks2 checks_push checks1 check_item &list 0 SelfhostTypePrefixItemKind::FunctionMarker
                    let checks3 checks_push checks2 check_item &list 1 SelfhostTypePrefixItemKind::NamedType
                    let checks4 checks_push checks3 check_item &list 2 SelfhostTypePrefixItemKind::FunctionMarker
                    let checks5 checks_push checks4 check_item &list 3 SelfhostTypePrefixItemKind::NamedType
                    let checks6 checks_push checks5 check_item &list 4 SelfhostTypePrefixItemKind::NamedType
                    selfhost_type_prefix_list_free list
                    v::free tokens
                    let shown checks_print_report checks6
                    checks_exit_code shown
                Result::Err _e:
                    v::free tokens
                    let checks1 checks_push checks0 Result::Err "type prefix list build failed"
                    let shown checks_print_report checks1
                    checks_exit_code shown
        Result::Err _diag:
            let checks1 checks_push checks0 Result::Err "lex failed"
            let shown checks_print_report checks1
            checks_exit_code shown
```

## reduces_curried_function_type_with_nonempty_flattening

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
#import "neplg2/core/resolve/type_resolver" as *
#import "neplg2/core/syntax/lexer" as *
#import "neplg2/core/syntax/parser/module_parser" as *
#import "neplg2/core/ty/ty" as *
#import "std/test" as *

fn primitive_kind_eq %fn SelfhostPrimitiveTypeKind fn SelfhostPrimitiveTypeKind bool \a\b:
    match a:
        SelfhostPrimitiveTypeKind::Unit:
            match b:
                SelfhostPrimitiveTypeKind::Unit:
                    true
                _:
                    false
        SelfhostPrimitiveTypeKind::I32:
            match b:
                SelfhostPrimitiveTypeKind::I32:
                    true
                _:
                    false
        _:
            false

fn reduce_header %impure fn str Result SelfhostResolvedTypeTreeRoot str \source:
    match lex_all source:
        Result::Ok tokens:
            let range %SelfhostSyntaxRange selfhost_parser_header_type_annotation_range &tokens v::len &tokens 0
            match selfhost_type_prefix_list_from_syntax_range &tokens range:
                Result::Ok list:
                    match selfhost_type_prefix_list_reduce source &list:
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
            Result::Err "node missing"

fn check_node_primitive %fn &SelfhostResolvedTypeTree fn SelfhostResolvedTypeNodeId fn SelfhostPrimitiveTypeKind Result unit str \tree\node_id\expected:
    match selfhost_resolved_type_tree_primitive_kind tree node_id:
        Option::Some actual:
            if primitive_kind_eq actual expected Result::Ok unit Result::Err "primitive mismatch"
        Option::None:
            Result::Err "primitive missing"

fn check_function_arg_primitive %fn &SelfhostResolvedTypeTree fn SelfhostResolvedTypeNodeId fn i32 fn SelfhostPrimitiveTypeKind Result unit str \tree\fn_id\idx\expected:
    match selfhost_resolved_type_tree_function_arg tree fn_id idx:
        Option::Some arg_id:
            check_node_primitive tree arg_id expected
        Option::None:
            Result::Err "function arg missing"

fn check_function_result_primitive %fn &SelfhostResolvedTypeTree fn SelfhostResolvedTypeNodeId fn SelfhostPrimitiveTypeKind Result unit str \tree\fn_id\expected:
    match selfhost_resolved_type_tree_function_result tree fn_id:
        Option::Some result_id:
            check_node_primitive tree result_id expected
        Option::None:
            Result::Err "function result missing"

fn main %impure fn void i32 \void:
    let checks0 checks_new
    let source %str "fn add %fn i32 fn i32 i32 \\a\\b:\n    add a b\n"
    match reduce_header source:
        Result::Ok root:
            let tree %&SelfhostResolvedTypeTree selfhost_resolved_type_tree_root_tree &root
            let root_id %SelfhostResolvedTypeNodeId selfhost_resolved_type_tree_root_id &root
            let checks1 checks_push checks0 check_node_kind tree root_id SelfhostResolvedTypeNodeKind::Function
            let checks2 checks_push checks1 check_eq_i32 2 unwrap selfhost_resolved_type_tree_function_arg_count tree root_id
            let checks3 checks_push checks2 check_function_arg_primitive tree root_id 0 SelfhostPrimitiveTypeKind::I32
            let checks4 checks_push checks3 check_function_arg_primitive tree root_id 1 SelfhostPrimitiveTypeKind::I32
            let checks5 checks_push checks4 check_function_result_primitive tree root_id SelfhostPrimitiveTypeKind::I32
            selfhost_resolved_type_tree_root_free root
            let shown checks_print_report checks5
            checks_exit_code shown
        Result::Err e:
            let checks1 checks_push checks0 Result::Err e
            let shown checks_print_report checks1
            checks_exit_code shown
```

## keeps_void_function_returning_function_nested

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
#import "neplg2/core/resolve/type_resolver" as *
#import "neplg2/core/syntax/lexer" as *
#import "neplg2/core/syntax/parser/module_parser" as *
#import "neplg2/core/ty/ty" as *
#import "std/test" as *

fn primitive_kind_eq %fn SelfhostPrimitiveTypeKind fn SelfhostPrimitiveTypeKind bool \a\b:
    match a:
        SelfhostPrimitiveTypeKind::Unit:
            match b:
                SelfhostPrimitiveTypeKind::Unit:
                    true
                _:
                    false
        _:
            false

fn reduce_header %impure fn str Result SelfhostResolvedTypeTreeRoot str \source:
    match lex_all source:
        Result::Ok tokens:
            let range %SelfhostSyntaxRange selfhost_parser_header_type_annotation_range &tokens v::len &tokens 0
            match selfhost_type_prefix_list_from_syntax_range &tokens range:
                Result::Ok list:
                    match selfhost_type_prefix_list_reduce source &list:
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
            Result::Err "node missing"

fn check_node_primitive %fn &SelfhostResolvedTypeTree fn SelfhostResolvedTypeNodeId fn SelfhostPrimitiveTypeKind Result unit str \tree\node_id\expected:
    match selfhost_resolved_type_tree_primitive_kind tree node_id:
        Option::Some actual:
            if primitive_kind_eq actual expected Result::Ok unit Result::Err "primitive mismatch"
        Option::None:
            Result::Err "primitive missing"

fn check_function_arg_primitive %fn &SelfhostResolvedTypeTree fn SelfhostResolvedTypeNodeId fn i32 fn SelfhostPrimitiveTypeKind Result unit str \tree\fn_id\idx\expected:
    match selfhost_resolved_type_tree_function_arg tree fn_id idx:
        Option::Some arg_id:
            check_node_primitive tree arg_id expected
        Option::None:
            Result::Err "function arg missing"

fn main %impure fn void i32 \void:
    let checks0 checks_new
    let source %str "fn make %fn void fn unit unit \\void:\n    id_unit\n"
    match reduce_header source:
        Result::Ok root:
            let tree %&SelfhostResolvedTypeTree selfhost_resolved_type_tree_root_tree &root
            let root_id %SelfhostResolvedTypeNodeId selfhost_resolved_type_tree_root_id &root
            match selfhost_resolved_type_tree_function_result tree root_id:
                Option::Some inner_id:
                    let checks1 checks_push checks0 check_eq_i32 0 unwrap selfhost_resolved_type_tree_function_arg_count tree root_id
                    let checks2 checks_push checks1 check_node_kind tree inner_id SelfhostResolvedTypeNodeKind::Function
                    let checks3 checks_push checks2 check_eq_i32 1 unwrap selfhost_resolved_type_tree_function_arg_count tree inner_id
                    let checks4 checks_push checks3 check_function_arg_primitive tree inner_id 0 SelfhostPrimitiveTypeKind::Unit
                    let checks5 checks_push checks4 check_node_primitive tree unwrap selfhost_resolved_type_tree_function_result tree inner_id SelfhostPrimitiveTypeKind::Unit
                    let checks6 checks_push checks5 check_node_kind tree root_id SelfhostResolvedTypeNodeKind::Function
                    selfhost_resolved_type_tree_root_free root
                    let shown checks_print_report checks6
                    checks_exit_code shown
                Option::None:
                    selfhost_resolved_type_tree_root_free root
                    let checks1 checks_push checks0 Result::Err "outer result missing"
                    let shown checks_print_report checks1
                    checks_exit_code shown
        Result::Err e:
            let checks1 checks_push checks0 Result::Err e
            let shown checks_print_report checks1
            checks_exit_code shown
```

## rejects_void_as_return_type

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
#import "core/result" as *
#import "neplg2/core/resolve/type_resolver" as *
#import "neplg2/core/syntax/lexer" as *
#import "neplg2/core/syntax/parser/module_parser" as *
#import "std/test" as *

fn reduce_header_error_kind %impure fn str Result SelfhostTypeReduceErrorKind str \source:
    match lex_all source:
        Result::Ok tokens:
            let range %SelfhostSyntaxRange selfhost_parser_header_type_annotation_range &tokens v::len &tokens 0
            match selfhost_type_prefix_list_from_syntax_range &tokens range:
                Result::Ok list:
                    match selfhost_type_prefix_list_reduce source &list:
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
    let source %str "fn bad %fn i32 void \\a:\n    a\n"
    match reduce_header_error_kind source:
        Result::Ok kind:
            let checks1 checks_push checks0:
                if:
                    selfhost_type_reduce_error_kind_eq kind SelfhostTypeReduceErrorKind::VoidAsType
                    then:
                        Result::Ok unit
                    else:
                        Result::Err "unexpected reduce error kind"
            let shown checks_print_report checks1
            checks_exit_code shown
        Result::Err e:
            let checks1 checks_push checks0 Result::Err e
            let shown checks_print_report checks1
            checks_exit_code shown
```

## projects_curried_function_type_into_type_arena

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
#import "neplg2/core/resolve/type_resolver" as *
#import "neplg2/core/syntax/lexer" as *
#import "neplg2/core/syntax/parser/module_parser" as *
#import "neplg2/core/ty/ty" as *
#import "std/test" as *

fn reduce_header %impure fn str Result SelfhostResolvedTypeTreeRoot str \source:
    match lex_all source:
        Result::Ok tokens:
            let range %SelfhostSyntaxRange selfhost_parser_header_type_annotation_range &tokens v::len &tokens 0
            match selfhost_type_prefix_list_from_syntax_range &tokens range:
                Result::Ok list:
                    match selfhost_type_prefix_list_reduce source &list:
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

fn check_type_kind_option %fn &SelfhostTypeArena fn SelfhostTypeId fn SelfhostTypeKind Result unit str \arena\type_id\expected:
    match selfhost_type_arena_get_kind arena type_id:
        Option::Some actual:
            if selfhost_type_kind_eq actual expected Result::Ok unit Result::Err "type kind mismatch"
        Option::None:
            Result::Err "type kind missing"

fn check_function_arg_kind %fn &SelfhostTypeArena fn SelfhostTypeId fn i32 fn SelfhostTypeKind Result unit str \arena\fn_id\idx\expected:
    match selfhost_type_arena_function_arg arena fn_id idx:
        Option::Some arg_id:
            check_type_kind_option arena arg_id expected
        Option::None:
            Result::Err "function arg missing"

fn check_function_result_kind %fn &SelfhostTypeArena fn SelfhostTypeId fn SelfhostTypeKind Result unit str \arena\fn_id\expected:
    match selfhost_type_arena_function_result arena fn_id:
        Option::Some result_id:
            check_type_kind_option arena result_id expected
        Option::None:
            Result::Err "function result missing"

fn main %impure fn void i32 \void:
    let checks0 checks_new
    let source %str "fn add %fn i32 fn i32 i32 \\a\\b:\n    add a b\n"
    match reduce_header source:
        Result::Ok root:
            match selfhost_type_arena_new:
                Result::Ok arena0:
                    match selfhost_type_project_root_into_arena arena0 &root:
                        Result::Ok alloc:
                            let fn_id %SelfhostTypeId selfhost_type_arena_alloc_type_id &alloc
                            let arena1 %SelfhostTypeArena selfhost_type_arena_alloc_into_arena alloc
                            let checks1 checks_push checks0 check_type_kind_option &arena1 fn_id SelfhostTypeKind::Function
                            let checks2 checks_push checks1 check_eq_i32 2 unwrap selfhost_type_arena_function_arg_count &arena1 fn_id
                            let checks3 checks_push checks2 check_function_arg_kind &arena1 fn_id 0 SelfhostTypeKind::I32
                            let checks4 checks_push checks3 check_function_arg_kind &arena1 fn_id 1 SelfhostTypeKind::I32
                            let checks5 checks_push checks4 check_function_result_kind &arena1 fn_id SelfhostTypeKind::I32
                            let checks6 checks_push checks5 check_eq_i32 4 selfhost_type_arena_len &arena1
                            selfhost_type_arena_free arena1
                            selfhost_resolved_type_tree_root_free root
                            let shown checks_print_report checks6
                            checks_exit_code shown
                        Result::Err _e:
                            selfhost_resolved_type_tree_root_free root
                            let checks1 checks_push checks0 Result::Err "project failed"
                            let shown checks_print_report checks1
                            checks_exit_code shown
                Result::Err _e:
                    selfhost_resolved_type_tree_root_free root
                    let checks1 checks_push checks0 Result::Err "arena allocation failed"
                    let shown checks_print_report checks1
                    checks_exit_code shown
        Result::Err e:
            let checks1 checks_push checks0 Result::Err e
            let shown checks_print_report checks1
            checks_exit_code shown
```

## rejects_named_type_projection_until_constructor_lookup_exists

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
#import "core/result" as *
#import "neplg2/core/resolve/type_resolver" as *
#import "neplg2/core/syntax/lexer" as *
#import "neplg2/core/syntax/parser/module_parser" as *
#import "neplg2/core/ty/ty" as *
#import "std/test" as *

fn reduce_header %impure fn str Result SelfhostResolvedTypeTreeRoot str \source:
    match lex_all source:
        Result::Ok tokens:
            let range %SelfhostSyntaxRange selfhost_parser_header_type_annotation_range &tokens v::len &tokens 0
            match selfhost_type_prefix_list_from_syntax_range &tokens range:
                Result::Ok list:
                    match selfhost_type_prefix_list_reduce source &list:
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

fn main %impure fn void i32 \void:
    let checks0 checks_new
    let source %str "fn use_named %Foo \\x:\n    x\n"
    match reduce_header source:
        Result::Ok root:
            match selfhost_type_arena_new:
                Result::Ok arena0:
                    match selfhost_type_project_root_into_arena arena0 &root:
                        Result::Ok alloc:
                            let arena1 %SelfhostTypeArena selfhost_type_arena_alloc_into_arena alloc
                            selfhost_type_arena_free arena1
                            selfhost_resolved_type_tree_root_free root
                            let checks1 checks_push checks0 Result::Err "named projection unexpectedly succeeded"
                            let shown checks_print_report checks1
                            checks_exit_code shown
                        Result::Err e:
                            selfhost_resolved_type_tree_root_free root
                            let checks1 checks_push checks0:
                                if:
                                    selfhost_type_project_error_kind_eq e.kind SelfhostTypeProjectErrorKind::UnsupportedNamedType
                                    then:
                                        Result::Ok unit
                                    else:
                                        Result::Err "unexpected project error kind"
                            let shown checks_print_report checks1
                            checks_exit_code shown
                Result::Err _e:
                    selfhost_resolved_type_tree_root_free root
                    let checks1 checks_push checks0 Result::Err "arena allocation failed"
                    let shown checks_print_report checks1
                    checks_exit_code shown
        Result::Err e:
            let checks1 checks_push checks0 Result::Err e
            let shown checks_print_report checks1
            checks_exit_code shown
```

## projects_named_type_with_constructor_table

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: mlstr:
    ##: Checked [ok,ok,ok]
    ##: [0] ok
    ##: [1] ok
    ##: [2] ok
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

fn reduce_header %impure fn str Result SelfhostResolvedTypeTreeRoot str \source:
    match lex_all source:
        Result::Ok tokens:
            let range %SelfhostSyntaxRange selfhost_parser_header_type_annotation_range &tokens v::len &tokens 0
            match selfhost_type_prefix_list_from_syntax_range &tokens range:
                Result::Ok list:
                    match selfhost_type_prefix_list_reduce source &list:
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

fn check_type_kind_option %fn &SelfhostTypeArena fn SelfhostTypeId fn SelfhostTypeKind Result unit str \arena\type_id\expected:
    match selfhost_type_arena_get_kind arena type_id:
        Option::Some actual:
            if selfhost_type_kind_eq actual expected Result::Ok unit Result::Err "type kind mismatch"
        Option::None:
            Result::Err "type kind missing"

fn check_named_id %fn &SelfhostTypeArena fn SelfhostTypeId fn SelfhostNamedTypeId Result unit str \arena\type_id\expected:
    match selfhost_type_arena_named_id arena type_id:
        Option::Some actual:
            if selfhost_named_type_id_eq actual expected Result::Ok unit Result::Err "named id mismatch"
        Option::None:
            Result::Err "named id missing"

fn main %impure fn void i32 \void:
    let checks0 checks_new
    let source %str "fn use_named %Foo \\x:\n    x\n"
    match reduce_header source:
        Result::Ok root:
            match selfhost_type_constructor_table_new:
                Result::Ok constructors0:
                    match selfhost_type_constructor_table_add constructors0 "Foo" 0 source_span_empty_unchecked 0 0:
                        Result::Ok added:
                            let foo_id %SelfhostNamedTypeId selfhost_type_constructor_add_result_nominal_id &added
                            let constructors %SelfhostTypeConstructorTable selfhost_type_constructor_add_result_into_table added
                            match selfhost_type_arena_new:
                                Result::Ok arena0:
                                    match selfhost_type_project_root_with_constructors_into_arena arena0 &source &constructors &root:
                                        Result::Ok alloc:
                                            let type_id %SelfhostTypeId selfhost_type_arena_alloc_type_id &alloc
                                            let arena1 %SelfhostTypeArena selfhost_type_arena_alloc_into_arena alloc
                                            let checks1 checks_push checks0 check_type_kind_option &arena1 type_id SelfhostTypeKind::Named
                                            let checks2 checks_push checks1 check_named_id &arena1 type_id foo_id
                                            let checks3 checks_push checks2 check_eq_i32 1 selfhost_type_arena_len &arena1
                                            selfhost_type_arena_free arena1
                                            selfhost_type_constructor_table_free constructors
                                            selfhost_resolved_type_tree_root_free root
                                            let shown checks_print_report checks3
                                            checks_exit_code shown
                                        Result::Err _e:
                                            selfhost_type_constructor_table_free constructors
                                            selfhost_resolved_type_tree_root_free root
                                            let checks1 checks_push checks0 Result::Err "named projection failed"
                                            let shown checks_print_report checks1
                                            checks_exit_code shown
                                Result::Err _e:
                                    selfhost_type_constructor_table_free constructors
                                    selfhost_resolved_type_tree_root_free root
                                    let checks1 checks_push checks0 Result::Err "arena allocation failed"
                                    let shown checks_print_report checks1
                                    checks_exit_code shown
                        Result::Err _e:
                            selfhost_resolved_type_tree_root_free root
                            let checks1 checks_push checks0 Result::Err "constructor add failed"
                            let shown checks_print_report checks1
                            checks_exit_code shown
                Result::Err _e:
                    selfhost_resolved_type_tree_root_free root
                    let checks1 checks_push checks0 Result::Err "constructor table allocation failed"
                    let shown checks_print_report checks1
                    checks_exit_code shown
        Result::Err e:
            let checks1 checks_push checks0 Result::Err e
            let shown checks_print_report checks1
            checks_exit_code shown
```

## rejects_unknown_named_type_with_constructor_projection

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
#import "core/result" as *
#import "neplg2/core/resolve/type_resolver" as *
#import "neplg2/core/syntax/lexer" as *
#import "neplg2/core/syntax/parser/module_parser" as *
#import "neplg2/core/ty/ty" as *
#import "std/test" as *

fn reduce_header %impure fn str Result SelfhostResolvedTypeTreeRoot str \source:
    match lex_all source:
        Result::Ok tokens:
            let range %SelfhostSyntaxRange selfhost_parser_header_type_annotation_range &tokens v::len &tokens 0
            match selfhost_type_prefix_list_from_syntax_range &tokens range:
                Result::Ok list:
                    match selfhost_type_prefix_list_reduce source &list:
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

fn main %impure fn void i32 \void:
    let checks0 checks_new
    let source %str "fn use_named %Foo \\x:\n    x\n"
    match reduce_header source:
        Result::Ok root:
            match selfhost_type_constructor_table_new:
                Result::Ok constructors:
                    match selfhost_type_arena_new:
                        Result::Ok arena0:
                            match selfhost_type_project_root_with_constructors_into_arena arena0 &source &constructors &root:
                                Result::Ok alloc:
                                    let arena1 %SelfhostTypeArena selfhost_type_arena_alloc_into_arena alloc
                                    selfhost_type_arena_free arena1
                                    selfhost_type_constructor_table_free constructors
                                    selfhost_resolved_type_tree_root_free root
                                    let checks1 checks_push checks0 Result::Err "unknown named projection unexpectedly succeeded"
                                    let shown checks_print_report checks1
                                    checks_exit_code shown
                                Result::Err e:
                                    selfhost_type_constructor_table_free constructors
                                    selfhost_resolved_type_tree_root_free root
                                    let checks1 checks_push checks0:
                                        if:
                                            selfhost_type_project_error_kind_eq e.kind SelfhostTypeProjectErrorKind::UnknownNamedType
                                            then:
                                                Result::Ok unit
                                            else:
                                                Result::Err "unexpected project error kind"
                                    let shown checks_print_report checks1
                                    checks_exit_code shown
                        Result::Err _e:
                            selfhost_type_constructor_table_free constructors
                            selfhost_resolved_type_tree_root_free root
                            let checks1 checks_push checks0 Result::Err "arena allocation failed"
                            let shown checks_print_report checks1
                            checks_exit_code shown
                Result::Err _e:
                    selfhost_resolved_type_tree_root_free root
                    let checks1 checks_push checks0 Result::Err "constructor table allocation failed"
                    let shown checks_print_report checks1
                    checks_exit_code shown
        Result::Err e:
            let checks1 checks_push checks0 Result::Err e
            let shown checks_print_report checks1
            checks_exit_code shown
```

## rejects_bare_generic_constructor_projection

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
#import "core/result" as *
#import "neplg2/core/infra/span" as *
#import "neplg2/core/resolve/type_resolver" as *
#import "neplg2/core/syntax/lexer" as *
#import "neplg2/core/syntax/parser/module_parser" as *
#import "neplg2/core/ty/ty" as *
#import "std/test" as *

fn reduce_header %impure fn str Result SelfhostResolvedTypeTreeRoot str \source:
    match lex_all source:
        Result::Ok tokens:
            let range %SelfhostSyntaxRange selfhost_parser_header_type_annotation_range &tokens v::len &tokens 0
            match selfhost_type_prefix_list_from_syntax_range &tokens range:
                Result::Ok list:
                    match selfhost_type_prefix_list_reduce source &list:
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

fn main %impure fn void i32 \void:
    let checks0 checks_new
    let source %str "fn use_box %Box \\x:\n    x\n"
    match reduce_header source:
        Result::Ok root:
            match selfhost_type_constructor_table_new:
                Result::Ok constructors0:
                    match selfhost_type_constructor_table_add constructors0 "Box" 1 source_span_empty_unchecked 0 0:
                        Result::Ok added:
                            let constructors %SelfhostTypeConstructorTable selfhost_type_constructor_add_result_into_table added
                            match selfhost_type_arena_new:
                                Result::Ok arena0:
                                    match selfhost_type_project_root_with_constructors_into_arena arena0 &source &constructors &root:
                                        Result::Ok alloc:
                                            let arena1 %SelfhostTypeArena selfhost_type_arena_alloc_into_arena alloc
                                            selfhost_type_arena_free arena1
                                            selfhost_type_constructor_table_free constructors
                                            selfhost_resolved_type_tree_root_free root
                                            let checks1 checks_push checks0 Result::Err "bare generic constructor unexpectedly succeeded"
                                            let shown checks_print_report checks1
                                            checks_exit_code shown
                                        Result::Err e:
                                            selfhost_type_constructor_table_free constructors
                                            selfhost_resolved_type_tree_root_free root
                                            let checks1 checks_push checks0:
                                                if:
                                                    selfhost_type_project_error_kind_eq e.kind SelfhostTypeProjectErrorKind::GenericConstructorNeedsArguments
                                                    then:
                                                        Result::Ok unit
                                                    else:
                                                        Result::Err "unexpected project error kind"
                                            let shown checks_print_report checks1
                                            checks_exit_code shown
                                Result::Err _e:
                                    selfhost_type_constructor_table_free constructors
                                    selfhost_resolved_type_tree_root_free root
                                    let checks1 checks_push checks0 Result::Err "arena allocation failed"
                                    let shown checks_print_report checks1
                                    checks_exit_code shown
                        Result::Err _e:
                            selfhost_resolved_type_tree_root_free root
                            let checks1 checks_push checks0 Result::Err "constructor add failed"
                            let shown checks_print_report checks1
                            checks_exit_code shown
                Result::Err _e:
                    selfhost_resolved_type_tree_root_free root
                    let checks1 checks_push checks0 Result::Err "constructor table allocation failed"
                    let shown checks_print_report checks1
                    checks_exit_code shown
        Result::Err e:
            let checks1 checks_push checks0 Result::Err e
            let shown checks_print_report checks1
            checks_exit_code shown
```

## distinguishes_void_marker_from_unit_type

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
#import "neplg2/core/resolve/type_resolver" as *
#import "neplg2/core/syntax/lexer" as *
#import "neplg2/core/syntax/parser/module_parser" as *
#import "std/test" as *

fn check_item %fn &SelfhostTypePrefixList fn i32 fn SelfhostTypePrefixItemKind Result unit str \list\idx\expected:
    match selfhost_type_prefix_list_get list idx:
        Option::Some item:
            if:
                selfhost_type_prefix_item_kind_eq item.kind expected
                then:
                    Result::Ok unit
                else:
                    Result::Err "type prefix item kind mismatch"
        Option::None:
            Result::Err "type prefix item missing"

fn main %impure fn void i32 \void:
    let checks0 checks_new
    let source %str "fn main %fn void unit \\void:\n    unit\n"
    match lex_all source:
        Result::Ok tokens:
            let range %SelfhostSyntaxRange selfhost_parser_header_type_annotation_range &tokens v::len &tokens 0
            match selfhost_type_prefix_list_from_syntax_range &tokens range:
                Result::Ok list:
                    let checks1 checks_push checks0 check_eq_i32 3 selfhost_type_prefix_list_len &list
                    let checks2 checks_push checks1 check_item &list 0 SelfhostTypePrefixItemKind::FunctionMarker
                    let checks3 checks_push checks2 check_item &list 1 SelfhostTypePrefixItemKind::VoidMarker
                    let checks4 checks_push checks3 check_item &list 2 SelfhostTypePrefixItemKind::NamedType
                    selfhost_type_prefix_list_free list
                    v::free tokens
                    let shown checks_print_report checks4
                    checks_exit_code shown
                Result::Err _e:
                    v::free tokens
                    let checks1 checks_push checks0 Result::Err "type prefix list build failed"
                    let shown checks_print_report checks1
                    checks_exit_code shown
        Result::Err _diag:
            let checks1 checks_push checks0 Result::Err "lex failed"
            let shown checks_print_report checks1
            checks_exit_code shown
```
