# tests/stdlib/neplg2_parser.n.md

## parses_raw_backend_blocks_into_module_items

neplg2:test
ret: 0
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

fn item_at <(&SelfhostModuleAst,i32)->SelfhostModuleItem> (ast, idx):
    unwrap<SelfhostModuleItem> selfhost_module_ast_get ast idx

fn check_item <(Vec<Result<(),str>>, &SelfhostModuleAst, i32, str, str)*>Vec<Result<(),str>>> (checks, ast, idx, expected_kind, expected_lexeme):
    let item <SelfhostModuleItem> item_at ast idx
    let kind_name <str> selfhost_module_item_kind_name item.kind
    let lexeme <str> item.lexeme
    let checks1 <Vec<Result<(),str>>> checks_push checks check_str_eq expected_kind kind_name
    checks_push checks1 check_str_eq expected_lexeme lexeme

fn main <()*>i32> ():
    let source <str> "//: doc\nfn add <(i32,i32)->i32> (a,b):\n    #if[target=wasm]\n    #wasm:\n        local.get 0\n        local.get 1\n    #if[target=llvm]\n    #llvmir:\n        %0 = add i32 %a, %b\n        ret i32 %0\n"
    let checks0 <Vec<Result<(),str>>> checks_new
    match selfhost_parse_module_source source:
        Result::Ok ast:
            let item_len <i32> selfhost_module_ast_len &ast
            let checks1 <Vec<Result<(),str>>> checks_push checks0 check_eq_i32 10 item_len
            let checks2 <Vec<Result<(),str>>> check_item checks1 &ast 0 "DocComment" "//: doc"
            let checks3 <Vec<Result<(),str>>> check_item checks2 &ast 1 "FunctionDecl" "fn"
            let checks4 <Vec<Result<(),str>>> check_item checks3 &ast 2 "IfTargetDirective" "#if[target=wasm]"
            let checks5 <Vec<Result<(),str>>> check_item checks4 &ast 3 "WasmBlock" "#wasm:"
            let checks6 <Vec<Result<(),str>>> check_item checks5 &ast 4 "WasmText" "local.get 0"
            let checks7 <Vec<Result<(),str>>> check_item checks6 &ast 5 "WasmText" "local.get 1"
            let checks8 <Vec<Result<(),str>>> check_item checks7 &ast 6 "IfTargetDirective" "#if[target=llvm]"
            let checks9 <Vec<Result<(),str>>> check_item checks8 &ast 7 "LlvmIrBlock" "#llvmir:"
            let checks10 <Vec<Result<(),str>>> check_item checks9 &ast 8 "LlvmIrText" "%0 = add i32 %a, %b"
            let checks11 <Vec<Result<(),str>>> check_item checks10 &ast 9 "LlvmIrText" "ret i32 %0"
            selfhost_module_ast_free ast
            let shown <Vec<Result<(),str>>> checks_print_report checks11
            checks_exit_code shown
        Result::Err diag:
            let _msg <str> diag.message
            let checks1 <Vec<Result<(),str>>> checks_push checks0 Result<(),str>::Err "parser returned Err"
            let shown <Vec<Result<(),str>>> checks_print_report checks1
            checks_exit_code shown
```
