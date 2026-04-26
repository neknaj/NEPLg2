# stdlib/btreeset.n.md

## btreeset_insert_and_len

neplg2:test
```neplg2
#entry main
#indent 4
#target std

#import "alloc/collections/btreeset" as *
#import "alloc/diag/error" as *
#import "std/test" as { checks_new, checks_push, checks_print_report, checks_exit_code, check_eq_i32, check }
#import "core/result" as *

fn must_set <(Result<BTreeSet<i32>, Diag>)*>BTreeSet<i32>> (r):
    match r:
        Result::Ok s:
            s
        Result::Err _d:
            #intrinsic "unreachable" <> ()

fn main <()*>i32> ():
    let mut checks <Vec<Result<(),str>>> checks_new;

    let s0 <BTreeSet<i32>>:
        unwrap_ok<BTreeSet<i32>, Diag> new<i32>
        |> insert<i32> 5
        |> must_set
        |> insert<i32> 1
        |> must_set
        |> insert<i32> 3
        |> must_set
    set checks checks_push checks check_eq_i32 3 len<i32> s0;

    let shown <Vec<Result<(),str>>> checks_print_report checks;
    checks_exit_code shown
```

## btreeset_insert_growth_boundary

neplg2:test
```neplg2
#entry main
#indent 4
#target std

#import "alloc/collections/btreeset" as *
#import "alloc/diag/error" as *
#import "std/test" as { checks_new, checks_push, checks_print_report, checks_exit_code, check }
#import "core/result" as *

fn must_set <(Result<BTreeSet<i32>, Diag>)*>BTreeSet<i32>> (r):
    match r:
        Result::Ok s:
            s
        Result::Err _d:
            #intrinsic "unreachable" <> ()

fn main <()*>i32> ():
    let mut checks <Vec<Result<(),str>>> checks_new;

    let s0 <BTreeSet<i32>>:
        unwrap_ok<BTreeSet<i32>, Diag> new<i32>
        |> insert<i32> 0
        |> must_set
        |> insert<i32> 1
        |> must_set
        |> insert<i32> 2
        |> must_set
        |> insert<i32> 3
        |> must_set
        |> insert<i32> 4
        |> must_set
        |> insert<i32> 5
        |> must_set
        |> insert<i32> 6
        |> must_set
        |> insert<i32> 7
        |> must_set
        |> insert<i32> 8
        |> must_set
    set checks checks_push checks check contains<i32> s0 8;

    let shown <Vec<Result<(),str>>> checks_print_report checks;
    checks_exit_code shown
```

## btreeset_contains_and_remove

neplg2:test
```neplg2
#entry main
#indent 4
#target std

#import "alloc/collections/btreeset" as *
#import "alloc/diag/error" as *
#import "std/test" as { checks_new, checks_push, checks_print_report, checks_exit_code, check_eq_i32, check }
#import "core/result" as *

fn must_set <(Result<BTreeSet<i32>, Diag>)*>BTreeSet<i32>> (r):
    match r:
        Result::Ok s:
            s
        Result::Err _d:
            #intrinsic "unreachable" <> ()

fn main <()*>i32> ():
    let mut checks <Vec<Result<(),str>>> checks_new;

    let s0 <BTreeSet<i32>>:
        unwrap_ok<BTreeSet<i32>, Diag> new<i32>
        |> insert<i32> 5
        |> must_set
        |> insert<i32> 1
        |> must_set
    set checks checks_push checks check contains<i32> s0 1;

    let s1 <BTreeSet<i32>>:
        unwrap_ok<BTreeSet<i32>, Diag> new<i32>
        |> insert<i32> 5
        |> must_set
        |> insert<i32> 1
        |> must_set
        |> remove<i32> 1
    set checks checks_push checks check not contains<i32> s1 1;

    let s2 <BTreeSet<i32>>:
        unwrap_ok<BTreeSet<i32>, Diag> new<i32>
        |> insert<i32> 5
        |> must_set
        |> insert<i32> 1
        |> must_set
        |> remove<i32> 1
    set checks checks_push checks check_eq_i32 1 len<i32> s2;

    let shown <Vec<Result<(),str>>> checks_print_report checks;
    checks_exit_code shown
```

## btreeset_duplicate_insert

neplg2:test
```neplg2
#entry main
#indent 4
#target std

#import "alloc/collections/btreeset" as *
#import "alloc/diag/error" as *
#import "std/test" as { checks_new, checks_push, checks_print_report, checks_exit_code, check_eq_i32 }
#import "core/result" as *

fn must_set <(Result<BTreeSet<i32>, Diag>)*>BTreeSet<i32>> (r):
    match r:
        Result::Ok s:
            s
        Result::Err _d:
            #intrinsic "unreachable" <> ()

fn main <()*>i32> ():
    let mut checks <Vec<Result<(),str>>> checks_new;

    let s0 <BTreeSet<i32>>:
        unwrap_ok<BTreeSet<i32>, Diag> new<i32>
        |> insert<i32> 3
        |> must_set
        |> insert<i32> 3
        |> must_set
    set checks checks_push checks check_eq_i32 1 len<i32> s0;

    let shown <Vec<Result<(),str>>> checks_print_report checks;
    checks_exit_code shown
```
