# stdlib/vec.n.md

## vec_main

neplg2:test
ret: 0
```neplg2

#entry main
#indent 4
#target std

#import "alloc/collections/vec" as *
#import "core/cast" as *
#import "core/option" as *
#import "std/test" as *

fn main <()*>i32> ():
    let mut checks <Vec<Result<(),str>>> checks_new;
    let v0_empty <Vec<i32>> unwrap_ok new<i32>;
    set checks checks_push checks check is_empty<i32> v0_empty;
    let v0_ptr <Vec<i32>> unwrap_ok new<i32>;
    set checks checks_push checks check gt data_ptr<i32> v0_ptr 0;

    let v2:
        unwrap_ok new<i32>
        |> push<i32> 10
        |> unwrap_ok
    set checks checks_push checks check_eq_i32 1 len<i32> v2;

    let v6:
        unwrap_ok new<i32>
        |> push<i32> 10
        |> unwrap_ok
        |> push<i32> 20
        |> unwrap_ok
        |> push<i32> 30
        |> unwrap_ok
    set checks checks_push checks check_eq_i32 3 len<i32> v6;

    let g2:
        unwrap_ok new<i32>
        |> push<i32> 10
        |> unwrap_ok
        |> push<i32> 20
        |> unwrap_ok
    match get<i32> g2 0:
        Option::Some x:
            set checks checks_push checks check_eq_i32 10 x
        Option::None:
            set checks checks_push checks Result<(),str>::Err "get 0 returned None";

    let s2:
        unwrap_ok new<i32>
        |> push<i32> 10
        |> unwrap_ok
        |> push<i32> 20
        |> unwrap_ok
    replace<i32> s2 1 21;
    let s2_ref:
        unwrap_ok new<i32>
        |> push<i32> 10
        |> unwrap_ok
        |> push<i32> 20
        |> unwrap_ok
    replace_ref<i32> &s2_ref 0 11;
    match get_ref<i32> &s2_ref 0:
        Option::Some x:
            set checks checks_push checks check_eq_i32 11 x
        Option::None:
            set checks checks_push checks Result<(),str>::Err "replace_ref get 0 returned None";

    let o1:
        unwrap_ok new<i32>
        |> push<i32> 10
        |> unwrap_ok
    set checks checks_push checks check is_none<i32> get<i32> o1 2;

    let p1:
        unwrap_ok new<i32>
        |> push<i32> 10
        |> unwrap_ok
    set checks checks_push checks check is_none<i32> get<i32> p1 -1;

    let u8_65 <u8> cast 65;
    let b1:
        unwrap_ok new<u8>
        |> push<u8> u8_65
        |> unwrap_ok
    match get<u8> b1 0:
        Option::Some x:
            set checks checks_push checks check_eq_i32 65 cast x
        Option::None:
            set checks checks_push checks Result<(),str>::Err "get<u8> returned None";

    let shown <Vec<Result<(),str>>> checks_print_report checks;
    checks_exit_code shown
```

## vec_functional_helpers

neplg2:test
ret: 0
```neplg2

#entry main
#indent 4
#target std

#import "alloc/collections/vec" as *
#import "core/math" as *
#import "core/option" as *
#import "std/test" as *

fn inc <(i32)->i32> (x):
    add x 1

fn is_even <(i32)->bool> (x):
    eq rem_s x 2 0

fn add_acc <(i32,i32)->i32> (acc, x):
    add acc x

fn gt_two <(i32)->bool> (x):
    gt x 2

fn lt_four <(i32)->bool> (x):
    lt x 4

fn main <()*>i32> ():
    let mut checks <Vec<Result<(),str>>> checks_new;

    let mapped_src <Vec<i32>>:
        unwrap_ok new<i32>
        |> push 1 |> uwok
        |> push 2 |> uwok
        |> push 3 |> uwok
    let mapped <Vec<i32>> unwrap_ok map<i32,i32> mapped_src inc;
    match get<i32> mapped 2:
        Option::Some x:
            set checks checks_push checks check_eq_i32 4 x
        Option::None:
            set checks checks_push checks Result<(),str>::Err "vec map returned None";

    let filtered_len_src <Vec<i32>>:
        unwrap_ok new<i32>
        |> push 1 |> uwok
        |> push 2 |> uwok
        |> push 3 |> uwok
        |> push 4 |> uwok
    let filtered_len <Vec<i32>> unwrap_ok filter<i32> filtered_len_src is_even;
    set checks checks_push checks check_eq_i32 2 len<i32> filtered_len;
    let filtered_get_src <Vec<i32>>:
        unwrap_ok new<i32>
        |> push 1 |> uwok
        |> push 2 |> uwok
        |> push 3 |> uwok
        |> push 4 |> uwok
    let filtered_get <Vec<i32>> unwrap_ok filter<i32> filtered_get_src is_even;
    match get<i32> filtered_get 1:
        Option::Some x:
            set checks checks_push checks check_eq_i32 4 x
        Option::None:
            set checks checks_push checks Result<(),str>::Err "vec filter returned None";

    let folded_src <Vec<i32>>:
        unwrap_ok new<i32>
        |> push 1 |> uwok
        |> push 2 |> uwok
        |> push 3 |> uwok
        |> push 4 |> uwok
    set checks checks_push checks check_eq_i32 10 fold<i32,i32> folded_src 0 add_acc;

    let reduced_src <Vec<i32>>:
        unwrap_ok new<i32>
        |> push 1 |> uwok
        |> push 2 |> uwok
        |> push 3 |> uwok
        |> push 4 |> uwok
    match reduce<i32> reduced_src add_acc:
        Option::Some x:
            set checks checks_push checks check_eq_i32 10 x
        Option::None:
            set checks checks_push checks Result<(),str>::Err "vec reduce returned None";

    let find_src <Vec<i32>>:
        unwrap_ok new<i32>
        |> push 1 |> uwok
        |> push 2 |> uwok
        |> push 3 |> uwok
    match find<i32> find_src gt_two:
        Option::Some x:
            set checks checks_push checks check_eq_i32 3 x
        Option::None:
            set checks checks_push checks Result<(),str>::Err "vec find returned None";

    let any_src <Vec<i32>>:
        unwrap_ok new<i32>
        |> push 1 |> uwok
        |> push 2 |> uwok
        |> push 3 |> uwok
    set checks checks_push checks check any<i32> any_src gt_two;

    let all_src <Vec<i32>>:
        unwrap_ok new<i32>
        |> push 2 |> uwok
        |> push 4 |> uwok
        |> push 6 |> uwok
    set checks checks_push checks check all<i32> all_src is_even;

    let partition_even_len_src <Vec<i32>>:
        unwrap_ok new<i32>
        |> push 1 |> uwok
        |> push 2 |> uwok
        |> push 3 |> uwok
        |> push 4 |> uwok
    let parts_even_len unwrap_ok partition<i32> partition_even_len_src is_even;
    let evens_len <Vec<i32>> get parts_even_len 0;
    set checks checks_push checks check_eq_i32 2 len<i32> evens_len;
    let partition_even_get_src <Vec<i32>>:
        unwrap_ok new<i32>
        |> push 1 |> uwok
        |> push 2 |> uwok
        |> push 3 |> uwok
        |> push 4 |> uwok
    let parts_even_get unwrap_ok partition<i32> partition_even_get_src is_even;
    let evens_get <Vec<i32>> get parts_even_get 0;
    match get<i32> evens_get 1:
        Option::Some x:
            set checks checks_push checks check_eq_i32 4 x
        Option::None:
            set checks checks_push checks Result<(),str>::Err "vec partition evens returned None";
    let partition_odds_len_src <Vec<i32>>:
        unwrap_ok new<i32>
        |> push 1 |> uwok
        |> push 2 |> uwok
        |> push 3 |> uwok
        |> push 4 |> uwok
    let parts_odds_len unwrap_ok partition<i32> partition_odds_len_src is_even;
    let odds_len <Vec<i32>> get parts_odds_len 1;
    set checks checks_push checks check_eq_i32 2 len<i32> odds_len;
    let partition_odds_get_src <Vec<i32>>:
        unwrap_ok new<i32>
        |> push 1 |> uwok
        |> push 2 |> uwok
        |> push 3 |> uwok
        |> push 4 |> uwok
    let parts_odds_get unwrap_ok partition<i32> partition_odds_get_src is_even;
    let odds_get <Vec<i32>> get parts_odds_get 1;
    match get<i32> odds_get 0:
        Option::Some x:
            set checks checks_push checks check_eq_i32 1 x
        Option::None:
            set checks checks_push checks Result<(),str>::Err "vec partition odds returned None";

    let take_src <Vec<i32>>:
        unwrap_ok new<i32>
        |> push 1 |> uwok
        |> push 2 |> uwok
        |> push 3 |> uwok
        |> push 5 |> uwok
        |> push 6 |> uwok
    let taken <Vec<i32>> unwrap_ok take_while<i32> take_src lt_four;
    set checks checks_push checks check_eq_i32 3 len<i32> taken;

    let drop_src <Vec<i32>>:
        unwrap_ok new<i32>
        |> push 1 |> uwok
        |> push 2 |> uwok
        |> push 3 |> uwok
        |> push 5 |> uwok
        |> push 6 |> uwok
    let dropped <Vec<i32>> unwrap_ok drop_while<i32> drop_src lt_four;
    match get<i32> dropped 0:
        Option::Some x:
            set checks checks_push checks check_eq_i32 5 x
        Option::None:
            set checks checks_push checks Result<(),str>::Err "vec drop_while returned None";

    let count_src <Vec<i32>>:
        unwrap_ok new<i32>
        |> push 1 |> uwok
        |> push 2 |> uwok
        |> push 3 |> uwok
        |> push 4 |> uwok
        |> push 5 |> uwok
    set checks checks_push checks check_eq_i32 2 count<i32> count_src is_even;

    let shown <Vec<Result<(),str>>> checks_print_report checks;
    checks_exit_code shown
```
