# simple sort algorithms

## sort_simple_algorithms_via_public_facade

neplg2:test
ret: 1734
```neplg2
#entry main
#indent 4
#target core
#import "alloc/collections/vec" as *
#import "alloc/collections/vec/sort" as *
#import "core/math" as *
#import "core/option" as *
#import "core/result" as *
#import "core/field" as *

fn make_vec4 %fn () Vec i32 \():
    let mut v %Vec i32 unwrap_ok new<i32>;
    set v unwrap_ok push<i32> v 4;
    set v unwrap_ok push<i32> v 1;
    set v unwrap_ok push<i32> v 3;
    set v unwrap_ok push<i32> v 2;
    v

fn check_at %fn &Vec i32 fn i32 fn i32 bool \v\idx\expected:
    match get<i32> v idx:
        Option::Some value:
            eq value expected
        Option::None:
            false

fn check_sorted4 %fn &Vec i32 bool \v:
    let b0 %bool check_at v 0 1;
    let b1 %bool check_at v 1 2;
    let b2 %bool check_at v 2 3;
    let b3 %bool check_at v 3 4;
    and b0 and b1 and b2 b3

fn main %impure fn () i32 \():
    let v0 %Vec i32 make_vec4;
    sort_insertion<i32> &v0;
    let ok0 %bool check_sorted4 &v0;
    free<i32> v0;

    let v1 %Vec i32 make_vec4;
    sort_gnome<i32> &v1;
    let ok1 %bool check_sorted4 &v1;
    free<i32> v1;

    let v2 %Vec i32 make_vec4;
    sort_selection<i32> &v2;
    let ok2 %bool check_sorted4 &v2;
    free<i32> v2;

    let v3 %Vec i32 make_vec4;
    sort_bubble<i32> &v3;
    let ok3 %bool check_sorted4 &v3;
    free<i32> v3;

    let v4 %Vec i32 make_vec4;
    sort_cocktail<i32> &v4;
    let ok4 %bool check_sorted4 &v4;
    free<i32> v4;

    let v5 %Vec i32 make_vec4;
    sort_shell<i32> &v5;
    let ok5 %bool check_sorted4 &v5;
    free<i32> v5;

    let v6 %Vec i32 make_vec4;
    sort_comb<i32> &v6;
    let ok6 %bool check_sorted4 &v6;
    free<i32> v6;

    let ok %bool and ok0 and ok1 and ok2 and ok3 and ok4 and ok5 ok6;
    if ok 1734 0
```
