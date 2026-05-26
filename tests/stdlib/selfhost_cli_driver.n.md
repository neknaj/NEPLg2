# selfhost CLI driver

## selfhost_cli_driver_success_vfs_returns_zero

neplg2:test[stdio, normalize_newlines]
stdout: mlstr:
##: Checked [ok,ok]
##: [0] ok
##: [1] ok
exit_code: 0
```neplg2
#entry main
#indent 4
#target std

#import "alloc/collections/vec" as v
#import "core/result" as *
#import "neplg2/cli/args" as *
#import "neplg2/cli/driver" as *
#import "neplg2/core/infra/diag" as *
#import "neplg2/core/module/loader" as *
#import "std/test" as *

fn main %impure fn unit i32 \unit:
    let args %Vec str:
        unwrap_ok v::new
        |> v::push "--target" |> uwok
        |> v::push "std" |> uwok
        |> v::push "-i" |> uwok
        |> v::push "main.nepl" |> uwok
    match selfhost_cli_parse_args &args:
        Result::Err _e:
            v::free args;
            1
        Result::Ok options:
            let vfs0 %SelfhostVirtualFileSystem unwrap_ok selfhost_vfs_new
            let vfs1 %SelfhostVirtualFileSystem unwrap_ok selfhost_vfs_add vfs0 "main.nepl" "fn main <()->i32> \():\n    0\n"
            match selfhost_cli_driver_compile_vfs &vfs1 options:
                Result::Err _e:
                    selfhost_vfs_free vfs1
                    v::free args;
                    2
                Result::Ok result:
                    let exit_code %i32 selfhost_cli_driver_result_exit_code &result
                    let diagnostics %&SelfhostDiagnostics selfhost_cli_driver_result_diagnostics &result
                    let diag_len %i32 selfhost_diagnostics_len diagnostics
                    selfhost_cli_driver_result_free result
                    selfhost_vfs_free vfs1
                    v::free args;
                    let checks:
                        checks_new
                        |> checks_push assert_eq_i32 0 exit_code
                        |> checks_push assert_eq_i32 0 diag_len
                    let shown checks_print_report checks;
                    checks_exit_code shown
```

## selfhost_cli_driver_missing_input_writes_json_diagnostic

neplg2:test[normalize_newlines]
ret: 0
stdout: "[{\"severity\":\"error\",\"code\":\"cli.input.missing\",\"message\":\"input source file is required\",\"primary_label\":null,\"note\":\"pass -i/--input or a positional input path\"}]"
```neplg2
#entry main
#indent 4
#target std

#import "core/result" as *
#import "neplg2/cli/args" as *
#import "neplg2/cli/driver" as *
#import "neplg2/core/module/loader" as *
#import "core/math" as *

fn main %impure fn unit i32 \unit:
    let options %SelfhostCliOptions selfhost_cli_default_options
    let vfs %SelfhostVirtualFileSystem unwrap_ok selfhost_vfs_new
    match selfhost_cli_driver_compile_vfs &vfs options:
        Result::Err _e:
            selfhost_vfs_free vfs
            1
        Result::Ok result:
            let exit_code %i32 selfhost_cli_driver_result_exit_code &result
            match selfhost_cli_driver_write_json_stdout &result:
                Result::Err _e:
                    selfhost_cli_driver_result_free result
                    selfhost_vfs_free vfs
                    2
                Result::Ok _:
                    selfhost_cli_driver_result_free result
                    selfhost_vfs_free vfs
                    if eq exit_code 1 0 3
```

## selfhost_cli_driver_missing_file_returns_loader_diagnostic

neplg2:test[stdio, normalize_newlines]
stdout: mlstr:
##: Checked [ok,ok]
##: [0] ok
##: [1] ok
exit_code: 0
```neplg2
#entry main
#indent 4
#target std

#import "alloc/collections/vec" as v
#import "core/result" as *
#import "neplg2/cli/args" as *
#import "neplg2/cli/driver" as *
#import "neplg2/cli/reporter" as *
#import "neplg2/core/infra/diag" as *
#import "neplg2/core/module/loader" as *
#import "std/test" as *
#import "core/math" as *

fn main %impure fn unit i32 \unit:
    let args %Vec str:
        unwrap_ok v::new
        |> v::push "-i" |> uwok
        |> v::push "missing.nepl" |> uwok
    match selfhost_cli_parse_args &args:
        Result::Err _e:
            v::free args;
            1
        Result::Ok options:
            let vfs %SelfhostVirtualFileSystem unwrap_ok selfhost_vfs_new
            match selfhost_cli_driver_compile_vfs &vfs options:
                Result::Err _e:
                    selfhost_vfs_free vfs
                    v::free args;
                    2
                Result::Ok result:
                    let exit_code %i32 selfhost_cli_driver_result_exit_code &result
                    let diagnostics %&SelfhostDiagnostics selfhost_cli_driver_result_diagnostics &result
                    let json %str selfhost_cli_render_diagnostics_json diagnostics
                    selfhost_cli_driver_result_free result
                    selfhost_vfs_free vfs
                    v::free args;
                    let checks:
                        checks_new
                        |> checks_push assert_eq_i32 1 exit_code
                        |> checks_push assert_str_eq "[{\"severity\":\"error\",\"code\":\"loader.source.file_not_found\",\"message\":\"source file is not registered in self-host VFS\",\"primary_label\":null,\"note\":\"missing.nepl\"}]" json
                    let shown checks_print_report checks;
                    checks_exit_code shown
```
