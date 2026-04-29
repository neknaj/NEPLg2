# NEPLg2 self-host stdlib map

## resolves_stdlib_and_relative_paths

neplg2:test
ret: 0
```neplg2
#entry main
#target std
#indent 4

#import "alloc/collections/vec" as *
#import "core/result" as *
#import "neplg2/core/infra/span" as *
#import "neplg2/core/infra/diag" as *
#import "neplg2/core/module/stdlib_map" as *
#import "std/test" as *

fn check_path_result <(TestReport,Result<SelfhostResolvedModulePath,SelfhostDiagnostic>,str,bool)*>TestReport> (checks, result, expected, expect_stdlib):
    match result:
        Result::Ok resolved:
            let checks1 checks_push checks check_str_eq expected resolved.path
            if:
                expect_stdlib
                then:
                    checks_push checks1 check selfhost_resolved_module_path_is_stdlib resolved
                else:
                    checks_push checks1 check not selfhost_resolved_module_path_is_stdlib resolved
        Result::Err _diag:
            checks_push checks Result<(),str>::Err "path resolution returned Err"

fn main <()*>i32> ():
    let map <SelfhostModulePathMap> selfhost_module_path_map_new "user" "stdlib"
    let span <SelfhostSourceSpan> source_span_empty 0 0
    let checks0 checks_new
    let checks1 check_path_result checks0 (selfhost_module_path_resolve_import &map "user/app/main.nepl" span "core/result") "stdlib/core/result.nepl" true
    let checks2 check_path_result checks1 (selfhost_module_path_resolve_import &map "user/app/main.nepl" span "./util") "user/app/util.nepl" false
    let checks3 check_path_result checks2 (selfhost_module_path_resolve_import &map "user/app/nested/main.nepl" span "../shared") "user/app/shared.nepl" false
    let checks4 check_path_result checks3 (selfhost_module_path_resolve_import &map "user/app/main.nepl" span "/stdlib/core/result") "stdlib/core/result.nepl" true
    let shown checks_print_report checks4
    checks_exit_code shown
```

## builds_graph_with_stdlib_and_user_roots

neplg2:test
ret: 0
```neplg2
#entry main
#target std
#indent 4

#import "alloc/collections/vec" as *
#import "core/option" as *
#import "core/result" as *
#import "neplg2/core/module/graph" as *
#import "neplg2/core/module/loader" as *
#import "neplg2/core/infra/diag" as *
#import "neplg2/core/module/stdlib_map" as *
#import "std/test" as *

fn edge_at <(&SelfhostModuleGraph,i32)->SelfhostModuleGraphEdge> (graph, idx):
    unwrap<SelfhostModuleGraphEdge> selfhost_module_graph_edge_at graph idx

fn main <()*>i32> ():
    let root <str> "#import \"./util\" as util\n#import \"core/result\" as *\nfn main <()->i32> ():\n    0\n"
    let util <str> "fn util <()->i32> ():\n    1\n"
    let result_mod <str> "enum Result:\n    Ok\n    Err\n"
    let map <SelfhostModulePathMap> selfhost_module_path_map_new "user" "stdlib"
    let checks0 checks_new
    match selfhost_vfs_new:
        Result::Ok vfs0:
            match selfhost_vfs_add vfs0 "user/app/main.nepl" root:
                Result::Ok vfs1:
                    match selfhost_vfs_add vfs1 "user/app/util.nepl" util:
                        Result::Ok vfs2:
                            match selfhost_vfs_add vfs2 "stdlib/core/result.nepl" result_mod:
                                Result::Ok vfs3:
                                    match selfhost_build_module_graph_with_path_map &vfs3 &map "user/app/main.nepl":
                                        Result::Ok graph:
                                            let e0 <SelfhostModuleGraphEdge> edge_at &graph 0
                                            let e1 <SelfhostModuleGraphEdge> edge_at &graph 1
                                            let checks1 checks_push checks0 check_eq_i32 3 selfhost_module_graph_node_len &graph
                                            let checks2 checks_push checks1 check_eq_i32 2 selfhost_module_graph_edge_len &graph
                                            let checks3 checks_push checks2 check selfhost_module_graph_has_path &graph "user/app/main.nepl"
                                            let checks4 checks_push checks3 check selfhost_module_graph_has_path &graph "user/app/util.nepl"
                                            let checks5 checks_push checks4 check selfhost_module_graph_has_path &graph "stdlib/core/result.nepl"
                                            let checks6 checks_push checks5 check_str_eq "user/app/main.nepl" e0.from
                                            let checks7 checks_push checks6 check_str_eq "user/app/util.nepl" e0.to
                                            let checks8 checks_push checks7 check_str_eq "user/app/main.nepl" e1.from
                                            let checks9 checks_push checks8 check_str_eq "stdlib/core/result.nepl" e1.to
                                            selfhost_module_graph_free graph
                                            selfhost_vfs_free vfs3
                                            let shown checks_print_report checks9
                                            checks_exit_code shown
                                        Result::Err _diag:
                                            selfhost_vfs_free vfs3
                                            let checks1 checks_push checks0 Result<(),str>::Err "mapped graph returned Err"
                                            let shown checks_print_report checks1
                                            checks_exit_code shown
                                Result::Err _e:
                                    let checks1 checks_push checks0 Result<(),str>::Err "stdlib VFS add failed"
                                    let shown checks_print_report checks1
                                    checks_exit_code shown
                        Result::Err _e:
                            let checks1 checks_push checks0 Result<(),str>::Err "util VFS add failed"
                            let shown checks_print_report checks1
                            checks_exit_code shown
                Result::Err _e:
                    let checks1 checks_push checks0 Result<(),str>::Err "root VFS add failed"
                    let shown checks_print_report checks1
                    checks_exit_code shown
        Result::Err _e:
            let checks1 checks_push checks0 Result<(),str>::Err "VFS allocation failed"
            let shown checks_print_report checks1
            checks_exit_code shown
```

## rejects_relative_escape_above_user_root

neplg2:test
ret: 0
```neplg2
#entry main
#target std
#indent 4

#import "alloc/collections/vec" as *
#import "core/result" as *
#import "neplg2/core/infra/span" as *
#import "neplg2/core/infra/diag" as *
#import "neplg2/core/module/stdlib_map" as *
#import "std/test" as *

fn main <()*>i32> ():
    let map <SelfhostModulePathMap> selfhost_module_path_map_new "user" "stdlib"
    let checks0 checks_new
    match selfhost_module_path_resolve_import &map "user/main.nepl" source_span_empty 0 0 "../escape":
        Result::Ok _resolved:
            let checks1 checks_push checks0 Result<(),str>::Err "escape above user root was accepted"
            let shown checks_print_report checks1
            checks_exit_code shown
        Result::Err diag:
            let checks1 checks_push checks0 check_str_eq "resolve.import_path.escape_root" selfhost_diag_code_name diag.code
            let shown checks_print_report checks1
            checks_exit_code shown
```
