# stdlib/queue.n.md

## queue_push_pop

neplg2:test
ret: 1
```neplg2
#entry main
#indent 4
#target std

#import "alloc/collections/queue" as *
#import "alloc/diag/error" as *
#import "core/math" as *
#import "core/option" as *
#import "core/result" as *

fn main <()*>i32> ():
    let q0 <Queue<i32>>:
        unwrap_ok<Queue<i32>, Diag> new<i32>
        |> push<i32> 1
        |> unwrap_ok<Queue<i32>, Diag>
        |> push<i32> 2
        |> unwrap_ok<Queue<i32>, Diag>
    let ok0 <bool> eq len<i32> q0 2;
    let q1 <Queue<i32>>:
        unwrap_ok<Queue<i32>, Diag> new<i32>
        |> push<i32> 1
        |> unwrap_ok<Queue<i32>, Diag>
        |> push<i32> 2
        |> unwrap_ok<Queue<i32>, Diag>
    let ok1 <bool> match peek<i32> q1:
        Option::Some v:
            eq v 1
        Option::None:
            false
    let q2 <Queue<i32>>:
        unwrap_ok<Queue<i32>, Diag> new<i32>
        |> push<i32> 5
        |> unwrap_ok<Queue<i32>, Diag>
    let ok2 <bool> match pop<i32> q2:
        Option::Some v:
            eq v 5
        Option::None:
            false
    if and ok0 and ok1 ok2 1 0
```

## queue_pop_empty

neplg2:test
ret: 1
```neplg2
#entry main
#indent 4
#target std

#import "alloc/collections/queue" as *
#import "alloc/diag/error" as *
#import "core/option" as *
#import "core/result" as *

fn main <()*>i32> ():
    let q <Queue<i32>> unwrap_ok<Queue<i32>, Diag> new<i32>;
    match pop<i32> q:
        Option::Some _:
            0
        Option::None:
            1
```
