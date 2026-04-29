# stdlib/list.n.md

## list_main

neplg2:test
ret: 0
```neplg2

#entry main
#indent 4
#target std

#import "alloc/collections/list" as *
#import "alloc/diag/error" as *
#import "core/option" as *
#import "core/result" as *
#import "std/test" as *

fn mk <()*>List<i32>> ():
    let l0 <List<i32>> unwrap_ok<List<i32>, Diag> new<i32>;
    let l1 <List<i32>> uwok cons<i32> 10 l0;
    let l2 <List<i32>> uwok cons<i32> 20 l1;
    uwok cons<i32> 30 l2

fn main <()*>i32> ():
    let mut checks checks_new;
    let l0 <List<i32>> unwrap_ok<List<i32>, Diag> new<i32>;
    set checks checks_push checks check_eq_i32 0 len<i32> l0;

    let l0a <List<i32>> unwrap_ok<List<i32>, Diag> new<i32>;
    let l1 <List<i32>> uwok cons<i32> 10 l0a;
    set checks checks_push checks check_eq_i32 1 len<i32> l1;

    let l0b <List<i32>> unwrap_ok<List<i32>, Diag> new<i32>;
    let l1b <List<i32>> uwok cons<i32> 10 l0b;
    let l2 <List<i32>> uwok cons<i32> 20 l1b;
    set checks checks_push checks check_eq_i32 2 len<i32> l2;

    let l3 <List<i32>> mk;
    set checks checks_push checks check_eq_i32 3 len<i32> l3;

    let l3_0 <List<i32>> mk;
    let l3_1 <List<i32>> mk;
    let l3_2 <List<i32>> mk;
    match get<i32> l3_0 0:
        Option::Some x:
            set checks checks_push checks check_eq_i32 30 x
        Option::None:
            set checks checks_push checks Result<(),str>::Err "get 0 returned None";

    match get<i32> l3_1 1:
        Option::Some x:
            set checks checks_push checks check_eq_i32 20 x
        Option::None:
            set checks checks_push checks Result<(),str>::Err "get 1 returned None";

    match get<i32> l3_2 2:
        Option::Some x:
            set checks checks_push checks check_eq_i32 10 x
        Option::None:
            set checks checks_push checks Result<(),str>::Err "get 2 returned None";

    let l3_3 <List<i32>> mk;
    let l3_100 <List<i32>> mk;
    set checks checks_push checks check is_none<i32> get<i32> l3_3 3;
    set checks checks_push checks check is_none<i32> get<i32> l3_100 100;

    let l3_n1 <List<i32>> mk;
    set checks checks_push checks check is_none<i32> get<i32> l3_n1 -1;

    let l3h <List<i32>> mk;
    match head<i32> l3h:
        Option::Some x:
            set checks checks_push checks check_eq_i32 30 x
        Option::None:
            set checks checks_push checks Result<(),str>::Err "head returned None";

    let l3t <List<i32>> mk;
    match tail<i32> l3t:
        Option::Some l3_tail:
            match head<i32> l3_tail:
                Option::Some x:
                    set checks checks_push checks check_eq_i32 20 x
                Option::None:
                    set checks checks_push checks Result<(),str>::Err "head tail returned None";
        Option::None:
            set checks checks_push checks Result<(),str>::Err "tail returned None";

    let l3r0 <List<i32>> mk;
    let l_rev <List<i32>> reverse<i32> l3r0;
    match get<i32> l_rev 0:
        Option::Some x:
            set checks checks_push checks check_eq_i32 10 x
        Option::None:
            set checks checks_push checks Result<(),str>::Err "get reverse 0 returned None";

    let l3r1 <List<i32>> mk;
    let l_rev2 <List<i32>> reverse<i32> l3r1;
    match get<i32> l_rev2 2:
        Option::Some x:
            set checks checks_push checks check_eq_i32 30 x
        Option::None:
            set checks checks_push checks Result<(),str>::Err "get reverse 2 returned None";

    let lf <List<i32>> mk;
    free<i32> lf;
    let shown checks_print_report checks;
    checks_exit_code shown
```

## list_functional_helpers

neplg2:test
ret: 0
```neplg2

#entry main
#indent 4
#target std

#import "alloc/collections/list" as *
#import "alloc/diag/error" as *
#import "core/math" as *
#import "core/option" as *
#import "core/result" as *
#import "std/test" as *

fn mk <()*>List<i32>> ():
    let xs <List<i32>>:
        unwrap_ok<List<i32>, Diag> new<i32>
        |> push<i32> 4 |> uwok
        |> push<i32> 3 |> uwok
        |> push<i32> 2 |> uwok
        |> push<i32> 1 |> uwok
    xs

fn inc <(i32)->i32> (x):
    add x 1

fn is_even <(i32)->bool> (x):
    eq rem_s x 2 0

fn add_acc <(i32,i32)->i32> (acc, x):
    add acc x

fn gt_two <(i32)->bool> (x):
    gt x 2

fn main <()*>i32> ():
    let mut checks checks_new;

    let mapped_src0 <List<i32>> mk;
    let mapped0 <List<i32>> uwok map<i32,i32> mapped_src0 inc;
    match get<i32> mapped0 0:
        Option::Some x:
            set checks checks_push checks check_eq_i32 2 x
        Option::None:
            set checks checks_push checks Result<(),str>::Err "map get 0 returned None";

    let mapped_src3 <List<i32>> mk;
    let mapped3 <List<i32>> uwok map<i32,i32> mapped_src3 inc;
    match get<i32> mapped3 3:
        Option::Some x:
            set checks checks_push checks check_eq_i32 5 x
        Option::None:
            set checks checks_push checks Result<(),str>::Err "map get 3 returned None";

    let filtered_len_src <List<i32>> mk;
    let filtered_len_list <List<i32>> uwok filter<i32> filtered_len_src is_even;
    set checks checks_push checks check_eq_i32 2 len<i32> filtered_len_list;

    let filtered_src0 <List<i32>> mk;
    let filtered0 <List<i32>> uwok filter<i32> filtered_src0 is_even;
    match get<i32> filtered0 0:
        Option::Some x:
            set checks checks_push checks check_eq_i32 2 x
        Option::None:
            set checks checks_push checks Result<(),str>::Err "filter get 0 returned None";

    let filtered_src1 <List<i32>> mk;
    let filtered1 <List<i32>> uwok filter<i32> filtered_src1 is_even;
    match get<i32> filtered1 1:
        Option::Some x:
            set checks checks_push checks check_eq_i32 4 x
        Option::None:
            set checks checks_push checks Result<(),str>::Err "filter get 1 returned None";

    let folded_src <List<i32>> mk;
    set checks checks_push checks check_eq_i32 10 fold<i32,i32> folded_src 0 add_acc;

    let reduced_src <List<i32>> mk;
    match reduce<i32> reduced_src add_acc:
        Option::Some x:
            set checks checks_push checks check_eq_i32 10 x
        Option::None:
            set checks checks_push checks Result<(),str>::Err "reduce returned None";

    let find_src <List<i32>> mk;
    match find<i32> find_src gt_two:
        Option::Some x:
            set checks checks_push checks check_eq_i32 3 x
        Option::None:
            set checks checks_push checks Result<(),str>::Err "find returned None";

    let any_src <List<i32>> mk;
    set checks checks_push checks check any<i32> any_src gt_two;

    let all_src <List<i32>> mk;
    set checks checks_push checks check not all<i32> all_src is_even;

    let shown checks_print_report checks;
    checks_exit_code shown
```
