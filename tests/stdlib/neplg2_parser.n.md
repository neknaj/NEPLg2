# tests/stdlib/neplg2_parser.n.md

## parses_raw_backend_blocks_into_module_items

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: mlstr:
    ##: Checked [ok,ok,ok,ok,ok,ok,ok,ok,ok,ok,ok,ok,ok,ok,ok,ok,ok,ok,ok,ok,ok,ok]
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
    ##: [12] ok
    ##: [13] ok
    ##: [14] ok
    ##: [15] ok
    ##: [16] ok
    ##: [17] ok
    ##: [18] ok
    ##: [19] ok
    ##: [20] ok
    ##: [21] ok
```neplg2
#entry main
#target std
#indent 4

#import "alloc/collections/vec" as *
#import "core/option" as *
#import "core/result" as *
#import "neplg2/core/syntax/ast/module_ast" as *
#import "neplg2/core/syntax/parser/module_parser" as *
#import "std/test" as *
#import "core/field" as *
#import "core/math" as *

fn item_at %fn &SelfhostModuleAst fn i32 SelfhostModuleItem \ast\idx:
    let item_opt %Option SelfhostModuleItem selfhost_module_ast_get ast idx
    unwrap item_opt

fn check_item %impure fn TestReport impure fn &SelfhostModuleAst impure fn i32 impure fn str impure fn str TestReport \checks\ast\idx\expected_kind\expected_lexeme:
    let item %SelfhostModuleItem item_at ast idx
    let kind_name %str selfhost_module_item_kind_name item.kind
    let lexeme %str item.lexeme
    let checks1 checks_push checks check_str_eq expected_kind kind_name
    checks_push checks1 check_str_eq expected_lexeme lexeme

fn check_function_declaration_header %fn SelfhostModuleItem Result unit str \item:
    match item.declaration:
        Option::Some header:
            match header.kind:
                SelfhostModuleDeclarationKind::Function:
                    match header.visibility:
                        SelfhostModuleDeclarationVisibility::Private:
                            match header.head:
                                Option::Some head:
                                    match head.kind:
                                        SelfhostModuleDeclarationHeadKind::Name:
                                            Result::Ok unit
                                        SelfhostModuleDeclarationHeadKind::TypeLabel:
                                            Result::Err "expected function name head"
                                        SelfhostModuleDeclarationHeadKind::GenericParams:
                                            Result::Err "expected function name head"
                                Option::None:
                                    Result::Err "expected declaration head"
                        SelfhostModuleDeclarationVisibility::Public:
                            Result::Err "expected private declaration"
                SelfhostModuleDeclarationKind::Struct:
                    Result::Err "expected function declaration"
                SelfhostModuleDeclarationKind::Enum:
                    Result::Err "expected function declaration"
                SelfhostModuleDeclarationKind::Trait:
                    Result::Err "expected function declaration"
                SelfhostModuleDeclarationKind::Impl:
                    Result::Err "expected function declaration"
        Option::None:
            Result::Err "expected parser declaration header evidence"

fn main %impure fn unit i32 \unit:
    let source %str "//: doc\nfn add <(i32,i32)->i32> (a,b):\n    #if[target=wasm]\n    #wasm:\n        local.get 0\n        local.get 1\n    #if[target=llvm]\n    #llvmir:\n        %0 = add i32 %a, %b\n        ret i32 %0\n"
    let checks0 checks_new
    match selfhost_parse_module_source source:
        Result::Ok ast:
            let item_len %i32 selfhost_module_ast_len &ast
            let checks1 checks_push checks0 check_eq_i32 10 item_len
            let checks2 check_item checks1 &ast 0 "DocComment" "//: doc"
            let checks3 check_item checks2 &ast 1 "FunctionDecl" "fn add <(i32,i32)->i32> (a,b):"
            let checks4 check_item checks3 &ast 2 "IfTargetDirective" "#if[target=wasm]"
            let checks5 check_item checks4 &ast 3 "WasmBlock" "#wasm:"
            let checks6 check_item checks5 &ast 4 "WasmText" "local.get 0"
            let checks7 check_item checks6 &ast 5 "WasmText" "local.get 1"
            let checks8 check_item checks7 &ast 6 "IfTargetDirective" "#if[target=llvm]"
            let checks9 check_item checks8 &ast 7 "LlvmIrBlock" "#llvmir:"
            let checks10 check_item checks9 &ast 8 "LlvmIrText" "%0 = add i32 %a, %b"
            let checks11 check_item checks10 &ast 9 "LlvmIrText" "ret i32 %0"
            let checks12 checks_push checks11 check_function_declaration_header item_at &ast 1
            selfhost_module_ast_free ast
            let shown checks_print_report checks12
            checks_exit_code shown
        Result::Err diag:
            let _msg %str diag.message
            let checks1 checks_push checks0 Result::Err "parser returned Err"
            let shown checks_print_report checks1
            checks_exit_code shown
```
