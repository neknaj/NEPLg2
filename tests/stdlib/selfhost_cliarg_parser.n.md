# selfhost_cliarg_parser.n.md

## selfhost_cliarg_parser_contract

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"selfhost_cliarg_parser\" count=10 failed=0\nassertion index=0 status=ok kind=bool label=\"accepts check emit output and input\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=1 status=ok kind=bool label=\"rejects unknown option\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=2 status=ok kind=bool label=\"rejects missing value\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=3 status=ok kind=bool label=\"rejects multiple input\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=4 status=ok kind=bool label=\"skips program name\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=5 status=ok kind=bool label=\"records run args start\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=6 status=ok kind=bool label=\"accepts aliases and profile\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=7 status=ok kind=bool label=\"accepts emit list and deduplicates\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=8 status=ok kind=bool label=\"accepts emit all\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=9 status=ok kind=bool label=\"rejects invalid emit member\" expected=\"true\" actual=\"true\" message=\"\"\n"
```neplg2
#entry main
#indent 4
#target std

#import "neplg2/cli/args" as *
#import "alloc/collections/vec" as v
#import "alloc/string" as *
#import "core/field" as *
#import "core/math" as *
#import "core/option" as *
#import "core/result" as *
#import "std/test" as *

fn selfhost_cliarg_parser_accepts_check_emit_output_and_input %impure fn unit bool \unit:
    let args %Vec str:
        unwrap_ok v::new<str>
        |> v::push<str> "--check" |> uwok
        |> v::push<str> "--emit" |> uwok
        |> v::push<str> "wasm" |> uwok
        |> v::push<str> "-o" |> uwok
        |> v::push<str> "out.wasm" |> uwok
        |> v::push<str> "main.nepl" |> uwok
    match selfhost_cli_parse_args &args:
        Result::Err _e:
            v::free<str> args;
            false
        Result::Ok opts:
            let output_ref %&Option str get_ref &opts "output"
            let input_ref %&Option str get_ref &opts "input"
            let check_ref %&bool get_ref &opts "check"
            let emit_ref %&SelfhostCliEmitSet get_ref &opts "emit"
            let output_ok %bool match *output_ref:
                Option::Some output:
                    str_eq output "out.wasm"
                Option::None:
                    false
            let input_ok %bool match *input_ref:
                Option::Some input:
                    str_eq input "main.nepl"
                Option::None:
                    false
            let check_ok %bool *check_ref
            let emit_ok %bool selfhost_cli_emit_is_wasm *emit_ref
            let ok %bool and check_ok and emit_ok and output_ok input_ok
            v::free<str> args;
            ok

fn selfhost_cliarg_parser_rejects_unknown_option %impure fn unit bool \unit:
    let args %Vec str:
        unwrap_ok v::new<str>
        |> v::push<str> "--unknown" |> uwok
    match selfhost_cli_parse_args &args:
        Result::Ok _opts:
            v::free<str> args;
            false
        Result::Err e:
            let ok %bool selfhost_cli_error_is_unknown_option e
            v::free<str> args;
            ok

fn selfhost_cliarg_parser_rejects_missing_value %impure fn unit bool \unit:
    let args %Vec str:
        unwrap_ok v::new<str>
        |> v::push<str> "--emit" |> uwok
    match selfhost_cli_parse_args &args:
        Result::Ok _opts:
            v::free<str> args;
            false
        Result::Err e:
            let ok %bool selfhost_cli_error_is_missing_value e
            v::free<str> args;
            ok

fn selfhost_cliarg_parser_rejects_multiple_input %impure fn unit bool \unit:
    let args %Vec str:
        unwrap_ok v::new<str>
        |> v::push<str> "a.nepl" |> uwok
        |> v::push<str> "b.nepl" |> uwok
    match selfhost_cli_parse_args &args:
        Result::Ok _opts:
            v::free<str> args;
            false
        Result::Err e:
            let ok %bool selfhost_cli_error_is_multiple_input e
            v::free<str> args;
            ok

fn selfhost_cliarg_parser_skips_program_name %impure fn unit bool \unit:
    let argv %Vec str:
        unwrap_ok v::new<str>
        |> v::push<str> "neplg2" |> uwok
        |> v::push<str> "--target" |> uwok
        |> v::push<str> "std" |> uwok
        |> v::push<str> "-i" |> uwok
        |> v::push<str> "main.nepl" |> uwok
    match selfhost_cli_parse_argv &argv:
        Result::Err _e:
            v::free<str> argv;
            false
        Result::Ok opts:
            let target_ref %&Option SelfhostCliTarget get_ref &opts "target"
            let input_ref %&Option str get_ref &opts "input"
            let target_ok %bool match *target_ref:
                Option::Some target:
                    selfhost_cli_target_is_wasi target
                Option::None:
                    false
            let input_ok %bool match *input_ref:
                Option::Some input:
                    str_eq input "main.nepl"
                Option::None:
                    false
            let ok %bool and target_ok input_ok
            v::free<str> argv;
            ok

fn selfhost_cliarg_parser_records_run_args_start %impure fn unit bool \unit:
    let args %Vec str:
        unwrap_ok v::new<str>
        |> v::push<str> "--run" |> uwok
        |> v::push<str> "main.nepl" |> uwok
        |> v::push<str> "--" |> uwok
        |> v::push<str> "--program-flag" |> uwok
    match selfhost_cli_parse_args &args:
        Result::Err _e:
            v::free<str> args;
            false
        Result::Ok opts:
            let start_ref %&Option i32 get_ref &opts "run_args_start"
            let start %Option i32 *start_ref
            let ok %bool match start:
                Option::Some idx:
                    eq idx 3
                Option::None:
                    false
            v::free<str> args;
            ok

fn selfhost_cliarg_parser_accepts_aliases_and_profile %impure fn unit bool \unit:
    let args %Vec str:
        unwrap_ok v::new<str>
        |> v::push<str> "--attach-source" |> uwok
        |> v::push<str> "--lib" |> uwok
        |> v::push<str> "-v" |> uwok
        |> v::push<str> "--target" |> uwok
        |> v::push<str> "core" |> uwok
        |> v::push<str> "--emit" |> uwok
        |> v::push<str> "llvm-min" |> uwok
        |> v::push<str> "--profile" |> uwok
        |> v::push<str> "release" |> uwok
        |> v::push<str> "--stdlib-root" |> uwok
        |> v::push<str> "stdlib" |> uwok
        |> v::push<str> "-i" |> uwok
        |> v::push<str> "main.nepl" |> uwok
        |> v::push<str> "--" |> uwok
        |> v::push<str> "--program-flag" |> uwok
    match selfhost_cli_parse_args &args:
        Result::Err _e:
            v::free<str> args;
            false
        Result::Ok opts:
            let attach_ref %&bool get_ref &opts "attach_source"
            let lib_ref %&bool get_ref &opts "lib"
            let verbose_ref %&bool get_ref &opts "verbose"
            let target_ref %&Option SelfhostCliTarget get_ref &opts "target"
            let emit_ref %&SelfhostCliEmitSet get_ref &opts "emit"
            let profile_ref %&Option SelfhostCliProfile get_ref &opts "profile"
            let stdlib_root_ref %&Option str get_ref &opts "stdlib_root"
            let input_ref %&Option str get_ref &opts "input"
            let run_args_start_ref %&Option i32 get_ref &opts "run_args_start"
            let attach_ok %bool *attach_ref
            let lib_ok %bool *lib_ref
            let verbose_ok %bool *verbose_ref
            let target_ok %bool match *target_ref:
                Option::Some target:
                    selfhost_cli_target_is_wasm target
                Option::None:
                    false
            let emit_ok %bool selfhost_cli_emit_set_has_llvm_min *emit_ref
            let profile_ok %bool match *profile_ref:
                Option::Some profile:
                    match profile:
                        SelfhostCliProfile::Debug:
                            false
                        SelfhostCliProfile::Release:
                            true
                Option::None:
                    false
            let stdlib_root_ok %bool match *stdlib_root_ref:
                Option::Some root:
                    str_eq root "stdlib"
                Option::None:
                    false
            let input_ok %bool match *input_ref:
                Option::Some input:
                    str_eq input "main.nepl"
                Option::None:
                    false
            let run_args_start_ok %bool match *run_args_start_ref:
                Option::Some idx:
                    eq idx 14
                Option::None:
                    false
            let ok %bool:
                and attach_ok:
                    and lib_ok:
                        and verbose_ok:
                            and target_ok:
                                and emit_ok:
                                    and profile_ok:
                                        and stdlib_root_ok:
                                            and input_ok run_args_start_ok
            v::free<str> args;
            ok

fn selfhost_cliarg_parser_accepts_emit_list_and_deduplicates %impure fn unit bool \unit:
    let args %Vec str:
        unwrap_ok v::new<str>
        |> v::push<str> "--emit" |> uwok
        |> v::push<str> "wasm,wat,llvm-min,wasm" |> uwok
        |> v::push<str> "main.nepl" |> uwok
    match selfhost_cli_parse_args &args:
        Result::Err _e:
            v::free<str> args;
            false
        Result::Ok opts:
            let emit_ref %&SelfhostCliEmitSet get_ref &opts "emit"
            let emit %SelfhostCliEmitSet *emit_ref
            let ok %bool:
                and selfhost_cli_emit_is_wasm emit:
                    and selfhost_cli_emit_set_has_wat emit:
                        and selfhost_cli_emit_set_has_llvm_min emit:
                            and not selfhost_cli_emit_set_has_wat_min emit:
                                not selfhost_cli_emit_set_has_llvm emit
            v::free<str> args;
            ok

fn selfhost_cliarg_parser_accepts_emit_all %impure fn unit bool \unit:
    let args %Vec str:
        unwrap_ok v::new<str>
        |> v::push<str> "--emit" |> uwok
        |> v::push<str> "all" |> uwok
        |> v::push<str> "main.nepl" |> uwok
    match selfhost_cli_parse_args &args:
        Result::Err _e:
            v::free<str> args;
            false
        Result::Ok opts:
            let emit_ref %&SelfhostCliEmitSet get_ref &opts "emit"
            let emit %SelfhostCliEmitSet *emit_ref
            let ok %bool:
                and selfhost_cli_emit_is_wasm emit:
                    and selfhost_cli_emit_set_has_wat emit:
                        and selfhost_cli_emit_set_has_wat_min emit:
                            and selfhost_cli_emit_set_has_llvm emit:
                                selfhost_cli_emit_set_has_llvm_min emit
            v::free<str> args;
            ok

fn selfhost_cliarg_parser_rejects_invalid_emit_member %impure fn unit bool \unit:
    let args %Vec str:
        unwrap_ok v::new<str>
        |> v::push<str> "--emit" |> uwok
        |> v::push<str> "wasm,,wat" |> uwok
    match selfhost_cli_parse_args &args:
        Result::Ok _opts:
            v::free<str> args;
            false
        Result::Err e:
            let ok %bool selfhost_cli_error_is_invalid_emit e
            v::free<str> args;
            ok

fn main %impure fn unit i32 \unit:
    let report %TestReport:
        test_report_new "selfhost_cliarg_parser"
        |> test_report_push assert "accepts check emit output and input" selfhost_cliarg_parser_accepts_check_emit_output_and_input
        |> test_report_push assert "rejects unknown option" selfhost_cliarg_parser_rejects_unknown_option
        |> test_report_push assert "rejects missing value" selfhost_cliarg_parser_rejects_missing_value
        |> test_report_push assert "rejects multiple input" selfhost_cliarg_parser_rejects_multiple_input
        |> test_report_push assert "skips program name" selfhost_cliarg_parser_skips_program_name
        |> test_report_push assert "records run args start" selfhost_cliarg_parser_records_run_args_start
        |> test_report_push assert "accepts aliases and profile" selfhost_cliarg_parser_accepts_aliases_and_profile
        |> test_report_push assert "accepts emit list and deduplicates" selfhost_cliarg_parser_accepts_emit_list_and_deduplicates
        |> test_report_push assert "accepts emit all" selfhost_cliarg_parser_accepts_emit_all
        |> test_report_push assert "rejects invalid emit member" selfhost_cliarg_parser_rejects_invalid_emit_member
    let shown test_report_print_stdout report
    test_report_exit_code shown
```
