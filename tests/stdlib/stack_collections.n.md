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
