# selfhost_cliarg_parser.n.md

## selfhost_cliarg_parser_accepts_check_emit_output_and_input

neplg2:test
ret: 0
```neplg2
#entry main
#indent 4
#target std

#import "neplg2/cli/args" as *
#import "alloc/collections/vec" as v
#import "alloc/string" as *
#import "core/field" as *
#import "core/option" as *
#import "core/result" as *
#import "std/test" as *

fn main <()*>i32> ():
    let args <Vec<str>>:
        unwrap_ok v::new<str>
        |> v::push<str> "--check" |> uwok
        |> v::push<str> "--emit" |> uwok
        |> v::push<str> "wasm" |> uwok
        |> v::push<str> "-o" |> uwok
        |> v::push<str> "out.wasm" |> uwok
        |> v::push<str> "main.nepl" |> uwok
    match selfhost_cli_parse_args &args:
        Result::Err _e:
            1
        Result::Ok opts:
            let output_ref <&Option<str>> get_ref &opts "output"
            let input_ref <&Option<str>> get_ref &opts "input"
            let check_ref <&bool> get_ref &opts "check"
            let emit_ref <&SelfhostCliEmitSet> get_ref &opts "emit"
            let output_ok <bool> match *output_ref:
                Option::Some output:
                    str_eq output "out.wasm"
                Option::None:
                    false
            let input_ok <bool> match *input_ref:
                Option::Some input:
                    str_eq input "main.nepl"
                Option::None:
                    false
            let check_ok <bool> *check_ref
            let emit_ok <bool> selfhost_cli_emit_is_wasm *emit_ref
            let checks:
                checks_new
                |> checks_push assert check_ok
                |> checks_push assert emit_ok
                |> checks_push assert output_ok
                |> checks_push assert input_ok
            checks_exit_code checks
```

## selfhost_cliarg_parser_rejects_unknown_option

neplg2:test
ret: 0
```neplg2
#entry main
#indent 4
#target std

#import "neplg2/cli/args" as *
#import "alloc/collections/vec" as v
#import "core/result" as *

fn main <()*>i32> ():
    let args <Vec<str>>:
        unwrap_ok v::new<str>
        |> v::push<str> "--unknown" |> uwok
    match selfhost_cli_parse_args &args:
        Result::Ok _opts:
            1
        Result::Err e:
            if selfhost_cli_error_is_unknown_option e 0 1
```

## selfhost_cliarg_parser_rejects_missing_value

neplg2:test
ret: 0
```neplg2
#entry main
#indent 4
#target std

#import "neplg2/cli/args" as *
#import "alloc/collections/vec" as v
#import "core/result" as *

fn main <()*>i32> ():
    let args <Vec<str>>:
        unwrap_ok v::new<str>
        |> v::push<str> "--emit" |> uwok
    match selfhost_cli_parse_args &args:
        Result::Ok _opts:
            1
        Result::Err e:
            if selfhost_cli_error_is_missing_value e 0 1
```

## selfhost_cliarg_parser_rejects_multiple_input

neplg2:test
ret: 0
```neplg2
#entry main
#indent 4
#target std

#import "neplg2/cli/args" as *
#import "alloc/collections/vec" as v
#import "core/result" as *

fn main <()*>i32> ():
    let args <Vec<str>>:
        unwrap_ok v::new<str>
        |> v::push<str> "a.nepl" |> uwok
        |> v::push<str> "b.nepl" |> uwok
    match selfhost_cli_parse_args &args:
        Result::Ok _opts:
            1
        Result::Err e:
            if selfhost_cli_error_is_multiple_input e 0 1
```

## selfhost_cliarg_parser_skips_program_name

neplg2:test
ret: 0
```neplg2
#entry main
#indent 4
#target std

#import "neplg2/cli/args" as *
#import "alloc/collections/vec" as v
#import "alloc/string" as *
#import "core/field" as *
#import "core/option" as *
#import "core/result" as *
#import "std/test" as *

fn main <()*>i32> ():
    let argv <Vec<str>>:
        unwrap_ok v::new<str>
        |> v::push<str> "neplg2" |> uwok
        |> v::push<str> "--target" |> uwok
        |> v::push<str> "std" |> uwok
        |> v::push<str> "-i" |> uwok
        |> v::push<str> "main.nepl" |> uwok
    match selfhost_cli_parse_argv &argv:
        Result::Err _e:
            1
        Result::Ok opts:
            let target_ref <&Option<SelfhostCliTarget>> get_ref &opts "target"
            let input_ref <&Option<str>> get_ref &opts "input"
            let target_ok <bool> match *target_ref:
                Option::Some target:
                    selfhost_cli_target_is_wasi target
                Option::None:
                    false
            let input_ok <bool> match *input_ref:
                Option::Some input:
                    str_eq input "main.nepl"
                Option::None:
                    false
            let checks:
                checks_new
                |> checks_push assert target_ok
                |> checks_push assert input_ok
            checks_exit_code checks
```

## selfhost_cliarg_parser_records_run_args_start

neplg2:test
ret: 0
```neplg2
#entry main
#indent 4
#target std

#import "neplg2/cli/args" as *
#import "alloc/collections/vec" as v
#import "core/field" as *
#import "core/option" as *
#import "core/result" as *

fn main <()*>i32> ():
    let args <Vec<str>>:
        unwrap_ok v::new<str>
        |> v::push<str> "--run" |> uwok
        |> v::push<str> "main.nepl" |> uwok
        |> v::push<str> "--" |> uwok
        |> v::push<str> "--program-flag" |> uwok
    match selfhost_cli_parse_args &args:
        Result::Err _e:
            1
        Result::Ok opts:
            let start_ref <&Option<i32>> get_ref &opts "run_args_start"
            let start <Option<i32>> *start_ref
            match start:
                Option::Some idx:
                    if eq idx 3 0 1
                Option::None:
                    1
```

## selfhost_cliarg_parser_accepts_aliases_and_profile

neplg2:test
ret: 0
```neplg2
#entry main
#indent 4
#target std

#import "neplg2/cli/args" as *
#import "alloc/collections/vec" as v
#import "alloc/string" as *
#import "core/field" as *
#import "core/option" as *
#import "core/result" as *
#import "std/test" as *

fn main <()*>i32> ():
    let args <Vec<str>>:
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
            1
        Result::Ok opts:
            let attach_ref <&bool> get_ref &opts "attach_source"
            let lib_ref <&bool> get_ref &opts "lib"
            let verbose_ref <&bool> get_ref &opts "verbose"
            let target_ref <&Option<SelfhostCliTarget>> get_ref &opts "target"
            let emit_ref <&SelfhostCliEmitSet> get_ref &opts "emit"
            let profile_ref <&Option<SelfhostCliProfile>> get_ref &opts "profile"
            let stdlib_root_ref <&Option<str>> get_ref &opts "stdlib_root"
            let input_ref <&Option<str>> get_ref &opts "input"
            let run_args_start_ref <&Option<i32>> get_ref &opts "run_args_start"
            let attach_ok <bool> *attach_ref
            let lib_ok <bool> *lib_ref
            let verbose_ok <bool> *verbose_ref
            let target_ok <bool> match *target_ref:
                Option::Some target:
                    selfhost_cli_target_is_wasm target
                Option::None:
                    false
            let emit_ok <bool> selfhost_cli_emit_set_has_llvm_min *emit_ref
            let profile_ok <bool> match *profile_ref:
                Option::Some profile:
                    match profile:
                        SelfhostCliProfile::Debug:
                            false
                        SelfhostCliProfile::Release:
                            true
                Option::None:
                    false
            let stdlib_root_ok <bool> match *stdlib_root_ref:
                Option::Some root:
                    str_eq root "stdlib"
                Option::None:
                    false
            let input_ok <bool> match *input_ref:
                Option::Some input:
                    str_eq input "main.nepl"
                Option::None:
                    false
            let run_args_start_ok <bool> match *run_args_start_ref:
                Option::Some idx:
                    eq idx 14
                Option::None:
                    false
            let checks:
                checks_new
                |> checks_push assert attach_ok
                |> checks_push assert lib_ok
                |> checks_push assert verbose_ok
                |> checks_push assert target_ok
                |> checks_push assert emit_ok
                |> checks_push assert profile_ok
                |> checks_push assert stdlib_root_ok
                |> checks_push assert input_ok
                |> checks_push assert run_args_start_ok
            checks_exit_code checks
```

## selfhost_cliarg_parser_accepts_emit_list_and_deduplicates

neplg2:test
ret: 0
```neplg2
#entry main
#indent 4
#target std

#import "neplg2/cli/args" as *
#import "alloc/collections/vec" as v
#import "core/field" as *
#import "core/math" as *
#import "core/result" as *
#import "std/test" as *

fn main <()*>i32> ():
    let args <Vec<str>>:
        unwrap_ok v::new<str>
        |> v::push<str> "--emit" |> uwok
        |> v::push<str> "wasm,wat,llvm-min,wasm" |> uwok
        |> v::push<str> "main.nepl" |> uwok
    match selfhost_cli_parse_args &args:
        Result::Err _e:
            1
        Result::Ok opts:
            let emit_ref <&SelfhostCliEmitSet> get_ref &opts "emit"
            let emit <SelfhostCliEmitSet> *emit_ref
            let checks:
                checks_new
                |> checks_push assert selfhost_cli_emit_is_wasm emit
                |> checks_push assert selfhost_cli_emit_set_has_wat emit
                |> checks_push assert selfhost_cli_emit_set_has_llvm_min emit
                |> checks_push assert not selfhost_cli_emit_set_has_wat_min emit
                |> checks_push assert not selfhost_cli_emit_set_has_llvm emit
            checks_exit_code checks
```

## selfhost_cliarg_parser_accepts_emit_all

neplg2:test
ret: 0
```neplg2
#entry main
#indent 4
#target std

#import "neplg2/cli/args" as *
#import "alloc/collections/vec" as v
#import "core/field" as *
#import "core/result" as *
#import "std/test" as *

fn main <()*>i32> ():
    let args <Vec<str>>:
        unwrap_ok v::new<str>
        |> v::push<str> "--emit" |> uwok
        |> v::push<str> "all" |> uwok
        |> v::push<str> "main.nepl" |> uwok
    match selfhost_cli_parse_args &args:
        Result::Err _e:
            1
        Result::Ok opts:
            let emit_ref <&SelfhostCliEmitSet> get_ref &opts "emit"
            let emit <SelfhostCliEmitSet> *emit_ref
            let checks:
                checks_new
                |> checks_push assert selfhost_cli_emit_is_wasm emit
                |> checks_push assert selfhost_cli_emit_set_has_wat emit
                |> checks_push assert selfhost_cli_emit_set_has_wat_min emit
                |> checks_push assert selfhost_cli_emit_set_has_llvm emit
                |> checks_push assert selfhost_cli_emit_set_has_llvm_min emit
            checks_exit_code checks
```

## selfhost_cliarg_parser_rejects_invalid_emit_member

neplg2:test
ret: 0
```neplg2
#entry main
#indent 4
#target std

#import "neplg2/cli/args" as *
#import "alloc/collections/vec" as v
#import "core/result" as *

fn main <()*>i32> ():
    let args <Vec<str>>:
        unwrap_ok v::new<str>
        |> v::push<str> "--emit" |> uwok
        |> v::push<str> "wasm,,wat" |> uwok
    match selfhost_cli_parse_args &args:
        Result::Ok _opts:
            1
        Result::Err e:
            if selfhost_cli_error_is_invalid_emit e 0 1
```
