# stdlib/hashmap_str.n.md

## hashmap_str_main

neplg2:test
```neplg2

#entry main
#indent 4
#target std

#import "alloc/collections/hashmap" as *
#import "core/traits/hash" as *
#import "alloc/diag/error" as *
#import "alloc/string" as *
#import "core/math" as *
#import "core/option" as *
#import "core/result" as *
#import "std/test" as *

fn must_hms <(Result<HashMap<str,i32,DefaultHash32>, Diag>)*>HashMap<str,i32,DefaultHash32>> (r):
    match r:
        Result::Ok hm:
            hm
        Result::Err _d:
            #intrinsic "unreachable" <> ()

fn main <()*> i32> ():
    let mut checks <Vec<Result<(),str>>> checks_new;
    let hm0 <HashMap<str,i32,DefaultHash32>> must_hms new DefaultHash32;
    set checks checks_push checks check_eq_i32 0 len hm0;

    let hm1 <HashMap<str,i32,DefaultHash32>> must_hms new DefaultHash32;
    set checks checks_push checks check not contains hm1 "foo";

    let hm2 <HashMap<str,i32,DefaultHash32>> must_hms new DefaultHash32;
    set checks checks_push checks check is_none<i32> get hm2 "foo";

    let hm3 <HashMap<str,i32,DefaultHash32>> must_hms new DefaultHash32;
    let hm3 <HashMap<str,i32,DefaultHash32>> must_hms insert hm3 "foo" 10;
    let hm3 <HashMap<str,i32,DefaultHash32>> must_hms insert hm3 "bar" 20;
    let hm3_len <i32> len hm3;
    set checks checks_push checks check_eq_i32 2 hm3_len;

    let hm3a <HashMap<str,i32,DefaultHash32>> must_hms new DefaultHash32;
    let hm3a <HashMap<str,i32,DefaultHash32>> must_hms insert hm3a "foo" 10;
    let hm3a <HashMap<str,i32,DefaultHash32>> must_hms insert hm3a "bar" 20;
    set checks checks_push checks check contains hm3a "foo";

    let hm3b <HashMap<str,i32,DefaultHash32>> must_hms new DefaultHash32;
    let hm3b <HashMap<str,i32,DefaultHash32>> must_hms insert hm3b "foo" 10;
    let hm3b <HashMap<str,i32,DefaultHash32>> must_hms insert hm3b "bar" 20;
    set checks checks_push checks check contains hm3b "bar";

    let hm3c <HashMap<str,i32,DefaultHash32>> must_hms new DefaultHash32;
    let hm3c <HashMap<str,i32,DefaultHash32>> must_hms insert hm3c "foo" 10;
    let hm3c <HashMap<str,i32,DefaultHash32>> must_hms insert hm3c "bar" 20;
    set checks checks_push checks check not contains hm3c "baz";

    let s1 <str> concat "a" "b";
    let s2 <str> concat "a" "b";
    let hm4 <HashMap<str,i32,DefaultHash32>> must_hms new DefaultHash32;
    let hm4 <HashMap<str,i32,DefaultHash32>> must_hms insert hm4 s1 30;
    match get hm4 s2:
        Option::Some v:
            set checks checks_push checks check_eq_i32 30 v
        Option::None:
            set checks checks_push checks Result<(),str>::Err "get with same content returned None";

    let hm5 <HashMap<str,i32,DefaultHash32>> must_hms new DefaultHash32;
    let hm5 <HashMap<str,i32,DefaultHash32>> must_hms insert hm5 "foo" 10;
    let hm5 <HashMap<str,i32,DefaultHash32>> must_hms insert hm5 "foo" 11;
    match get hm5 "foo":
        Option::Some v:
            set checks checks_push checks check_eq_i32 11 v
        Option::None:
            set checks checks_push checks Result<(),str>::Err "get foo after update returned None";

    let hm6 <HashMap<str,i32,DefaultHash32>> must_hms new DefaultHash32;
    let hm6 <HashMap<str,i32,DefaultHash32>> must_hms insert hm6 "foo" 10;
    let hm6 <HashMap<str,i32,DefaultHash32>> must_hms insert hm6 "bar" 20;
    let hm6 <HashMap<str,i32,DefaultHash32>> must_hms remove hm6 "bar";
    set checks checks_push checks check not contains hm6 "bar";

    let hm7 <HashMap<str,i32,DefaultHash32>> must_hms new DefaultHash32;
    let hm7 <HashMap<str,i32,DefaultHash32>> must_hms insert hm7 "foo" 10;
    set checks checks_push checks check is_err<HashMap<str,i32,DefaultHash32>, Diag> remove hm7 "zzz";

    let hmf <HashMap<str,i32,DefaultHash32>> must_hms new DefaultHash32;
    let hmf <HashMap<str,i32,DefaultHash32>> must_hms insert hmf "x" 1;
    free hmf;
    let shown <Vec<Result<(),str>>> checks_print_report checks;
    checks_exit_code shown
```
