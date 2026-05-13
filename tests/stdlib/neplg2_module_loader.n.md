# NEPLg2 self-host module loader

## loads_module_from_in_memory_vfs

neplg2:test
ret: 0
```neplg2
#entry main
#target std
#indent 4

#import "alloc/collections/vec" as *
#import "core/option" as *
#import "core/result" as *
#import "neplg2/core/module/loader" as *
#import "neplg2/core/syntax/ast/module_ast" as *
#import "std/test" as *
#import "core/math" as *

fn item_at <(&SelfhostModuleAst,i32)->SelfhostModuleItem> (ast, idx):
    unwrap<SelfhostModuleItem> selfhost_module_ast_get ast idx

fn main <()*>i32> ():
    let source_main <str> "fn main <()->i32> ():\n    0\n"
    let source_helper <str> "//: helper\nfn helper <()->i32> ():\n    1\n"
    let checks0 checks_new
    match selfhost_vfs_new:
        Result::Ok vfs0:
            match selfhost_vfs_add vfs0 "main.nepl" source_main:
                Result::Ok vfs1:
                    match selfhost_vfs_add vfs1 "helper.nepl" source_helper:
                        Result::Ok vfs2:
                            match selfhost_load_module &vfs2 "helper.nepl":
                                Result::Ok loaded:
                                    let ast_ref <&SelfhostModuleAst> selfhost_loaded_module_ast &loaded
                                    let item_len <i32> selfhost_module_ast_len ast_ref
                                    let item <SelfhostModuleItem> item_at ast_ref 1
                                    let kind_name <str> selfhost_module_item_kind_name item.kind
                                    let span_file_id <i32> item.span.file_id
                                    let path <str> selfhost_loaded_module_path &loaded
                                    let checks1 checks_push checks0 check_eq_i32 2 item_len
                                    let checks2 checks_push checks1 check_str_eq "helper.nepl" path
                                    let checks3 checks_push checks2 check_str_eq "FunctionDecl" kind_name
                                    let checks4 checks_push checks3 check_eq_i32 1 span_file_id
                                    let checks5 checks_push checks4 check_eq_i32 2 selfhost_vfs_len &vfs2
                                    selfhost_loaded_module_free loaded
                                    selfhost_vfs_free vfs2
                                    let shown checks_print_report checks5
                                    checks_exit_code shown
                                Result::Err _diag:
                                    selfhost_vfs_free vfs2
                                    let checks1 checks_push checks0 Result<(),str>::Err "loader returned Err"
                                    let shown checks_print_report checks1
                                    checks_exit_code shown
                        Result::Err _e:
                            let checks1 checks_push checks0 Result<(),str>::Err "second VFS add failed"
                            let shown checks_print_report checks1
                            checks_exit_code shown
                Result::Err _e:
                    let checks1 checks_push checks0 Result<(),str>::Err "first VFS add failed"
                    let shown checks_print_report checks1
                    checks_exit_code shown
        Result::Err _e:
            let checks1 checks_push checks0 Result<(),str>::Err "VFS allocation failed"
            let shown checks_print_report checks1
            checks_exit_code shown
```

## reports_missing_vfs_file

neplg2:test
ret: 0
```neplg2
#entry main
#target std
#indent 4

#import "alloc/collections/vec" as *
#import "core/option" as *
#import "core/result" as *
#import "neplg2/core/infra/diag" as *
#import "neplg2/core/module/loader" as *
#import "std/test" as *

fn check_missing_note <(TestReport, Option<str>)*>TestReport> (checks, note):
    match note:
        Option::Some text:
            checks_push checks check_str_eq "missing.nepl" text
        Option::None:
            checks_push checks Result<(),str>::Err "missing file diagnostic note was absent"

fn main <()*>i32> ():
    let checks0 checks_new
    match selfhost_vfs_new:
        Result::Ok vfs:
            match selfhost_load_module &vfs "missing.nepl":
                Result::Ok loaded:
                    selfhost_loaded_module_free loaded
                    selfhost_vfs_free vfs
                    let checks1 checks_push checks0 Result<(),str>::Err "missing file was loaded"
                    let shown checks_print_report checks1
                    checks_exit_code shown
                Result::Err diag:
                    let checks1 checks_push checks0 check_str_eq "loader.source.file_not_found" selfhost_diag_code_name diag.code
                    let checks2 check_missing_note checks1 diag.note
                    selfhost_vfs_free vfs
                    let shown checks_print_report checks2
                    checks_exit_code shown
        Result::Err _e:
            let checks1 checks_push checks0 Result<(),str>::Err "VFS allocation failed"
            let shown checks_print_report checks1
            checks_exit_code shown
```
