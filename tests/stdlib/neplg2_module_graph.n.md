# NEPLg2 self-host module graph

## builds_transitive_import_graph

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
#import "neplg2/core/module/graph" as *
#import "neplg2/core/module/loader" as *
#import "std/test" as *

fn edge_at <(&SelfhostModuleGraph,i32)->SelfhostModuleGraphEdge> (graph, idx):
    unwrap<SelfhostModuleGraphEdge> selfhost_module_graph_edge_at graph idx

fn main <()*>i32> ():
    let root <str> "#import \"util.nepl\" as util\n#import \"leaf.nepl\" as *\nfn main <()->i32> ():\n    0\n"
    let util <str> "#import \"leaf.nepl\" as leaf\nfn util <()->i32> ():\n    1\n"
    let leaf <str> "fn leaf <()->i32> ():\n    2\n"
    let checks0 checks_new
    match selfhost_vfs_new:
        Result::Ok vfs0:
            match selfhost_vfs_add vfs0 "main.nepl" root:
                Result::Ok vfs1:
                    match selfhost_vfs_add vfs1 "util.nepl" util:
                        Result::Ok vfs2:
                            match selfhost_vfs_add vfs2 "leaf.nepl" leaf:
                                Result::Ok vfs3:
                                    match selfhost_build_module_graph &vfs3 "main.nepl":
                                        Result::Ok graph:
                                            let e0 <SelfhostModuleGraphEdge> edge_at &graph 0
                                            let e1 <SelfhostModuleGraphEdge> edge_at &graph 1
                                            let checks1 checks_push checks0 check_eq_i32 3 selfhost_module_graph_node_len &graph
                                            let checks2 checks_push checks1 check_eq_i32 3 selfhost_module_graph_edge_len &graph
                                            let checks3 checks_push checks2 check selfhost_module_graph_has_path &graph "main.nepl"
                                            let checks4 checks_push checks3 check selfhost_module_graph_has_path &graph "util.nepl"
                                            let checks5 checks_push checks4 check selfhost_module_graph_has_path &graph "leaf.nepl"
                                            let checks6 checks_push checks5 check_str_eq "main.nepl" e0.from
                                            let checks7 checks_push checks6 check_str_eq "util.nepl" e0.to
                                            let checks8 checks_push checks7 check_str_eq "util.nepl" e1.from
                                            let checks9 checks_push checks8 check_str_eq "leaf.nepl" e1.to
                                            selfhost_module_graph_free graph
                                            selfhost_vfs_free vfs3
                                            let shown checks_print_report checks9
                                            checks_exit_code shown
                                        Result::Err _diag:
                                            selfhost_vfs_free vfs3
                                            let checks1 checks_push checks0 Result<(),str>::Err "module graph returned Err"
                                            let shown checks_print_report checks1
                                            checks_exit_code shown
                                Result::Err _e:
                                    let checks1 checks_push checks0 Result<(),str>::Err "leaf VFS add failed"
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

## reports_missing_import_module

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
#import "neplg2/core/module/graph" as *
#import "neplg2/core/module/loader" as *
#import "std/test" as *

fn check_note <(Checks, Option<str>)*>Checks> (checks, note):
    match note:
        Option::Some text:
            checks_push checks check_str_eq "missing.nepl" text
        Option::None:
            checks_push checks Result<(),str>::Err "missing module diagnostic note was absent"

fn main <()*>i32> ():
    let root <str> "#import \"missing.nepl\" as *\nfn main <()->i32> ():\n    0\n"
    let checks0 checks_new
    match selfhost_vfs_new:
        Result::Ok vfs0:
            match selfhost_vfs_add vfs0 "main.nepl" root:
                Result::Ok vfs1:
                    match selfhost_build_module_graph &vfs1 "main.nepl":
                        Result::Ok graph:
                            selfhost_module_graph_free graph
                            selfhost_vfs_free vfs1
                            let checks1 checks_push checks0 Result<(),str>::Err "missing import was accepted"
                            let shown checks_print_report checks1
                            checks_exit_code shown
                        Result::Err diag:
                            let checks1 checks_push checks0 check_str_eq "resolve.import_graph.missing_module" selfhost_diag_code_name diag.code
                            let checks2 check_note checks1 diag.note
                            selfhost_vfs_free vfs1
                            let shown checks_print_report checks2
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

## reports_import_cycle

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
#import "neplg2/core/module/graph" as *
#import "neplg2/core/module/loader" as *
#import "std/test" as *

fn check_note <(Checks, Option<str>)*>Checks> (checks, note):
    match note:
        Option::Some text:
            checks_push checks check_str_eq "a.nepl" text
        Option::None:
            checks_push checks Result<(),str>::Err "cycle diagnostic note was absent"

fn main <()*>i32> ():
    let source_a <str> "#import \"b.nepl\" as b\nfn a <()->i32> ():\n    1\n"
    let source_b <str> "#import \"a.nepl\" as a\nfn b <()->i32> ():\n    2\n"
    let checks0 checks_new
    match selfhost_vfs_new:
        Result::Ok vfs0:
            match selfhost_vfs_add vfs0 "a.nepl" source_a:
                Result::Ok vfs1:
                    match selfhost_vfs_add vfs1 "b.nepl" source_b:
                        Result::Ok vfs2:
                            match selfhost_build_module_graph &vfs2 "a.nepl":
                                Result::Ok graph:
                                    selfhost_module_graph_free graph
                                    selfhost_vfs_free vfs2
                                    let checks1 checks_push checks0 Result<(),str>::Err "import cycle was accepted"
                                    let shown checks_print_report checks1
                                    checks_exit_code shown
                                Result::Err diag:
                                    let checks1 checks_push checks0 check_str_eq "resolve.import_graph.cycle" selfhost_diag_code_name diag.code
                                    let checks2 check_note checks1 diag.note
                                    selfhost_vfs_free vfs2
                                    let shown checks_print_report checks2
                                    checks_exit_code shown
                        Result::Err _e:
                            let checks1 checks_push checks0 Result<(),str>::Err "b VFS add failed"
                            let shown checks_print_report checks1
                            checks_exit_code shown
                Result::Err _e:
                    let checks1 checks_push checks0 Result<(),str>::Err "a VFS add failed"
                    let shown checks_print_report checks1
                    checks_exit_code shown
        Result::Err _e:
            let checks1 checks_push checks0 Result<(),str>::Err "VFS allocation failed"
            let shown checks_print_report checks1
            checks_exit_code shown
```
