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
#import "core/mem" as *
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
            let opts_mem <i32> alloc_raw size_of<SelfhostCliOptions>;
            store<SelfhostCliOptions> opts_mem opts;
            let output_ok <bool> match get load<SelfhostCliOptions> opts_mem "output":
                Option::Some output:
                    str_eq output "out.wasm"
                Option::None:
                    false
            let input_ok <bool> match get load<SelfhostCliOptions> opts_mem "input":
                Option::Some input:
                    str_eq input "main.nepl"
                Option::None:
                    false
            let check_ok <bool> get load<SelfhostCliOptions> opts_mem "check";
            let emit_ok <bool> selfhost_cli_emit_is_wasm get load<SelfhostCliOptions> opts_mem "emit";
            dealloc_raw opts_mem size_of<SelfhostCliOptions>;
            let checks <Vec<Result<(),str>>>:
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
#import "core/mem" as *
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
            let opts_mem <i32> alloc_raw size_of<SelfhostCliOptions>;
            store<SelfhostCliOptions> opts_mem opts;
            let target_ok <bool> match get load<SelfhostCliOptions> opts_mem "target":
                Option::Some target:
                    selfhost_cli_target_is_wasi target
                Option::None:
                    false
            let input_ok <bool> match get load<SelfhostCliOptions> opts_mem "input":
                Option::Some input:
                    str_eq input "main.nepl"
                Option::None:
                    false
            dealloc_raw opts_mem size_of<SelfhostCliOptions>;
            let checks <Vec<Result<(),str>>>:
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
#import "core/mem" as *
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
            let opts_mem <i32> alloc_raw size_of<SelfhostCliOptions>;
            store<SelfhostCliOptions> opts_mem opts;
            let start <Option<i32>> get load<SelfhostCliOptions> opts_mem "run_args_start";
            dealloc_raw opts_mem size_of<SelfhostCliOptions>;
            match start:
                Option::Some idx:
                    if eq idx 3 0 1
                Option::None:
                    1
```
