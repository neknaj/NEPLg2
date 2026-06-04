# tests/stdlib/neplg2_name_resolver.n.md

## hoists_declaration_names_from_module_ast

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: mlstr:
    ##: Checked [ok,ok,ok,ok,ok,ok,ok,ok,ok,ok,ok,ok]
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
    ##: [11] ok
```neplg2
#entry main
#target std
#indent 4

#import "core/math" as *
#import "core/option" as *
#import "core/result" as *
#import "neplg2/core/resolve/name_resolver" as *
#import "neplg2/core/syntax/ast/module_ast" as *
#import "neplg2/core/syntax/parser/module_parser" as *
#import "std/test" as *

fn check_binding %fn &SelfhostNameScope fn str fn SelfhostDefKind fn i32 Result unit str \scope\name\kind\expected_index:
    match selfhost_name_scope_find_kind scope name kind:
        Option::Some binding:
            match binding.def_id:
                Option::Some def_id:
                    if:
                        and selfhost_def_kind_eq binding.kind kind eq selfhost_def_id_index def_id expected_index
                        then:
                            Result::Ok unit
                        else:
                            Result::Err "binding kind or def id mismatch"
                Option::None:
                    Result::Err "binding def id was not assigned"
        Option::None:
            Result::Err "binding was not found"

fn check_item_kind %fn &SelfhostModuleAst fn i32 fn str Result unit str \ast\idx\expected:
    match selfhost_module_ast_get ast idx:
        Option::Some item:
            check_str_eq expected selfhost_module_item_kind_name item.kind
        Option::None:
            Result::Err "module item was not found"

fn main %impure fn void i32 \void:
    let source %str "fn main %fn void unit \\void:\n    unit\nstruct Item:\n    field %i32\nenum Choice:\n    A\ntrait Show:\n    fn show %fn i32 i32\nimpl .T:\n    fn show %fn i32 i32\n"
    let checks0 checks_new
    match selfhost_parse_module_source source:
        Result::Ok ast:
            match selfhost_name_scope_hoist_module_declarations source &ast:
                Result::Ok scope:
                    let checks1 checks_push checks0 check_eq_i32 5 selfhost_module_ast_len &ast
                    let checks2 checks_push checks1 check_item_kind &ast 0 "FunctionDecl"
                    let checks3 checks_push checks2 check_item_kind &ast 1 "StructDecl"
                    let checks4 checks_push checks3 check_item_kind &ast 2 "EnumDecl"
                    let checks5 checks_push checks4 check_item_kind &ast 3 "TraitDecl"
                    let checks6 checks_push checks5 check_item_kind &ast 4 "ImplDecl"
                    let checks7 checks_push checks6 check_eq_i32 4 selfhost_name_scope_len &scope
                    let checks8 checks_push checks7 check_binding &scope "main" SelfhostDefKind::Function 0
                    let checks9 checks_push checks8 check_binding &scope "Item" SelfhostDefKind::Struct 1
                    let checks10 checks_push checks9 check_binding &scope "Choice" SelfhostDefKind::Enum 2
                    let checks11 checks_push checks10 check_binding &scope "Show" SelfhostDefKind::Trait 3
                    let checks12 checks_push checks11 check selfhost_def_kind_eq SelfhostDefKind::Impl SelfhostDefKind::Impl
                    selfhost_name_scope_free scope
                    selfhost_module_ast_free ast
                    let shown checks_print_report checks12
                    checks_exit_code shown
                Result::Err _e:
                    selfhost_module_ast_free ast
                    let checks1 checks_push checks0 Result::Err "hoist returned Err"
                    let shown checks_print_report checks1
                    checks_exit_code shown
        Result::Err _diag:
            let checks1 checks_push checks0 Result::Err "parser returned Err"
            let shown checks_print_report checks1
            checks_exit_code shown
```
