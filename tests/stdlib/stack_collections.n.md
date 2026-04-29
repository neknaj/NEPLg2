# tests/stack_collections.n.md

## stack_new_and_len

neplg2:test
ret: 1
```neplg2
#entry main
#indent 4
#target std

#import "alloc/collections/stack" as *
#import "alloc/diag/error" as *
#import "core/math" as *
#import "core/result" as *

fn main <()*>i32> ():
    let mut s <Stack<i32>> unwrap_ok<Stack<i32>, Diag> new<i32>;
    set s unwrap_ok<Stack<i32>, Diag> push<i32> s 10;
    set s unwrap_ok<Stack<i32>, Diag> push<i32> s 20;
    if eq len<i32> s 2 1 0
```

## stack_peek_and_pop

neplg2:test
ret: 1
```neplg2
#entry main
#indent 4
#target std

#import "alloc/collections/stack" as *
#import "alloc/diag/error" as *
#import "core/math" as *
#import "core/option" as *
#import "core/field" as *
#import "core/result" as *

fn main <()*>i32> ():
    let s0 <Stack<i32>>:
        unwrap_ok<Stack<i32>, Diag> new<i32>
        |> push<i32> 10
        |> unwrap_ok<Stack<i32>, Diag>
        |> push<i32> 20
        |> unwrap_ok<Stack<i32>, Diag>
    let ok0 <bool> match peek<i32> s0:
        Option::Some v:
            eq v 20
        Option::None:
            false
    let s1 <Stack<i32>>:
        unwrap_ok<Stack<i32>, Diag> new<i32>
        |> push<i32> 10
        |> unwrap_ok<Stack<i32>, Diag>
        |> push<i32> 20
        |> unwrap_ok<Stack<i32>, Diag>
    let p pop<i32> s1;
    let ok1 <bool> match p:
        Option::Some v:
            eq v 20
        Option::None:
            false
    if and ok0 ok1 1 0
```

## stack_pop_empty

neplg2:test
ret: 1
```neplg2
#entry main
#indent 4
#target std

#import "alloc/collections/stack" as *
#import "alloc/diag/error" as *
#import "core/math" as *
#import "core/option" as *
#import "core/field" as *
#import "core/result" as *

fn main <()*>i32> ():
    let s <Stack<i32>> unwrap_ok<Stack<i32>, Diag> new<i32>;
    let p pop<i32> s;
    match p:
        Option::Some _:
            0
        Option::None:
            1
```

## stack_new_and_len_pipe

neplg2:test
ret: 1
```neplg2
#entry main
#indent 4
#target std

#import "alloc/collections/stack" as *
#import "alloc/diag/error" as *
#import "core/math" as *
#import "core/result" as *

fn main <()*>i32> ():
    let s <Stack<i32>>:
        unwrap_ok<Stack<i32>, Diag> new<i32>
        |> push<i32> 10
        |> unwrap_ok<Stack<i32>, Diag>
        |> push<i32> 20
        |> unwrap_ok<Stack<i32>, Diag>
    if eq len<i32> s 2 1 0
```

## stack_peek_and_pop_pipe

neplg2:test
ret: 1
```neplg2
#entry main
#indent 4
#target std

#import "alloc/collections/stack" as *
#import "alloc/diag/error" as *
#import "core/math" as *
#import "core/option" as *
#import "core/field" as *
#import "core/result" as *

fn main <()*>i32> ():
    let s0 <Stack<i32>>:
        unwrap_ok<Stack<i32>, Diag> new<i32>
        |> push<i32> 10
        |> unwrap_ok<Stack<i32>, Diag>
        |> push<i32> 20
        |> unwrap_ok<Stack<i32>, Diag>
    let ok0 <bool> match s0 |> peek<i32>:
        Option::Some v:
            eq v 20
        Option::None:
            false
    let s1 <Stack<i32>>:
        unwrap_ok<Stack<i32>, Diag> new<i32>
        |> push<i32> 10
        |> unwrap_ok<Stack<i32>, Diag>
        |> push<i32> 20
        |> unwrap_ok<Stack<i32>, Diag>
    let p <Option<i32>> pop<i32> s1;
    let ok1 <bool> match p:
        Option::Some v:
            eq v 20
        Option::None:
            false
    if and ok0 ok1 1 0
```

## stack_pop_empty_pipe

neplg2:test
ret: 1
```neplg2
#entry main
#indent 4
#target std

#import "alloc/collections/stack" as *
#import "alloc/diag/error" as *
#import "core/math" as *
#import "core/option" as *
#import "core/field" as *
#import "core/result" as *

fn main <()*>i32> ():
    let s <Stack<i32>> unwrap_ok<Stack<i32>, Diag> new<i32>;
    let p <Option<i32>> pop<i32> s;
    match p:
        Option::Some _:
            0
        Option::None:
            1
```

## stack_get_ref_keeps_stack

neplg2:test
ret: 1
```neplg2
#entry main
#indent 4
#target std

#import "alloc/collections/stack" as *
#import "alloc/diag/error" as *
#import "core/math" as *
#import "core/option" as *
#import "core/result" as *

fn main <()*>i32> ():
    let mut s <Stack<i32>> unwrap_ok<Stack<i32>, Diag> new<i32>;
    set s unwrap_ok<Stack<i32>, Diag> push<i32> s 10;
    set s unwrap_ok<Stack<i32>, Diag> push<i32> s 20;
    let first_ok <bool> match get_ref<i32> &s 0:
        Option::Some v:
            eq v 10
        Option::None:
            false
    let len_before <i32> len_ref<i32> &s;
    set s unwrap_ok<Stack<i32>, Diag> push<i32> s 30;
    let ok <bool> and first_ok and eq len_before 2 eq len_ref<i32> &s 3;
    free<i32> s;
    if ok 1 0
```

## stack_pop_ref_keeps_stack

neplg2:test
ret: 1
```neplg2
#entry main
#indent 4
#target std

#import "alloc/collections/stack" as *
#import "alloc/diag/error" as *
#import "core/field" as field
#import "core/math" as *
#import "core/option" as *
#import "core/result" as *

fn main <()*>i32> ():
    let mut s <Stack<i32>> unwrap_ok<Stack<i32>, Diag> new<i32>;
    set s unwrap_ok<Stack<i32>, Diag> push<i32> s 10;
    set s unwrap_ok<Stack<i32>, Diag> push<i32> s 20;
    let p0 <StackPop<i32>> pop_top<i32> s;
    let a <Option<i32>> *field::get_ref &p0 "item";
    let s1 <Stack<i32>> field::get p0 "stack";
    let p1 <StackPop<i32>> pop_top<i32> s1;
    let b <Option<i32>> *field::get_ref &p1 "item";
    let s2 <Stack<i32>> field::get p1 "stack";
    let empty_ok <bool> eq len_ref<i32> &s2 0;
    let s3 <Stack<i32>> unwrap_ok<Stack<i32>, Diag> push<i32> s2 30;
    let a_ok <bool> match a:
        Option::Some v:
            eq v 20
        Option::None:
            false
    let b_ok <bool> match b:
        Option::Some v:
            eq v 10
        Option::None:
            false
    let ok <bool> and a_ok and b_ok and empty_ok eq len_ref<i32> &s3 1;
    free<i32> s3;
    if ok 1 0
```

## stack_grow_clear_free_reallocates

[目的/もくてき]:
- `Stack` が[容量/ようりょう]拡張後に `clear` / `free` しても trap せず、その後の[再確保/さいかくほ]が[正常/せいじょう]に[動作/どうさ]することを[確認/かくにん]します。

[何/なに]を[確/たし]かめるか:
- `new`
- `push`
- grow
- `clear`
- `free`

neplg2:test
ret: 1
```neplg2
#entry main
#indent 4
#target std

#import "alloc/collections/stack" as *
#import "alloc/diag/error" as *
#import "core/math" as *
#import "core/option" as *
#import "core/result" as *

fn main <()*>i32> ():
    let mut s <Stack<i32>> unwrap_ok<Stack<i32>, Diag> new<i32>;
    set s unwrap_ok<Stack<i32>, Diag> push<i32> s 0;
    set s unwrap_ok<Stack<i32>, Diag> push<i32> s 1;
    set s unwrap_ok<Stack<i32>, Diag> push<i32> s 2;
    set s unwrap_ok<Stack<i32>, Diag> push<i32> s 3;
    set s unwrap_ok<Stack<i32>, Diag> push<i32> s 4;
    set s unwrap_ok<Stack<i32>, Diag> push<i32> s 5;
    set s unwrap_ok<Stack<i32>, Diag> push<i32> s 6;
    set s unwrap_ok<Stack<i32>, Diag> push<i32> s 7;
    set s unwrap_ok<Stack<i32>, Diag> push<i32> s 8;
    set s unwrap_ok<Stack<i32>, Diag> push<i32> s 9;
    let grown_ok <bool> eq len_ref<i32> &s 10;
    set s clear<i32> s;
    let clear_ok <bool> eq len_ref<i32> &s 0;
    free<i32> s;
    let mut next <Stack<i32>> unwrap_ok<Stack<i32>, Diag> new<i32>;
    set next unwrap_ok<Stack<i32>, Diag> push<i32> next 42;
    let top_ok <bool> match peek_ref<i32> &next:
        Option::Some v:
            eq v 42
        Option::None:
            false
    free<i32> next;
    if and grown_ok and clear_ok top_ok 1 0
```
