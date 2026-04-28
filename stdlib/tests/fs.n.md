# stdlib/fs.n.md

## fs_main

neplg2:test
```neplg2

#entry main
#indent 4
#target std

#import "std/fs" as *
#import "alloc/string" as *
#import "core/math" as *
#import "core/result" as *
#import "std/test" as *

fn main <()*>i32> ():
    let mut checks checks_new;
    match fs_read_to_string "__definitely_missing_file__.txt":
        Result::Ok s:
            set checks checks_push checks Result<(),str>::Err "fs_read_to_string unexpectedly succeeded"
        Result::Err e:
            set checks checks_push checks Result<(),str>::Ok ();
    let shown checks_print_report checks;
    checks_exit_code shown
```
