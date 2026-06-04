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
