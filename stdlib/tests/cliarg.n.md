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
    let c cliarg_count;
    let _a cliarg_get -1;
    let _b cliarg_get c;
    let _p cliarg_program;
    if ge c 0 1 0
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

## cliarg_cstr_requires_mem_ptr

neplg2:test[compile_fail]
diag_id: D3006
```neplg2
#entry main
#indent 4
#target std

#import "std/env/cliarg" as *

fn main <()*>()> ():
    let _n cstr_len 0;
```

## cliarg_cstr_to_str_requires_mem_ptr

neplg2:test[compile_fail]
diag_id: D3006
```neplg2
#entry main
#indent 4
#target std

#import "std/env/cliarg" as *

fn main <()*>()> ():
    let _s cstr_to_str 0;
```
