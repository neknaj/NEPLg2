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

fn make_vec4 %fn unit Vec i32 \unit:
    let mut v %Vec i32 unwrap_ok new;
    set v unwrap_ok push v 4;
    set v unwrap_ok push v 1;
    set v unwrap_ok push v 3;
    set v unwrap_ok push v 2;
    v

fn check_at %fn &Vec i32 fn i32 fn i32 bool \v\idx\expected:
    match get v idx:
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

fn main %impure fn unit i32 \unit:
    let v0 %Vec i32 make_vec4;
    sort_insertion &v0;
    let ok0 %bool check_sorted4 &v0;
    free v0;

    let v1 %Vec i32 make_vec4;
    sort_gnome &v1;
    let ok1 %bool check_sorted4 &v1;
    free v1;

    let v2 %Vec i32 make_vec4;
    sort_selection &v2;
    let ok2 %bool check_sorted4 &v2;
    free v2;

    let v3 %Vec i32 make_vec4;
    sort_bubble &v3;
    let ok3 %bool check_sorted4 &v3;
    free v3;

    let v4 %Vec i32 make_vec4;
    sort_cocktail &v4;
    let ok4 %bool check_sorted4 &v4;
    free v4;

    let v5 %Vec i32 make_vec4;
    sort_shell &v5;
    let ok5 %bool check_sorted4 &v5;
    free v5;

    let v6 %Vec i32 make_vec4;
    sort_comb &v6;
    let ok6 %bool check_sorted4 &v6;
    free v6;

    let ok %bool and ok0 and ok1 and ok2 and ok3 and ok4 and ok5 ok6;
    if ok 1734 0
```
