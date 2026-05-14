# stdlib/cliarg.n.md

## cliarg_basic

neplg2:test
argv: ["--flag", "value"]
ret: 1
```neplg2
#entry main
#indent 4
#target std

#import "std/env/cliarg" as *
#import "core/math" as *
#import "core/option" as *

fn main <()*>i32> ():
    let c <i32> cliarg_count;
    let neg_missing <bool> is_none<str> cliarg_get -1;
    let end_missing <bool> is_none<str> cliarg_get c;
    if and and ge c 0 neg_missing end_missing 1 0
```

## cliarg_argv_stdout_count

neplg2:test[assert_io]
argv: ["--flag", "value"]
stdout: "3"
```neplg2
#entry main
#indent 4
#target std

#import "std/env/cliarg" as *
#import "std/stdio" as *

fn main <()*>()> ():
    print_i32 cliarg_count;
```

## cliarg_get_reads_injected_argv_values

neplg2:test[assert_io]
argv: ["--flag", "value"]
stdout: "--flag:value"
```neplg2
#entry main
#indent 4
#target std

#import "std/env/cliarg" as *
#import "std/stdio" as *
#import "alloc/string" as *
#import "core/option" as *

fn print_arg <(i32)*>()> (idx):
    match cliarg_get idx:
        Option::Some arg:
            print arg
        Option::None:
            print "<none>"

fn main <()*>()> ():
    print_arg 1;
    print ":";
    print_arg 2;
```

## cliarg_get_rejects_out_of_range

neplg2:test
argv: ["--flag", "value"]
ret: 0
```neplg2
#entry main
#indent 4
#target std

#import "std/env/cliarg" as *
#import "core/option" as *
#import "core/math" as *

fn main <()*>i32> ():
    let c <i32> cliarg_count;
    let neg_missing <bool> is_none<str> cliarg_get -1;
    let end_missing <bool> is_none<str> cliarg_get c;
    if and neg_missing end_missing 0 1
```

## cliarg_cstr_requires_mem_ptr

neplg2:test[compile_fail]
diag_code: type.overload.no_match
```neplg2
#entry main
#indent 4
#target std

#import "std/env/cliarg/cstr" as *

fn main <()*>()> ():
    let _n cstr_len 0;
```

## cliarg_cstr_to_str_requires_mem_ptr

neplg2:test[compile_fail]
diag_code: type.overload.no_match
```neplg2
#entry main
#indent 4
#target std

#import "std/env/cliarg/cstr" as *

fn main <()*>()> ():
    let _s cstr_to_str 0;
```
